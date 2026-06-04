mod db;
mod file_service;
mod preview_service;
mod processing_pipeline;
#[cfg(test)]
mod tests;
mod types;

use rusqlite::Connection;
use std::sync::Mutex;
use tauri::Manager;
use types::*;

struct AppState {
    db: Mutex<Option<Connection>>,
}

#[tauri::command]
async fn add_files(
    paths: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<AddFilesResponse, String> {
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().ok_or("DB_NOT_INIT")?;

    let hard_cap: u32 = db::get_setting(conn, "file_hard_cap")
        .ok()
        .flatten()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    let mut files = Vec::new();
    let mut current_count = 0u32;

    for path in &paths {
        match file_service::validate_and_build_file_info(path, hard_cap, current_count) {
            Ok(file_info) => {
                current_count += 1;
                files.push(file_info);
            }
            Err(e) => {
                if e.starts_with("FILE_NOT_FOUND") || e.starts_with("PERMISSION_DENIED") {
                    return Err(e);
                }
                // Skip unsupported types silently
                if e.starts_with("TOO_MANY_FILES") {
                    return Err(e);
                }
            }
        }
    }

    Ok(AddFilesResponse { files })
}

#[tauri::command]
async fn preview_rename(
    file_ids: Vec<String>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
) -> Result<PreviewResponse, String> {
    let filtered: Vec<FileInfo> = files
        .into_iter()
        .filter(|f| file_ids.contains(&f.id))
        .collect();

    let previews = preview_service::generate_previews(&filtered, &pattern)?;
    let conflicts = previews.iter().filter(|p| p.has_conflict).count() as u32;

    Ok(PreviewResponse {
        previews,
        total_conflicts: conflicts,
    })
}

#[tauri::command]
async fn apply_rename(
    app: tauri::AppHandle,
    file_ids: Vec<String>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
    state: tauri::State<'_, AppState>,
) -> Result<JobStartResponse, String> {
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().ok_or("DB_NOT_INIT")?;

    let filtered: Vec<FileInfo> = files
        .into_iter()
        .filter(|f| file_ids.contains(&f.id))
        .collect();

    let file_count = filtered.len();
    let job_id = processing_pipeline::execute_batch_rename(&app, conn, filtered, pattern)?;

    Ok(JobStartResponse {
        job_id,
        status: "started".into(),
        file_count,
    })
}

#[tauri::command]
async fn undo_job(
    app: tauri::AppHandle,
    job_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<UndoResponse, String> {
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().ok_or("DB_NOT_INIT")?;
    processing_pipeline::undo_batch(&app, conn, &job_id)
}

#[tauri::command]
async fn cancel_job(job_id: String) -> Result<bool, String> {
    processing_pipeline::cancel_job(&job_id)
}

#[tauri::command]
async fn get_job_history(
    limit: u32,
    offset: u32,
    search: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<HistoryResponse, String> {
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().ok_or("DB_NOT_INIT")?;
    db::get_history(conn, limit, offset, search.as_deref()).map_err(|e| format!("DB_ERROR: {}", e))
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().ok_or("DB_NOT_INIT")?;
    db::get_all_settings(conn).map_err(|e| format!("DB_ERROR: {}", e))
}

#[tauri::command]
async fn update_settings(
    settings: std::collections::HashMap<String, String>,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let db = state.db.lock().unwrap();
    let conn = db.as_ref().ok_or("DB_NOT_INIT")?;

    for (key, value) in &settings {
        db::set_setting(conn, key, value).map_err(|e| format!("DB_ERROR: {}", e))?;
    }
    Ok(true)
}

#[tauri::command]
async fn open_file_picker() -> Result<Vec<String>, String> {
    // File picker requires frontend-side handling via dialog plugin JS API
    // Return empty — frontend should use @tauri-apps/plugin-dialog directly
    Ok(vec![])
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            db: Mutex::new(None),
        })
        .setup(|app| {
            let app_data = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data)?;

            let conn = db::init_db(&app_data)?;

            // Save default settings if none exist
            let defaults = Settings::default();
            if db::get_setting(&conn, "theme")?.is_none() {
                for (key, value) in [
                    ("theme", defaults.theme.as_str()),
                    ("accent_color", defaults.accent_color.as_str()),
                    ("max_parallel_jobs", &defaults.max_parallel_jobs.to_string()),
                    (
                        "auto_backup",
                        if defaults.auto_backup {
                            "true"
                        } else {
                            "false"
                        },
                    ),
                    (
                        "backup_retention_days",
                        &defaults.backup_retention_days.to_string(),
                    ),
                    ("file_hard_cap", &defaults.file_hard_cap.to_string()),
                ] {
                    db::set_setting(&conn, key, value)?;
                }
            }

            let state: tauri::State<AppState> = app.state();
            *state.db.lock().unwrap() = Some(conn);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_files,
            preview_rename,
            apply_rename,
            undo_job,
            cancel_job,
            get_job_history,
            get_settings,
            update_settings,
            open_file_picker,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running BatchRename Pro");
}
