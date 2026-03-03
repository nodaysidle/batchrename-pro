use crate::conversion_service;
use crate::file_service;
use crate::metadata_service;
use crate::pipeline;
use crate::preview_service;
use crate::types::*;
use crate::AppDatabase;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn add_files(
    paths: Vec<String>,
    db_state: State<'_, AppDatabase>,
) -> Result<AddFilesResponse, String> {
    let hard_cap = {
        let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;
        db.get_settings().map(|s| s.file_hard_cap).unwrap_or(5000)
    };

    let files = file_service::validate_and_create_file_info(&paths, hard_cap)?;
    Ok(AddFilesResponse { files })
}

#[tauri::command]
pub async fn preview_rename(
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<PreviewResponse, String> {
    let previews = preview_service::generate_previews(&files, &pattern)?;
    let total_conflicts = previews.iter().filter(|p| p.has_conflict).count();
    Ok(PreviewResponse {
        previews,
        total_conflicts,
    })
}

#[tauri::command]
pub async fn get_settings(
    db_state: State<'_, AppDatabase>,
) -> Result<SettingsResponse, String> {
    let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;
    let settings = db.get_settings()?;
    Ok(SettingsResponse { settings })
}

#[tauri::command]
pub async fn update_settings(
    settings: serde_json::Value,
    db_state: State<'_, AppDatabase>,
) -> Result<(), String> {
    let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;

    if let Some(obj) = settings.as_object() {
        for (key, value) in obj {
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Null => continue,
                other => other.to_string(),
            };
            db.update_setting(key, &val_str)?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_job_history(
    limit: u32,
    offset: u32,
    search: Option<String>,
    db_state: State<'_, AppDatabase>,
) -> Result<HistoryResponse, String> {
    let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;
    let (jobs, total_count, has_more) = db.get_history(limit, offset, search.as_deref())?;
    Ok(HistoryResponse {
        jobs,
        total_count,
        has_more,
    })
}

#[tauri::command]
pub async fn get_job_detail(
    job_id: String,
    db_state: State<'_, AppDatabase>,
) -> Result<JobDetailResponse, String> {
    let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;
    let (job, files) = db.get_job_detail(&job_id)?;
    Ok(JobDetailResponse {
        job: JobWithFiles {
            id: job.id,
            created_at: job.created_at,
            operation_type: job.operation_type,
            status: job.status,
            file_count: job.file_count,
            description: job.description,
            can_undo: job.can_undo,
            files,
        },
    })
}

#[tauri::command]
pub async fn apply_rename(
    app_handle: tauri::AppHandle,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
    db_state: State<'_, AppDatabase>,
) -> Result<JobStartResponse, String> {
    let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;
    pipeline::execute_batch_rename(&app_handle, &files, &pattern, &db)
}

#[tauri::command]
pub async fn undo_job(
    job_id: String,
    db_state: State<'_, AppDatabase>,
) -> Result<UndoResponse, String> {
    let db = db_state.0.lock().map_err(|e| format!("DB_ERROR: {}", e))?;
    pipeline::undo_job(&db, &job_id)
}

#[tauri::command]
pub async fn convert_files(
    files: Vec<FileInfo>,
    options: ConvertOptions,
) -> Result<JobStartResponse, String> {
    // Validate conversion is supported
    for file in &files {
        if !conversion_service::validate_conversion(&file.extension, &options.target_format) {
            return Err(format!(
                "UNSUPPORTED_CONVERSION: Cannot convert {} to {}",
                file.extension, options.target_format
            ));
        }
    }
    // For MVP, image conversion works; audio/video return errors
    Err("UNSUPPORTED_CONVERSION: Batch conversion pipeline not yet wired".to_string())
}

#[tauri::command]
pub async fn read_metadata(
    files: Vec<FileInfo>,
) -> Result<serde_json::Value, String> {
    let mut results = Vec::new();

    for file in &files {
        let path = std::path::Path::new(&file.original_path);
        let mut entry = serde_json::json!({
            "file_id": file.id,
            "file_type": file.file_type,
            "tags": {},
            "has_exif": false,
            "has_id3": false,
            "raw_fields": [],
        });

        match file.file_type {
            FileType::Audio => {
                if let Ok(tags) = metadata_service::read_id3_tags(path) {
                    entry["has_id3"] = serde_json::Value::Bool(true);
                    entry["tags"] = serde_json::to_value(&tags).unwrap_or_default();
                    let fields: Vec<serde_json::Value> = tags
                        .iter()
                        .map(|(k, v)| {
                            serde_json::json!({
                                "key": k,
                                "value": v,
                                "editable": true,
                            })
                        })
                        .collect();
                    entry["raw_fields"] = serde_json::Value::Array(fields);
                }
            }
            FileType::Image => {
                if let Ok(tags) = metadata_service::read_exif(path) {
                    entry["has_exif"] = serde_json::Value::Bool(true);
                    entry["tags"] = serde_json::to_value(&tags).unwrap_or_default();
                    let fields: Vec<serde_json::Value> = tags
                        .iter()
                        .map(|(k, v)| {
                            serde_json::json!({
                                "key": k,
                                "value": v,
                                "editable": false,
                            })
                        })
                        .collect();
                    entry["raw_fields"] = serde_json::Value::Array(fields);
                }
            }
            _ => {}
        }

        results.push(entry);
    }

    Ok(serde_json::json!({ "metadata": results }))
}

#[tauri::command]
pub async fn write_metadata(
    files: Vec<FileInfo>,
    changes: MetadataChanges,
) -> Result<JobStartResponse, String> {
    let mut success_count = 0;

    for file in &files {
        let path = std::path::Path::new(&file.original_path);

        if changes.strip_all_id3 && file.file_type == FileType::Audio {
            metadata_service::strip_id3(path)?;
            success_count += 1;
        } else if changes.strip_all_exif && file.file_type == FileType::Image {
            metadata_service::strip_exif(path)?;
            success_count += 1;
        } else if file.file_type == FileType::Audio && !changes.tags.is_empty() {
            metadata_service::write_id3_tags(path, &changes.tags)?;
            success_count += 1;
        }
    }

    Ok(JobStartResponse {
        job_id: uuid::Uuid::new_v4().to_string(),
        status: "completed".to_string(),
        file_count: success_count,
    })
}

#[tauri::command]
pub async fn cancel_job() -> Result<CancelResponse, String> {
    pipeline::cancel_active_job()
}

#[tauri::command]
pub async fn open_file_picker() -> Result<FilePickerResponse, String> {
    let paths = rfd::FileDialog::new()
        .set_title("Select files")
        .add_filter("All Supported", &[
            "mp3", "wav", "flac", "m4a",
            "jpg", "jpeg", "png", "webp", "avif",
            "mp4", "webm", "mkv",
        ])
        .add_filter("Audio", &["mp3", "wav", "flac", "m4a"])
        .add_filter("Images", &["jpg", "jpeg", "png", "webp", "avif"])
        .add_filter("Video", &["mp4", "webm", "mkv"])
        .pick_files();

    match paths {
        Some(paths) => Ok(FilePickerResponse {
            paths: paths.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        }),
        None => Ok(FilePickerResponse { paths: vec![] }),
    }
}
