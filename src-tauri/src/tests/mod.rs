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
