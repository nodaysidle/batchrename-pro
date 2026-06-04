use crate::db;
use crate::file_service;
use crate::preview_service;
use crate::types::*;
use rayon::prelude::*;
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

#[derive(Debug)]
struct RenameOutcome {
    job_file_id: String,
    transformed_path: Option<String>,
    backup_path: Option<String>,
    status: &'static str,
    error: Option<String>,
}

pub fn execute_batch_rename(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<String, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("APP_DATA_ERROR: {}", e))?;
    execute_batch_rename_with_paths(conn, &app_data, Some(app), files, pattern)
}

pub fn execute_batch_rename_with_paths(
    conn: &rusqlite::Connection,
    app_data: &Path,
    app: Option<&AppHandle>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<String, String> {
    if files.is_empty() {
        return Err("NO_FILES: Add files before applying rename".into());
    }

    let previews = preview_service::generate_previews(&files, &pattern)?;
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

    let files_total = files.len() as u32;
    let processed_count = Arc::new(Mutex::new(0u32));
    let app_handle = app.cloned();

    let outcomes: Vec<RenameOutcome> = files
        .par_iter()
        .zip(previews.par_iter())
        .zip(job_file_ids.par_iter())
        .map(|((file, preview), job_file_id)| {
            if cancel_flag.load(Ordering::Relaxed) {
                return RenameOutcome {
                    job_file_id: job_file_id.clone(),
                    transformed_path: None,
                    backup_path: None,
                    status: "failed",
                    error: Some("CANCELLED: Job was cancelled".into()),
                };
            }

            emit_progress(
                app_handle.as_ref(),
                &job_id,
                &file.id,
                &file.original_name,
                "processing",
                25.0,
                None,
                *processed_count.lock().unwrap(),
                files_total,
            );

            let backup_path = match file_service::create_backup(&file.original_path, &backup_dir) {
                Ok(path) => path,
                Err(error) => {
                    return finish_outcome(
                        app_handle.as_ref(),
                        &job_id,
                        files_total,
                        &processed_count,
                        job_file_id,
                        file,
                        None,
                        None,
                        "failed",
                        Some(error),
                    );
                }
            };

            let parent = Path::new(&file.original_path)
                .parent()
                .unwrap_or(Path::new("."));
            let new_path = parent.join(&preview.transformed_name);
            let new_path_string = new_path.to_string_lossy().to_string();

            if target_exists_and_is_not_source(&file.original_path, &new_path) {
                return finish_outcome(
                    app_handle.as_ref(),
                    &job_id,
                    files_total,
                    &processed_count,
                    job_file_id,
                    file,
                    Some(new_path_string),
                    Some(backup_path),
                    "failed",
                    Some("TARGET_EXISTS: Target path already exists".into()),
                );
            }

            let rename_result = fs::rename(&file.original_path, &new_path)
                .map_err(|e| format!("RENAME_FAILED: {}", e));

            match rename_result {
                Ok(()) => finish_outcome(
                    app_handle.as_ref(),
                    &job_id,
                    files_total,
                    &processed_count,
                    job_file_id,
                    file,
                    Some(new_path_string),
                    Some(backup_path),
                    "success",
                    None,
                ),
                Err(error) => finish_outcome(
                    app_handle.as_ref(),
                    &job_id,
                    files_total,
                    &processed_count,
                    job_file_id,
                    file,
                    Some(new_path_string),
                    Some(backup_path),
                    "failed",
                    Some(error),
                ),
            }
        })
        .collect();

    let mut completed = 0u32;
    let mut failed = 0u32;
    let mut db_recording_errors = Vec::new();
    for outcome in &outcomes {
        if outcome.status == "success" {
            completed += 1;
        } else {
            failed += 1;
        }
        if let Err(error) = db::update_job_file_result(
            conn,
            &outcome.job_file_id,
            outcome.status,
            outcome.transformed_path.as_deref(),
            outcome.backup_path.as_deref(),
            outcome.error.as_deref(),
        ) {
            db_recording_errors.push(format!("{}: {}", outcome.job_file_id, error));
        }
    }

    let job_status = if failed == 0 && db_recording_errors.is_empty() {
        "completed"
    } else if completed > 0 || !db_recording_errors.is_empty() {
        "partial"
    } else {
        "failed"
    };
    db::update_job_status(conn, &job_id, job_status).map_err(|e| format!("DB_ERROR: {}", e))?;

    let file_names: Vec<String> = files.iter().map(|f| f.original_name.clone()).collect();
    let _ = db::insert_search_entry(
        conn,
        &job_id,
        &format!("Batch rename: {} files", file_count),
        &file_names.join(" "),
    );

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

    Ok(job_id)
}

fn finish_outcome(
    app: Option<&AppHandle>,
    job_id: &str,
    files_total: u32,
    processed_count: &Arc<Mutex<u32>>,
    job_file_id: &str,
    file: &FileInfo,
    transformed_path: Option<String>,
    backup_path: Option<String>,
    status: &'static str,
    error: Option<String>,
) -> RenameOutcome {
    let mut processed = processed_count.lock().unwrap();
    *processed += 1;
    let event_status = if status == "success" {
        "completed"
    } else {
        "failed"
    };
    emit_progress(
        app,
        job_id,
        &file.id,
        &file.original_name,
        event_status,
        if status == "success" { 100.0 } else { 0.0 },
        error.as_deref(),
        *processed,
        files_total,
    );
    RenameOutcome {
        job_file_id: job_file_id.to_string(),
        transformed_path,
        backup_path,
        status,
        error,
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
    !paths_refer_to_same_file(Path::new(original_path), target_path)
}

pub fn undo_batch(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    job_id: &str,
) -> Result<UndoResponse, String> {
    undo_batch_with_emitter(Some(app), conn, job_id)
}

pub fn undo_batch_with_emitter(
    _app: Option<&AppHandle>,
    conn: &rusqlite::Connection,
    job_id: &str,
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

    for record in records {
        match undo_one_record(&record) {
            Ok(()) => files_restored += 1,
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

    let same_path = paths_refer_to_same_file(&original, &transformed) || original == transformed;
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

fn paths_refer_to_same_file(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
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
