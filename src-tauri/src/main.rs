mod db;
mod file_service;
mod preview_service;
mod processing_pipeline;
#[cfg(test)]
mod tests;
mod types;

use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;
use types::*;

struct AppState {
    db: Mutex<Option<Connection>>,
    /// Server-trusted files added via `add_files`, keyed by file id. Preview/apply
    /// commands resolve paths from this registry instead of trusting client-supplied
    /// `FileInfo.original_path` values, since the webview is not a trusted boundary.
    file_registry: Mutex<HashMap<String, FileInfo>>,
}

/// Look up trusted `FileInfo` entries by id and re-canonicalize their paths,
/// rejecting ids that are unknown or whose backing file no longer exists.
fn lookup_trusted_files(
    state: &tauri::State<'_, AppState>,
    file_ids: &[String],
) -> Result<Vec<FileInfo>, String> {
    let registry = state.file_registry.lock().unwrap();
    let mut trusted = Vec::with_capacity(file_ids.len());
    for id in file_ids {
        let mut file = registry
            .get(id)
            .cloned()
            .ok_or_else(|| format!("UNKNOWN_FILE: {}", id))?;
        file.original_path = file_service::revalidate_path(&file.original_path)?;
        trusted.push(file);
    }
    Ok(trusted)
}

pub(crate) fn forget_registry_ids(
    registry: &Mutex<HashMap<String, FileInfo>>,
    ids: &[String],
) {
    let mut reg = registry.lock().unwrap();
    for id in ids {
        reg.remove(id);
    }
}

pub(crate) fn clear_registry(registry: &Mutex<HashMap<String, FileInfo>>) {
    registry.lock().unwrap().clear();
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
    // Seed from the registry's live size so the hard cap is enforced across the
    // whole session, not just within a single `add_files` invocation.
    let mut current_count = state.file_registry.lock().unwrap().len() as u32;

    for path in &paths {
        match file_service::validate_and_build_file_info(path, hard_cap, current_count) {
            Ok(file_info) => {
                current_count += 1;
                state
                    .file_registry
                    .lock()
                    .unwrap()
                    .insert(file_info.id.clone(), file_info.clone());
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
async fn forget_files(
    file_ids: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<u32, String> {
    forget_registry_ids(&state.file_registry, &file_ids);
    Ok(state.file_registry.lock().unwrap().len() as u32)
}

#[tauri::command]
async fn clear_files(state: tauri::State<'_, AppState>) -> Result<u32, String> {
    clear_registry(&state.file_registry);
    Ok(0)
}

#[tauri::command]
async fn preview_rename(
    file_ids: Vec<String>,
    files: Vec<FileInfo>,
    pattern: RenamePattern,
    state: tauri::State<'_, AppState>,
) -> Result<PreviewResponse, String> {
    // Client-supplied `files` is accepted only for API compatibility; the
    // webview is not a trusted boundary, so paths are resolved server-side.
    let _ = files;
    let trusted = lookup_trusted_files(&state, &file_ids)?;

    let previews = preview_service::generate_previews(&trusted, &pattern)?;
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
    // Client-supplied `files` is accepted only for API compatibility; the
    // webview is not a trusted boundary, so paths are resolved server-side.
    let _ = files;
    let trusted = lookup_trusted_files(&state, &file_ids)?;
    let file_count = trusted.len();
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("APP_DATA_ERROR: {}", e))?;

    let prepared = {
        let db = state.db.lock().unwrap();
        let conn = db.as_ref().ok_or("DB_NOT_INIT")?;
        processing_pipeline::prepare_batch_rename(conn, &app_data, Some(&app), trusted, pattern)?
    };

    let job_id = prepared.job_id.clone();
    let worker = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = worker.state::<AppState>();
        if let Err(error) = processing_pipeline::run_prepared_rename_locked(
            &state.db,
            &worker,
            &state.file_registry,
            prepared,
        ) {
            tracing::error!("rename job failed: {error}");
        }
    });

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
    processing_pipeline::undo_batch(&app, conn, &job_id, Some(&state.file_registry))
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
        let validated = db::validate_setting(key, value)?;
        db::set_setting(conn, key, &validated).map_err(|e| format!("DB_ERROR: {}", e))?;
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
        .manage(AppState {
            db: Mutex::new(None),
            file_registry: Mutex::new(HashMap::new()),
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
            forget_files,
            clear_files,
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
