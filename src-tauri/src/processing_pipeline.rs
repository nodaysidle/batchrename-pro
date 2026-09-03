use crate::db;
use crate::file_service;
use crate::preview_service::{self, RenameStep};
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};

static CANCEL_FLAGS: once_cell::sync::Lazy<
    Arc<Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>>,
> = once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

enum DbAccess<'a> {
    Direct(&'a rusqlite::Connection),
    Locked(&'a Mutex<Option<rusqlite::Connection>>),
}

impl DbAccess<'_> {
    fn with<T>(&self, f: impl FnOnce(&rusqlite::Connection) -> Result<T, String>) -> Result<T, String> {
        match self {
            DbAccess::Direct(conn) => f(conn),
            DbAccess::Locked(mutex) => {
                let guard = mutex.lock().unwrap();
                let conn = guard.as_ref().ok_or("DB_NOT_INIT")?;
                f(conn)
            }
        }
    }
}

pub struct PreparedRename {
    pub job_id: String,
    files: Vec<FileInfo>,
    plan: Vec<RenameStep>,
    job_file_ids: Vec<String>,
    backup_dir: PathBuf,
    cancel_flag: Arc<AtomicBool>,
    start_time: Instant,
}

#[allow(dead_code)]
pub fn execute_batch_rename(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    registry: &Mutex<HashMap<String, FileInfo>>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<String, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("APP_DATA_ERROR: {}", e))?;
    execute_batch_rename_with_paths(conn, &app_data, Some(app), Some(registry), files, pattern)
}

pub fn execute_batch_rename_with_paths(
    conn: &rusqlite::Connection,
    app_data: &Path,
    app: Option<&AppHandle>,
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<String, String> {
    let prepared = prepare_batch_rename(conn, app_data, app, files, pattern)?;
    let job_id = prepared.job_id.clone();
    run_prepared_rename(DbAccess::Direct(conn), app, registry, prepared, None)?;
    Ok(job_id)
}

/// Create the job row, cancel flag, and occupancy plan. Does not rename.
/// The caller must drop any DB lock before `run_prepared_rename`.
pub fn prepare_batch_rename(
    conn: &rusqlite::Connection,
    app_data: &Path,
    app: Option<&AppHandle>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<PreparedRename, String> {
    if files.is_empty() {
        return Err("NO_FILES: Add files before applying rename".into());
    }

    let previews = preview_service::generate_previews(&files, &pattern)?;
    prepare_batch_rename_from_previews(conn, app_data, app, files, previews)
}

pub fn prepare_batch_rename_from_previews(
    conn: &rusqlite::Connection,
    app_data: &Path,
    app: Option<&AppHandle>,
    files: Vec<FileInfo>,
    previews: Vec<crate::types::PreviewPair>,
) -> Result<PreparedRename, String> {
    let conflicts: Vec<String> = previews
        .iter()
        .filter(|p| p.has_conflict)
        .map(|p| {
            format!(
                "{}: {}",
                p.original_name,
                p.conflict_reason.as_deref().unwrap_or("Conflicting output")
            )
        })
        .collect();
    if !conflicts.is_empty() {
        return Err(format!("CONFLICTS_DETECTED: {}", conflicts.join("; ")));
    }

    let plan = preview_service::plan_renames(&files, &previews)?;

    let job_id = uuid::Uuid::new_v4().to_string();
    let start_time = Instant::now();
    let file_count = files.len() as u32;
    db::create_job(
        conn,
        &job_id,
        "rename",
        file_count,
        &format!("Batch rename: {} files", file_count),
    )
    .map_err(|e| format!("DB_ERROR: {}", e))?;

    let job_file_ids: Vec<String> = files
        .iter()
        .map(|file| format!("{}:{}", job_id, file.id))
        .collect();

    for ((file, preview), job_file_id) in files.iter().zip(previews.iter()).zip(job_file_ids.iter())
    {
        db::add_job_file(
            conn,
            job_file_id,
            &job_id,
            &file.original_path,
            &file.original_name,
            Some(&preview.transformed_name),
            None,
            None,
            None,
            None,
            "pending",
        )
        .map_err(|e| format!("DB_ERROR: {}", e))?;
    }

    let backup_dir = app_data.join("backups").join(&job_id);
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = CANCEL_FLAGS.lock().unwrap();
        flags.insert(job_id.clone(), cancel_flag.clone());
    }

    if let Some(app) = app {
        let _ = app.emit(
            "job_started",
            JobStartedEvent {
                job_id: job_id.clone(),
            },
        );
    }

    Ok(PreparedRename {
        job_id,
        files,
        plan,
        job_file_ids,
        backup_dir,
        cancel_flag,
        start_time,
    })
}

pub fn run_prepared_rename_locked(
    db: &Mutex<Option<rusqlite::Connection>>,
    app: &AppHandle,
    registry: &Mutex<HashMap<String, FileInfo>>,
    prepared: PreparedRename,
) -> Result<(), String> {
    run_prepared_rename(DbAccess::Locked(db), Some(app), Some(registry), prepared, None)
}

pub fn run_prepared_rename_direct(
    conn: &rusqlite::Connection,
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
    prepared: PreparedRename,
) -> Result<(), String> {
    run_prepared_rename(DbAccess::Direct(conn), None, registry, prepared, None)
}

#[cfg(test)]
pub fn run_prepared_rename_direct_with_after_hop(
    conn: &rusqlite::Connection,
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
    prepared: PreparedRename,
    mut after_hop: impl FnMut(&RenameStep),
) -> Result<(), String> {
    run_prepared_rename(
        DbAccess::Direct(conn),
        None,
        registry,
        prepared,
        Some(&mut after_hop),
    )
}

fn run_prepared_rename(
    db: DbAccess<'_>,
    app: Option<&AppHandle>,
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
    prepared: PreparedRename,
    mut after_hop: Option<&mut dyn FnMut(&RenameStep)>,
) -> Result<(), String> {
    let PreparedRename {
        job_id,
        files,
        plan,
        job_file_ids,
        backup_dir,
        cancel_flag,
        start_time,
    } = prepared;

    let files_total = files.len() as u32;
    let mut last_step_for_file: HashMap<String, usize> = HashMap::new();
    for (i, step) in plan.iter().enumerate() {
        last_step_for_file.insert(step.file_id.clone(), i);
    }

    let mut backups: HashMap<String, String> = HashMap::new();
    let mut terminal: HashSet<String> = HashSet::new();
    let mut current_paths: HashMap<String, String> = HashMap::new();
    let mut completed = 0u32;
    let mut failed = 0u32;
    let mut processed = 0u32;

    if cancel_flag.load(Ordering::Relaxed) {
        for (idx, file) in files.iter().enumerate() {
            processed += 1;
            failed += 1;
            let error = "CANCELLED: Job was cancelled".to_string();
            emit_progress(
                app,
                &job_id,
                &file.id,
                &file.original_name,
                "failed",
                0.0,
                Some(&error),
                processed,
                files_total,
                None,
            );
            db.with(|conn| {
                db::update_job_file_result(
                    conn,
                    &job_file_ids[idx],
                    "skipped",
                    None,
                    None,
                    Some(&error),
                )
                .map_err(|e| format!("DB_ERROR: {}", e))
            })?;
        }
        db.with(|conn| {
            db::update_job_status(conn, &job_id, "failed").map_err(|e| format!("DB_ERROR: {}", e))
        })?;
        {
            let mut flags = CANCEL_FLAGS.lock().unwrap();
            flags.remove(&job_id);
        }
        emit_complete(
            app,
            &job_id,
            "failed",
            0,
            failed,
            start_time.elapsed().as_millis() as u64,
        );
        return Ok(());
    }

    // Identity renames (not in the plan) complete immediately without touching disk.
    let planned_ids: HashSet<&str> = plan.iter().map(|s| s.file_id.as_str()).collect();
    for (idx, file) in files.iter().enumerate() {
        if planned_ids.contains(file.id.as_str()) {
            continue;
        }
        processed += 1;
        completed += 1;
        terminal.insert(file.id.clone());
        emit_progress(
            app,
            &job_id,
            &file.id,
            &file.original_name,
            "completed",
            100.0,
            None,
            processed,
            files_total,
            Some(&file.original_path),
        );
        let job_file_id = job_file_ids[idx].clone();
        let path = file.original_path.clone();
        db.with(|conn| {
            db::update_job_file_result(conn, &job_file_id, "success", Some(&path), None, None)
                .map_err(|e| format!("DB_ERROR: {}", e))
        })?;
    }

    for (step_i, step) in plan.iter().enumerate() {
        if cancel_flag.load(Ordering::Relaxed) {
            break;
        }

        let file = &files[step.file_index];
        let job_file_id = &job_file_ids[step.file_index];
        let is_last = last_step_for_file.get(&step.file_id) == Some(&step_i);

        emit_progress(
            app,
            &job_id,
            &file.id,
            &file.original_name,
            "processing",
            25.0,
            None,
            processed,
            files_total,
            None,
        );

        if step.create_backup {
            match file_service::create_backup(&file.original_path, &backup_dir) {
                Ok(path) => {
                    backups.insert(file.id.clone(), path);
                }
                Err(error) => {
                    processed += 1;
                    failed += 1;
                    terminal.insert(file.id.clone());
                    emit_progress(
                        app,
                        &job_id,
                        &file.id,
                        &file.original_name,
                        "failed",
                        0.0,
                        Some(&error),
                        processed,
                        files_total,
                        None,
                    );
                    db.with(|conn| {
                        db::update_job_file_result(
                            conn,
                            job_file_id,
                            "failed",
                            None,
                            None,
                            Some(&error),
                        )
                        .map_err(|e| format!("DB_ERROR: {}", e))
                    })?;
                    continue;
                }
            }
        }

        if terminal.contains(&step.file_id) {
            continue;
        }

        let new_path_string = step.to.to_string_lossy().to_string();
        if target_exists_and_is_not_source(step.from.to_str().unwrap_or(""), &step.to) {
            let error = "TARGET_EXISTS: Target path already exists".to_string();
            processed += 1;
            failed += 1;
            terminal.insert(file.id.clone());
            emit_progress(
                app,
                &job_id,
                &file.id,
                &file.original_name,
                "failed",
                0.0,
                Some(&error),
                processed,
                files_total,
                None,
            );
            let backup = backups.get(&file.id).cloned();
            db.with(|conn| {
                db::update_job_file_result(
                    conn,
                    job_file_id,
                    "failed",
                    Some(&new_path_string),
                    backup.as_deref(),
                    Some(&error),
                )
                .map_err(|e| format!("DB_ERROR: {}", e))
            })?;
            continue;
        }

        // Cancel is checked before this hop. An in-flight fs::rename is not reversed.
        if let Err(e) = fs::rename(&step.from, &step.to) {
            let error = format!("RENAME_FAILED: {}", e);
            processed += 1;
            failed += 1;
            terminal.insert(file.id.clone());
            emit_progress(
                app,
                &job_id,
                &file.id,
                &file.original_name,
                "failed",
                0.0,
                Some(&error),
                processed,
                files_total,
                None,
            );
            let backup = backups.get(&file.id).cloned();
            db.with(|conn| {
                db::update_job_file_result(
                    conn,
                    job_file_id,
                    "failed",
                    Some(&new_path_string),
                    backup.as_deref(),
                    Some(&error),
                )
                .map_err(|e| format!("DB_ERROR: {}", e))
            })?;
            continue;
        }

        current_paths.insert(file.id.clone(), new_path_string.clone());
        if let Some(hook) = after_hop.as_mut() {
            hook(step);
        }

        if is_last {
            processed += 1;
            completed += 1;
            terminal.insert(file.id.clone());
            emit_progress(
                app,
                &job_id,
                &file.id,
                &file.original_name,
                "completed",
                100.0,
                None,
                processed,
                files_total,
                Some(&new_path_string),
            );
            let backup = backups.get(&file.id).cloned();
            db.with(|conn| {
                db::update_job_file_result(
                    conn,
                    job_file_id,
                    "success",
                    Some(&new_path_string),
                    backup.as_deref(),
                    None,
                )
                .map_err(|e| format!("DB_ERROR: {}", e))
            })?;
            update_registry_path(registry, &file.id, &new_path_string);
        }
    }

    // Remaining files were not started, or were mid-plan when cancel landed.
    // Cancel does not undo hops that already ran. Registry must follow disk:
    // a completed temp hop leaves the file at .brp-tmp-* until a later hop.
    let cancelled = cancel_flag.load(Ordering::Relaxed);
    for (idx, file) in files.iter().enumerate() {
        if terminal.contains(&file.id) {
            continue;
        }
        processed += 1;
        failed += 1;
        let error = if cancelled {
            "CANCELLED: Job was cancelled".to_string()
        } else {
            "CANCELLED: Remaining rename hops were not applied".to_string()
        };
        let current_path = current_paths.get(&file.id).cloned();
        if let Some(path) = current_path.as_deref() {
            update_registry_path(registry, &file.id, path);
        }
        emit_progress(
            app,
            &job_id,
            &file.id,
            &file.original_name,
            "failed",
            0.0,
            Some(&error),
            processed,
            files_total,
            current_path.as_deref(),
        );
        let backup = backups.get(&file.id).cloned();
        let status = if cancelled { "skipped" } else { "failed" };
        db.with(|conn| {
            db::update_job_file_result(
                conn,
                &job_file_ids[idx],
                status,
                current_path.as_deref(),
                backup.as_deref(),
                Some(&error),
            )
            .map_err(|e| format!("DB_ERROR: {}", e))
        })?;
    }

    let job_status = if failed == 0 {
        "completed"
    } else if completed > 0 {
        "partial"
    } else {
        "failed"
    };
    db.with(|conn| {
        db::update_job_status(conn, &job_id, job_status).map_err(|e| format!("DB_ERROR: {}", e))?;
        let file_names: Vec<String> = files.iter().map(|f| f.original_name.clone()).collect();
        let _ = db::insert_search_entry(
            conn,
            &job_id,
            &format!("Batch rename: {} files", files.len()),
            &file_names.join(" "),
        );
        Ok(())
    })?;

    {
        let mut flags = CANCEL_FLAGS.lock().unwrap();
        flags.remove(&job_id);
    }

    emit_complete(
        app,
        &job_id,
        job_status,
        completed,
        failed,
        start_time.elapsed().as_millis() as u64,
    );

    Ok(())
}

fn update_registry_path(
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
    file_id: &str,
    new_path: &str,
) {
    let Some(registry) = registry else {
        return;
    };
    let mut reg = registry.lock().unwrap();
    if let Some(entry) = reg.get_mut(file_id) {
        let new_name = Path::new(new_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&entry.original_name)
            .to_string();
        entry.original_path = new_path.to_string();
        entry.original_name = new_name;
    }
}

fn emit_progress(
    app: Option<&AppHandle>,
    job_id: &str,
    file_id: &str,
    file_name: &str,
    status: &str,
    progress_percent: f32,
    error_message: Option<&str>,
    files_completed: u32,
    files_total: u32,
    transformed_path: Option<&str>,
) {
    if let Some(app) = app {
        let _ = app.emit(
            "job_progress",
            JobProgressEvent {
                job_id: job_id.to_string(),
                file_id: file_id.to_string(),
                file_name: file_name.to_string(),
                status: status.into(),
                progress_percent,
                error_message: error_message.map(ToOwned::to_owned),
                files_completed,
                files_total,
                transformed_path: transformed_path.map(ToOwned::to_owned),
            },
        );
    }
}

fn emit_complete(
    app: Option<&AppHandle>,
    job_id: &str,
    status: &str,
    files_completed: u32,
    files_failed: u32,
    duration_ms: u64,
) {
    if let Some(app) = app {
        let _ = app.emit(
            "job_complete",
            JobCompleteEvent {
                job_id: job_id.to_string(),
                status: status.into(),
                files_completed,
                files_failed,
                duration_ms,
            },
        );
    }
}

fn target_exists_and_is_not_source(original_path: &str, target_path: &Path) -> bool {
    if !target_path.exists() {
        return false;
    }
    !file_service::paths_refer_to_same_file(Path::new(original_path), target_path)
}

pub fn undo_batch(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    job_id: &str,
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
) -> Result<UndoResponse, String> {
    undo_batch_with_emitter(Some(app), conn, job_id, registry)
}

pub fn undo_batch_with_emitter(
    _app: Option<&AppHandle>,
    conn: &rusqlite::Connection,
    job_id: &str,
    registry: Option<&Mutex<HashMap<String, FileInfo>>>,
) -> Result<UndoResponse, String> {
    let status: String = conn
        .query_row("SELECT status FROM jobs WHERE id = ?1", [job_id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("JOB_NOT_FOUND: {}", e))?;

    if status == "rolled_back" {
        return Err("ALREADY_ROLLED_BACK: This job has already been undone".into());
    }

    let records =
        db::get_successful_undo_records(conn, job_id).map_err(|e| format!("DB_ERROR: {}", e))?;
    let mut files_restored = 0u32;
    let mut files_failed = 0u32;
    let mut errors = Vec::new();
    let mut restored_paths: Vec<(String, String)> = Vec::new();

    for record in records {
        let original_path = record.original_path.clone();
        let transformed_path = record.transformed_path.clone();
        match undo_one_record(&record) {
            Ok(()) => {
                files_restored += 1;
                restored_paths.push((transformed_path, original_path));
            }
            Err(error) => {
                files_failed += 1;
                errors.push(FileError {
                    file_id: record.id,
                    error,
                });
            }
        }
    }

    if files_failed == 0 {
        db::mark_rolled_back(conn, job_id).map_err(|e| format!("DB_ERROR: {}", e))?;
        if let Some(registry) = registry {
            // Full undo: drop restored ids so the session cap has no ghosts.
            // Matching is by current (transformed) path because apply updated
            // registry entries to the new location.
            let mut reg = registry.lock().unwrap();
            let drop_keys: Vec<String> = reg
                .iter()
                .filter(|(_, file)| {
                    restored_paths.iter().any(|(transformed, original)| {
                        file.original_path == *transformed || file.original_path == *original
                    })
                })
                .map(|(id, _)| id.clone())
                .collect();
            for id in drop_keys {
                reg.remove(&id);
            }
        }
    } else if let Some(registry) = registry {
        // Partial undo: restored files move back; failed ones stay put.
        let mut reg = registry.lock().unwrap();
        for (transformed, original) in &restored_paths {
            for entry in reg.values_mut() {
                if entry.original_path == *transformed {
                    entry.original_path = original.clone();
                    entry.original_name = Path::new(original)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&entry.original_name)
                        .to_string();
                }
            }
        }
    }

    Ok(UndoResponse {
        success: files_failed == 0,
        files_restored,
        files_failed,
        errors,
    })
}

fn undo_one_record(record: &db::UndoFileRecord) -> Result<(), String> {
    let original = PathBuf::from(&record.original_path);
    let transformed = PathBuf::from(&record.transformed_path);
    let backup = PathBuf::from(&record.backup_path);

    if !backup.exists() {
        return Err("BACKUP_MISSING: Backup file no longer exists".into());
    }

    if original.exists()
        && !files_match(&original, &backup).map_err(|e| format!("RESTORE_CHECK_FAILED: {}", e))?
    {
        return Err("ORIGINAL_EXISTS: Refusing to overwrite a file at the original path".into());
    }

    let same_path =
        file_service::paths_refer_to_same_file(&original, &transformed) || original == transformed;
    if transformed.exists() && !same_path {
        if !files_match(&transformed, &backup).map_err(|e| format!("OUTPUT_CHECK_FAILED: {}", e))? {
            return Err("OUTPUT_CHANGED: Refusing to remove a changed renamed file".into());
        }
        fs::remove_file(&transformed).map_err(|e| format!("REMOVE_RENAMED_FAILED: {}", e))?;
    }

    if !original.exists() {
        file_service::restore_from_backup(&record.backup_path, &record.original_path)?;
    }

    Ok(())
}

fn files_match(left: &Path, right: &Path) -> std::io::Result<bool> {
    let left_meta = fs::metadata(left)?;
    let right_meta = fs::metadata(right)?;
    if left_meta.len() != right_meta.len() {
        return Ok(false);
    }

    let mut left_reader = BufReader::new(fs::File::open(left)?);
    let mut right_reader = BufReader::new(fs::File::open(right)?);
    let mut left_buf = [0u8; 8192];
    let mut right_buf = [0u8; 8192];

    loop {
        let left_read = left_reader.read(&mut left_buf)?;
        let right_read = right_reader.read(&mut right_buf)?;
        if left_read != right_read {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
        if left_buf[..left_read] != right_buf[..right_read] {
            return Ok(false);
        }
    }
}

pub fn cancel_job(job_id: &str) -> Result<bool, String> {
    let flags = CANCEL_FLAGS.lock().unwrap();
    if let Some(flag) = flags.get(job_id) {
        flag.store(true, Ordering::Relaxed);
        Ok(true)
    } else {
        Err("JOB_NOT_FOUND: No active job with this ID".into())
    }
}
