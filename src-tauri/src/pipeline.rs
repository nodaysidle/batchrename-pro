use crate::db::Database;
use crate::preview_service;
use crate::types::*;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

/// Shared cancel flag for active jobs
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

fn backup_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("BACKUP_FAILED: {}", e))?;
    let backup = data_dir.join("backups");
    std::fs::create_dir_all(&backup)
        .map_err(|e| format!("BACKUP_FAILED: Cannot create backup dir: {}", e))?;
    Ok(backup)
}

fn create_backup(original_path: &str, backup_base: &Path, job_id: &str) -> Result<String, String> {
    let src = Path::new(original_path);
    let file_name = src
        .file_name()
        .ok_or_else(|| "BACKUP_FAILED: No filename".to_string())?;

    let job_backup_dir = backup_base.join(job_id);
    std::fs::create_dir_all(&job_backup_dir)
        .map_err(|e| format!("BACKUP_FAILED: {}", e))?;

    let backup_path = job_backup_dir.join(file_name);
    std::fs::copy(src, &backup_path)
        .map_err(|e| format!("BACKUP_FAILED: {}", e))?;

    Ok(backup_path.to_string_lossy().to_string())
}

fn restore_from_backup(backup_path: &str, original_path: &str) -> Result<(), String> {
    std::fs::copy(backup_path, original_path)
        .map_err(|e| format!("RESTORE_FAILED: {}", e))?;
    Ok(())
}

/// Execute a batch rename operation.
pub fn execute_batch_rename(
    app_handle: &AppHandle,
    files: &[FileInfo],
    pattern: &RenamePattern,
    db: &Database,
) -> Result<JobStartResponse, String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);

    let previews = preview_service::generate_previews(files, pattern)?;

    // Check for conflicts
    let conflicts: Vec<_> = previews.iter().filter(|p| p.has_conflict).collect();
    if !conflicts.is_empty() {
        return Err(format!(
            "NAME_CONFLICT: {} files have naming conflicts",
            conflicts.len()
        ));
    }

    let job_id = Uuid::new_v4().to_string();
    let file_count = files.len();
    let description = format!(
        "Renamed {} files using {:?} mode",
        file_count, pattern.mode
    );

    db.create_job(&job_id, "rename", file_count as i64, &description)?;

    let backup_base = backup_dir(app_handle)?;
    let start = Instant::now();
    let completed = Arc::new(Mutex::new(0usize));
    let failed = Arc::new(Mutex::new(0usize));

    // Pair files with their previewed names
    let work_items: Vec<_> = files.iter().zip(previews.iter()).collect();

    // Process files in parallel
    let results: Vec<_> = work_items
        .par_iter()
        .map(|(file, preview)| {
            if CANCEL_FLAG.load(Ordering::SeqCst) {
                return (file, preview, Err("CANCELLED: Job was cancelled".to_string()));
            }

            let result = (|| -> Result<(String, String), String> {
                // Create backup
                let backup_path = create_backup(&file.original_path, &backup_base, &job_id)?;

                // Compute new path
                let parent = Path::new(&file.original_path)
                    .parent()
                    .ok_or("RENAME_FAILED: No parent directory")?;
                let new_path = parent.join(&preview.transformed_name);

                // Execute rename
                std::fs::rename(&file.original_path, &new_path)
                    .map_err(|e| format!("RENAME_FAILED: {}", e))?;

                Ok((backup_path, new_path.to_string_lossy().to_string()))
            })();

            result.as_ref().ok(); // consume for type inference
            (file, preview, result)
        })
        .collect();

    // Record results and emit events
    for (file, preview, result) in &results {
        let file_record_id = Uuid::new_v4().to_string();
        match result {
            Ok((backup_path, new_path)) => {
                *completed.lock().unwrap() += 1;
                let completed_count = *completed.lock().unwrap();

                let record = JobFileRecord {
                    id: file_record_id,
                    job_id: job_id.clone(),
                    original_path: file.original_path.clone(),
                    original_name: file.original_name.clone(),
                    transformed_name: Some(preview.transformed_name.clone()),
                    transformed_path: Some(new_path.clone()),
                    backup_path: Some(backup_path.clone()),
                    format_from: None,
                    format_to: None,
                    status: "success".to_string(),
                    error_message: None,
                };
                let _ = db.add_job_file(&record);

                let _ = app_handle.emit(
                    "job_progress",
                    JobProgressEvent {
                        job_id: job_id.clone(),
                        file_id: file.id.clone(),
                        file_name: file.original_name.clone(),
                        status: "completed".to_string(),
                        progress_percent: (completed_count as f64 / file_count as f64) * 100.0,
                        error_message: None,
                        files_completed: completed_count,
                        files_total: file_count,
                    },
                );
            }
            Err(err) => {
                *failed.lock().unwrap() += 1;
                let completed_count = *completed.lock().unwrap();

                let record = JobFileRecord {
                    id: file_record_id,
                    job_id: job_id.clone(),
                    original_path: file.original_path.clone(),
                    original_name: file.original_name.clone(),
                    transformed_name: Some(preview.transformed_name.clone()),
                    transformed_path: None,
                    backup_path: None,
                    format_from: None,
                    format_to: None,
                    status: "failed".to_string(),
                    error_message: Some(err.clone()),
                };
                let _ = db.add_job_file(&record);

                let _ = app_handle.emit(
                    "job_progress",
                    JobProgressEvent {
                        job_id: job_id.clone(),
                        file_id: file.id.clone(),
                        file_name: file.original_name.clone(),
                        status: "failed".to_string(),
                        progress_percent: (completed_count as f64 / file_count as f64) * 100.0,
                        error_message: Some(err.clone()),
                        files_completed: completed_count,
                        files_total: file_count,
                    },
                );
            }
        }
    }

    let final_completed = *completed.lock().unwrap();
    let final_failed = *failed.lock().unwrap();
    let duration = start.elapsed();

    let job_status = if final_failed == 0 {
        "completed"
    } else if final_completed == 0 {
        "failed"
    } else {
        "partial"
    };

    db.update_job_status(&job_id, job_status)?;

    // Index for FTS5 search
    let file_names: Vec<_> = files.iter().map(|f| f.original_name.as_str()).collect();
    let _ = db.index_job_for_search(&job_id, &description, &file_names.join(", "));

    let _ = app_handle.emit(
        "job_complete",
        JobCompleteEvent {
            job_id: job_id.clone(),
            status: job_status.to_string(),
            files_completed: final_completed,
            files_failed: final_failed,
            duration_ms: duration.as_millis() as u64,
        },
    );

    Ok(JobStartResponse {
        job_id,
        status: "started".to_string(),
        file_count,
    })
}

/// Undo a completed job by restoring files from backups.
pub fn undo_job(db: &Database, job_id: &str) -> Result<UndoResponse, String> {
    let (job, files) = db.get_job_detail(job_id)?;

    if job.status == "rolled_back" {
        return Err("ALREADY_ROLLED_BACK: Job was already undone".to_string());
    }

    let mut files_restored = 0usize;
    let mut files_failed = 0usize;
    let mut errors = Vec::new();

    for file in &files {
        if file.status != "success" {
            continue;
        }

        let backup_path = match &file.backup_path {
            Some(p) => p,
            None => {
                files_failed += 1;
                errors.push(UndoError {
                    file_id: file.id.clone(),
                    error: "No backup path recorded".to_string(),
                });
                continue;
            }
        };

        if !Path::new(backup_path).exists() {
            files_failed += 1;
            errors.push(UndoError {
                file_id: file.id.clone(),
                error: "Backup file no longer exists".to_string(),
            });
            continue;
        }

        match restore_from_backup(backup_path, &file.original_path) {
            Ok(_) => {
                files_restored += 1;
                // If a transformed file exists at the new location, remove it
                if let Some(transformed_path) = &file.transformed_path {
                    if transformed_path != &file.original_path && Path::new(transformed_path).exists() {
                        let _ = std::fs::remove_file(transformed_path);
                    }
                }
            }
            Err(e) => {
                files_failed += 1;
                errors.push(UndoError {
                    file_id: file.id.clone(),
                    error: e,
                });
            }
        }
    }

    db.mark_rolled_back(job_id)?;

    Ok(UndoResponse {
        success: files_failed == 0,
        files_restored,
        files_failed,
        errors,
    })
}

/// Cancel the currently active job.
pub fn cancel_active_job() -> Result<CancelResponse, String> {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(CancelResponse {
        cancelled: true,
        files_completed_before_cancel: 0, // Actual count set by the pipeline
    })
}
