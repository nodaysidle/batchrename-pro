use crate::db;
use crate::file_service;
use crate::preview_service;
use crate::processing_pipeline;
use crate::types::*;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};

fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "batchrename-pro-test-{}-{}",
        name,
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

fn file_info(path: &Path) -> FileInfo {
    file_service::validate_and_build_file_info(path.to_str().unwrap(), 5000, 0).unwrap()
}

fn regex_pattern(find: &str, replace: &str) -> RenamePattern {
    RenamePattern {
        mode: RenameMode::Regex,
        regex_find: Some(find.into()),
        regex_replace: Some(replace.into()),
        template: None,
        start_number: Some(1),
        zero_pad: Some(0),
        prefix: None,
        suffix: None,
        case_transform: CaseTransform::None,
    }
}

fn template_pattern(template: &str, start: u32, zero_pad: u32) -> RenamePattern {
    RenamePattern {
        mode: RenameMode::Template,
        regex_find: None,
        regex_replace: None,
        template: Some(template.into()),
        start_number: Some(start),
        zero_pad: Some(zero_pad),
        prefix: None,
        suffix: None,
        case_transform: CaseTransform::None,
    }
}

fn numbering_pattern(prefix: Option<&str>, suffix: Option<&str>, start: u32, zero_pad: u32) -> RenamePattern {
    RenamePattern {
        mode: RenameMode::Numbering,
        regex_find: None,
        regex_replace: None,
        template: None,
        start_number: Some(start),
        zero_pad: Some(zero_pad),
        prefix: prefix.map(String::from),
        suffix: suffix.map(String::from),
        case_transform: CaseTransform::None,
    }
}

#[test]
fn preview_regex_replace() {
    let dir = test_dir("preview-regex");
    let source = dir.join("invoice-001.txt");
    write_file(&source, "one");

    let previews = preview_service::generate_previews(
        &[file_info(&source)],
        &regex_pattern("invoice", "receipt"),
    )
    .unwrap();

    assert_eq!(previews[0].transformed_name, "receipt-001.txt");
    assert!(!previews[0].has_conflict);
}

#[test]
fn preview_template_tokens_and_zero_pad_numbering() {
    let dir = test_dir("preview-template");
    let source = dir.join("photo.jpg");
    write_file(&source, "one");

    let previews = preview_service::generate_previews(
        &[file_info(&source)],
        &template_pattern("{original}-{number}-{date}-{ext}", 7, 3),
    )
    .unwrap();

    assert!(previews[0].transformed_name.starts_with("photo-007-"));
    assert!(previews[0].transformed_name.ends_with("-jpg.jpg"));
}

#[test]
fn preview_numbering_does_not_double_prefix() {
    let dir = test_dir("preview-numbering-prefix");
    let source = dir.join("photo.txt");
    write_file(&source, "one");

    let previews = preview_service::generate_previews(
        &[file_info(&source)],
        &numbering_pattern(Some("img"), None, 1, 0),
    )
    .unwrap();

    assert_eq!(previews[0].transformed_name, "img1.txt");
}

#[test]
fn preview_numbering_empty_prefix_defaults_to_file() {
    let dir = test_dir("preview-numbering-empty-prefix");
    let source = dir.join("photo.txt");
    write_file(&source, "one");

    let previews = preview_service::generate_previews(
        &[file_info(&source)],
        &numbering_pattern(Some(""), None, 1, 0),
    )
    .unwrap();

    assert_eq!(previews[0].transformed_name, "file1.txt");
}

#[test]
fn preview_template_mode_still_applies_outer_prefix_once() {
    let dir = test_dir("preview-template-prefix");
    let source = dir.join("photo.jpg");
    write_file(&source, "one");

    let mut pattern = template_pattern("{original}", 1, 0);
    pattern.prefix = Some("pre-".into());

    let previews = preview_service::generate_previews(&[file_info(&source)], &pattern).unwrap();

    assert_eq!(previews[0].transformed_name, "pre-photo.jpg");
}

#[test]
fn preview_flags_duplicate_output_conflict() {
    let dir = test_dir("preview-duplicate");
    let one = dir.join("one.txt");
    let two = dir.join("two.txt");
    write_file(&one, "one");
    write_file(&two, "two");

    let previews = preview_service::generate_previews(
        &[file_info(&one), file_info(&two)],
        &template_pattern("same", 1, 0),
    )
    .unwrap();

    assert!(previews.iter().any(|p| p.has_conflict));
}

#[test]
fn preview_flags_existing_target_conflict() {
    let dir = test_dir("preview-existing");
    let source = dir.join("source.txt");
    let target = dir.join("target.txt");
    write_file(&source, "source");
    write_file(&target, "target");

    let previews = preview_service::generate_previews(
        &[file_info(&source)],
        &regex_pattern("source", "target"),
    )
    .unwrap();

    assert!(previews[0].has_conflict);
    assert_eq!(
        previews[0].conflict_reason.as_deref(),
        Some("Target already exists")
    );
}

#[test]
fn preview_rejects_path_separator() {
    let dir = test_dir("preview-separator");
    let source = dir.join("source.txt");
    write_file(&source, "source");

    let err = preview_service::generate_previews(
        &[file_info(&source)],
        &template_pattern("bad/name", 1, 0),
    )
    .unwrap_err();

    assert!(err.starts_with("INVALID_RESULT"));
}

#[test]
fn file_type_detection_maps_known_extensions() {
    assert_eq!(file_service::detect_file_type("mp3"), FileType::Audio);
    assert_eq!(file_service::detect_file_type("PNG"), FileType::Image);
    assert_eq!(file_service::detect_file_type("mkv"), FileType::Video);
    assert_eq!(file_service::detect_file_type("txt"), FileType::Document);
}

#[test]
fn backup_names_are_unique_for_same_filename_in_different_dirs() {
    let dir = test_dir("backup-unique");
    let source_a = dir.join("a").join("same.txt");
    let source_b = dir.join("b").join("same.txt");
    let backup_dir = dir.join("backups");
    write_file(&source_a, "a");
    write_file(&source_b, "b");

    let backup_a = file_service::create_backup(source_a.to_str().unwrap(), &backup_dir).unwrap();
    let backup_b = file_service::create_backup(source_b.to_str().unwrap(), &backup_dir).unwrap();

    assert_ne!(backup_a, backup_b);
    assert_eq!(fs::read_to_string(backup_a).unwrap(), "a");
    assert_eq!(fs::read_to_string(backup_b).unwrap(), "b");
}

#[test]
fn db_job_file_lifecycle_and_history() {
    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    db::create_job(&conn, "job-1", "rename", 1, "Rename one").unwrap();
    db::add_job_file(
        &conn,
        "file-1",
        "job-1",
        "/tmp/original.txt",
        "original.txt",
        Some("renamed.txt"),
        None,
        None,
        None,
        None,
        "pending",
    )
    .unwrap();
    db::update_job_file_result(
        &conn,
        "file-1",
        "success",
        Some("/tmp/renamed.txt"),
        Some("/tmp/backup.txt"),
        None,
    )
    .unwrap();
    db::update_job_status(&conn, "job-1", "completed").unwrap();

    let history = db::get_history(&conn, 20, 0, None).unwrap();
    assert_eq!(history.total_count, 1);
    assert!(history.jobs[0].can_undo);

    db::mark_rolled_back(&conn, "job-1").unwrap();
    let history = db::get_history(&conn, 20, 0, None).unwrap();
    assert_eq!(history.jobs[0].status, "rolled_back");
    assert!(!history.jobs[0].can_undo);
}

#[test]
fn rename_operation_creates_backup_and_undo_removes_output() {
    let dir = test_dir("rename-undo");
    let source = dir.join("source.txt");
    let app_data = dir.join("app-data");
    write_file(&source, "source-content");

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    let job_id = processing_pipeline::execute_batch_rename_with_paths(
        &conn,
        &app_data,
        None,
        None,
        vec![file_info(&source)],
        regex_pattern("source", "renamed"),
    )
    .unwrap();

    let renamed = dir.join("renamed.txt");
    assert!(!source.exists());
    assert_eq!(fs::read_to_string(&renamed).unwrap(), "source-content");
    assert!(!db::get_job_backup_paths(&conn, &job_id).unwrap().is_empty());

    let undo = processing_pipeline::undo_batch_with_emitter(None, &conn, &job_id).unwrap();

    assert!(undo.success);
    assert_eq!(fs::read_to_string(&source).unwrap(), "source-content");
    assert!(!renamed.exists());
    let status: String = conn
        .query_row("SELECT status FROM jobs WHERE id = ?1", [&job_id], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(status, "rolled_back");
}

#[test]
fn revalidate_path_accepts_existing_file_and_rejects_missing() {
    let dir = test_dir("revalidate-path");
    let source = dir.join("source.txt");
    write_file(&source, "content");

    let canonical = file_service::revalidate_path(source.to_str().unwrap()).unwrap();
    assert_eq!(
        Path::new(&canonical).canonicalize().unwrap(),
        source.canonicalize().unwrap()
    );

    let missing = dir.join("does-not-exist.txt");
    let err = file_service::revalidate_path(missing.to_str().unwrap()).unwrap_err();
    assert!(err.starts_with("FILE_NOT_FOUND"));
}

#[test]
fn rename_updates_trusted_registry_with_new_path() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let dir = test_dir("registry-update");
    let source = dir.join("source.txt");
    let app_data = dir.join("app-data");
    write_file(&source, "content");

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    let file = file_info(&source);
    let file_id = file.id.clone();
    let mut initial_registry = HashMap::new();
    initial_registry.insert(file_id.clone(), file.clone());
    let registry: Mutex<HashMap<String, FileInfo>> = Mutex::new(initial_registry);

    processing_pipeline::execute_batch_rename_with_paths(
        &conn,
        &app_data,
        None,
        Some(&registry),
        vec![file],
        regex_pattern("source", "renamed"),
    )
    .unwrap();

    let renamed = dir.join("renamed.txt");
    let updated = registry.lock().unwrap().get(&file_id).cloned().unwrap();
    assert_eq!(
        Path::new(&updated.original_path).canonicalize().unwrap(),
        renamed.canonicalize().unwrap()
    );
    assert_eq!(updated.original_name, "renamed.txt");
}

#[test]
fn validate_setting_rejects_unknown_key() {
    let err = db::validate_setting("not_a_real_setting", "x").unwrap_err();
    assert!(err.starts_with("INVALID_SETTING"));
}

#[test]
fn validate_setting_clamps_file_hard_cap_and_parallel_jobs() {
    assert_eq!(db::validate_setting("file_hard_cap", "999999").unwrap(), "10000");
    assert_eq!(db::validate_setting("file_hard_cap", "0").unwrap(), "1");
    assert_eq!(db::validate_setting("max_parallel_jobs", "999").unwrap(), "16");
    assert_eq!(db::validate_setting("max_parallel_jobs", "0").unwrap(), "1");
}

#[test]
fn validate_and_build_file_info_rejects_when_session_count_already_at_cap() {
    let dir = test_dir("hard-cap-session");
    let path = dir.join("one.txt");
    write_file(&path, "content");

    // Simulates `add_files` seeding `current_count` from a registry that
    // already holds 2 files when the hard cap is 2: the session total must
    // be enforced, not just the count within a single `add_files` call.
    let err =
        file_service::validate_and_build_file_info(path.to_str().unwrap(), 2, 2).unwrap_err();
    assert!(err.starts_with("TOO_MANY_FILES"));

    // Below the cap it still succeeds.
    assert!(file_service::validate_and_build_file_info(path.to_str().unwrap(), 2, 1).is_ok());
}

#[test]
fn validate_setting_rejects_invalid_enum_and_bool_values() {
    assert!(db::validate_setting("theme", "purple").is_err());
    assert!(db::validate_setting("accent_color", "gold").is_err());
    assert!(db::validate_setting("auto_backup", "yes").is_err());
    assert_eq!(db::validate_setting("theme", "light").unwrap(), "light");
    assert_eq!(db::validate_setting("auto_backup", "true").unwrap(), "true");
}

#[test]
fn partial_undo_does_not_mark_rolled_back_or_overwrite_user_file() {
    let dir = test_dir("partial-undo");
    let original = dir.join("original.txt");
    let renamed = dir.join("renamed.txt");
    let backup = dir.join("backup.txt");
    write_file(&original, "user-created");
    write_file(&renamed, "renamed-output");
    write_file(&backup, "original-content");

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();
    db::create_job(&conn, "job-1", "rename", 1, "Rename one").unwrap();
    db::add_job_file(
        &conn,
        "file-1",
        "job-1",
        original.to_str().unwrap(),
        "original.txt",
        Some("renamed.txt"),
        Some(renamed.to_str().unwrap()),
        Some(backup.to_str().unwrap()),
        None,
        None,
        "success",
    )
    .unwrap();
    db::update_job_status(&conn, "job-1", "completed").unwrap();

    let undo = processing_pipeline::undo_batch_with_emitter(None, &conn, "job-1").unwrap();

    assert!(!undo.success);
    assert_eq!(fs::read_to_string(&original).unwrap(), "user-created");
    assert!(renamed.exists());
    let status: String = conn
        .query_row("SELECT status FROM jobs WHERE id = 'job-1'", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(status, "completed");
}
