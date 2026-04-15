use crate::db;
use crate::file_service;
use crate::preview_service;
use crate::types::*;
use rayon::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};

static CANCEL_FLAGS: once_cell::sync::Lazy<Arc<Mutex<std::collections::HashMap<String, Arc<AtomicBool>>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(Mutex::new(std::collections::HashMap::new())));

pub fn execute_batch_rename(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<String, String> {
    let job_id = uuid::Uuid::new_v4().to_string();
    let start_time = Instant::now();

    // Create job in DB
    let file_count = files.len() as u32;
    db::create_job(conn, &job_id, "rename", file_count, &format!("Batch rename: {} files", file_count))
        .map_err(|e| format!("DB_ERROR: {}", e))?;

    // Create backup dir
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("APP_DATA_ERROR: {}", e))?;
    let backup_dir = app_data.join("backups").join(&job_id);

    // Register cancel flag
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let mut flags = CANCEL_FLAGS.lock().unwrap();
        flags.insert(job_id.clone(), cancel_flag.clone());
    }

    // Generate previews
    let previews = preview_service::generate_previews(&files, &pattern)?;

    // Register all files in DB
    for (file, preview) in files.iter().zip(previews.iter()) {
        db::add_job_file(
            conn,
            &uuid::Uuid::new_v4().to_string(),
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

    // Process files in parallel
    let files_completed = Arc::new(Mutex::new(0u32));
    let files_failed = Arc::new(Mutex::new(0u32));
    let app_handle = app.clone();
    let job_id_clone = job_id.clone();
    let files_total = files.len() as u32;

    let results: Vec<_> = files
        .par_iter()
        .zip(previews.par_iter())
        .map(|(file, preview)| {
            // Check cancel
            if cancel_flag.load(Ordering::Relaxed) {
                return Err("CANCELLED".to_string());
            }

            let file_id = uuid::Uuid::new_v4().to_string();

            // Emit progress: processing
            let _ = app_handle.emit(
                "job_progress",
                JobProgressEvent {
                    job_id: job_id_clone.clone(),
                    file_id: file_id.clone(),
                    file_name: file.original_name.clone(),
                    status: "processing".into(),
                    progress_percent: 50.0,
                    error_message: None,
                    files_completed: *files_completed.lock().unwrap(),
                    files_total,
                },
            );

            // Backup
            let backup_path = match file_service::create_backup(&file.original_path, &backup_dir) {
                Ok(p) => Some(p),
                Err(e) => return Err(e),
            };

            // Rename
            let parent = Path::new(&file.original_path)
                .parent()
                .unwrap_or(Path::new("."));
            let new_path = parent.join(&preview.transformed_name);

            let result = std::fs::rename(&file.original_path, &new_path)
                .map_err(|e| format!("RENAME_FAILED: {}", e));

            match &result {
                Ok(()) => {
                    let mut comp = files_completed.lock().unwrap();
                    *comp += 1;

                    // Emit progress: done
                    let _ = app_handle.emit(
                        "job_progress",
                        JobProgressEvent {
                            job_id: job_id_clone.clone(),
                            file_id: file_id.clone(),
                            file_name: file.original_name.clone(),
                            status: "completed".into(),
                            progress_percent: 100.0,
                            error_message: None,
                            files_completed: *comp,
                            files_total,
                        },
                    );
                }
                Err(e) => {
                    let mut fail = files_failed.lock().unwrap();
                    *fail += 1;
                    let _ = app_handle.emit(
                        "job_progress",
                        JobProgressEvent {
                            job_id: job_id_clone.clone(),
                            file_id: file_id.clone(),
                            file_name: file.original_name.clone(),
                            status: "failed".into(),
                            progress_percent: 0.0,
                            error_message: Some(e.clone()),
                            files_completed: *files_completed.lock().unwrap(),
                            files_total,
                        },
                    );
                }
            }

            Ok((file_id, backup_path, new_path.to_string_lossy().to_string(), result))
        })
        .collect();

    // Update DB records
    for (i, res) in results.iter().enumerate() {
        if let Ok((file_id, backup_path, new_path, rename_result)) = res {
            let db_file_id = uuid::Uuid::new_v4().to_string();
            let status = if rename_result.is_ok() { "success" } else { "failed" };
            let _ = db::add_job_file(
                conn,
                &db_file_id,
                &job_id,
                &files[i].original_path,
                &files[i].original_name,
                Some(&previews[i].transformed_name),
                Some(new_path),
                backup_path.as_deref(),
                None,
                None,
                status,
            );
        }
    }

    let completed = *files_completed.lock().unwrap();
    let failed = *files_failed.lock().unwrap();
    let job_status = if failed == 0 {
        "completed"
    } else if completed > 0 {
        "partial"
    } else {
        "failed"
    };

    db::update_job_status(conn, &job_id, job_status).map_err(|e| format!("DB_ERROR: {}", e))?;

    // Insert FTS entry
    let file_names: Vec<String> = files.iter().map(|f| f.original_name.clone()).collect();
    let _ = db::insert_search_entry(
        conn,
        &job_id,
        &format!("Batch rename: {} files", file_count),
        &file_names.join(" "),
    );

    // Remove cancel flag
    {
        let mut flags = CANCEL_FLAGS.lock().unwrap();
        flags.remove(&job_id);
    }

    // Emit complete
    let duration = start_time.elapsed().as_millis() as u64;
    let _ = app.emit(
        "job_complete",
        JobCompleteEvent {
            job_id: job_id.clone(),
            status: job_status.into(),
            files_completed: completed,
            files_failed: failed,
            duration_ms: duration,
        },
    );

    Ok(job_id)
}

pub fn undo_batch(
    app: &AppHandle,
    conn: &rusqlite::Connection,
    job_id: &str,
) -> Result<UndoResponse, String> {
    // Check if already rolled back
    let status: String = conn
        .query_row("SELECT status FROM jobs WHERE id = ?1", [job_id], |row| row.get(0))
        .map_err(|e| format!("JOB_NOT_FOUND: {}", e))?;

    if status == "rolled_back" {
        return Err("ALREADY_ROLLED_BACK: This job has already been undone".into());
    }

    let backup_paths = db::get_job_backup_paths(conn, job_id)
        .map_err(|e| format!("DB_ERROR: {}", e))?;

    let mut files_restored = 0u32;
    let mut files_failed = 0u32;
    let mut errors = Vec::new();

    for (original_path, backup_path) in &backup_paths {
        if !Path::new(backup_path).exists() {
            files_failed += 1;
            errors.push(FileError {
                file_id: original_path.clone(),
                error: "BACKUP_MISSING: Backup file no longer exists".into(),
            });
            continue;
        }

        match file_service::restore_from_backup(backup_path, original_path) {
            Ok(()) => files_restored += 1,
            Err(e) => {
                files_failed += 1;
                errors.push(FileError {
                    file_id: original_path.clone(),
                    error: e,
                });
            }
        }
    }

    db::mark_rolled_back(conn, job_id).map_err(|e| format!("DB_ERROR: {}", e))?;

    Ok(UndoResponse {
        success: files_failed == 0,
        files_restored,
        files_failed,
        errors,
    })
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
