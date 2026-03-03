mod commands;
mod conversion_service;
mod db;
mod file_service;
mod metadata_service;
mod pipeline;
mod preview_service;
mod types;

use db::Database;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppDatabase(pub Mutex<Database>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter("batchrename_pro=debug,info")
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_data_dir)?;

            let database = Database::new(&app_data_dir)?;
            database.run_migrations()?;

            app.manage(AppDatabase(Mutex::new(database)));

            tracing::info!("BatchRename Pro initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::add_files,
            commands::preview_rename,
            commands::apply_rename,
            commands::convert_files,
            commands::read_metadata,
            commands::write_metadata,
            commands::undo_job,
            commands::cancel_job,
            commands::get_settings,
            commands::update_settings,
            commands::get_job_history,
            commands::get_job_detail,
            commands::open_file_picker,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
