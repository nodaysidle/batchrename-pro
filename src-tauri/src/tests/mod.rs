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

    let undo = processing_pipeline::undo_batch_with_emitter(None, &conn, &job_id, None).unwrap();

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
    assert_eq!(db::validate_setting("accent_color", "volt").unwrap(), "volt");
    assert_eq!(db::validate_setting("accent_color", "graphite").unwrap(), "graphite");
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

    let undo = processing_pipeline::undo_batch_with_emitter(None, &conn, "job-1", None).unwrap();

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

fn swapped_previews(files: &[FileInfo]) -> Vec<PreviewPair> {
    assert_eq!(files.len(), 2);
    vec![
        PreviewPair {
            file_id: files[0].id.clone(),
            original_name: files[0].original_name.clone(),
            transformed_name: files[1].original_name.clone(),
            has_conflict: false,
            conflict_reason: None,
        },
        PreviewPair {
            file_id: files[1].id.clone(),
            original_name: files[1].original_name.clone(),
            transformed_name: files[0].original_name.clone(),
            has_conflict: false,
            conflict_reason: None,
        },
    ]
}

#[test]
fn preview_occupancy_chain_is_not_a_conflict() {
    let dir = test_dir("preview-chain");
    let one = dir.join("track1.txt");
    let two = dir.join("track2.txt");
    write_file(&one, "one");
    write_file(&two, "two");

    // Numbering start=2: track1→track2, track2→track3 (A→B while B→C).
    let previews = preview_service::generate_previews(
        &[file_info(&one), file_info(&two)],
        &numbering_pattern(Some("track"), None, 2, 0),
    )
    .unwrap();

    assert!(
        previews.iter().all(|p| !p.has_conflict),
        "chain occupancy should be planned, not blocked: {:?}",
        previews
    );
    assert_eq!(previews[0].transformed_name, "track2.txt");
    assert_eq!(previews[1].transformed_name, "track3.txt");
}

#[test]
fn preview_swap_is_not_a_conflict() {
    let dir = test_dir("preview-swap");
    let left = dir.join("alpha.txt");
    let right = dir.join("beta.txt");
    write_file(&left, "alpha");
    write_file(&right, "beta");

    let files = vec![file_info(&left), file_info(&right)];
    let mut previews = swapped_previews(&files);
    preview_service::mark_occupancy_conflicts(&files, &mut previews);

    assert!(
        previews.iter().all(|p| !p.has_conflict),
        "swap occupancy should be planned, not blocked: {:?}",
        previews
    );
}

#[test]
fn preview_external_occupancy_is_still_a_conflict() {
    let dir = test_dir("preview-external");
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
fn plan_swap_inserts_temp_hop() {
    let dir = test_dir("plan-swap-temp");
    let left = dir.join("alpha.txt");
    let right = dir.join("beta.txt");
    write_file(&left, "alpha");
    write_file(&right, "beta");

    let files = vec![file_info(&left), file_info(&right)];
    let previews = swapped_previews(&files);
    let plan = preview_service::plan_renames(&files, &previews).unwrap();

    assert!(
        plan.iter().any(|step| {
            step.to
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(".brp-tmp-"))
        }),
        "swap plan should hop through a temp path: {:?}",
        plan
    );
}

#[test]
fn apply_occupancy_chain_renames_in_safe_order() {
    let dir = test_dir("apply-chain");
    let one = dir.join("track1.txt");
    let two = dir.join("track2.txt");
    let app_data = dir.join("app-data");
    write_file(&one, "one-content");
    write_file(&two, "two-content");

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    processing_pipeline::execute_batch_rename_with_paths(
        &conn,
        &app_data,
        None,
        None,
        vec![file_info(&one), file_info(&two)],
        numbering_pattern(Some("track"), None, 2, 0),
    )
    .unwrap();

    assert!(!one.exists());
    assert_eq!(fs::read_to_string(dir.join("track2.txt")).unwrap(), "one-content");
    assert_eq!(fs::read_to_string(dir.join("track3.txt")).unwrap(), "two-content");
}

#[test]
fn apply_swap_succeeds_via_temp_hop() {
    let dir = test_dir("apply-swap");
    let left = dir.join("alpha.txt");
    let right = dir.join("beta.txt");
    let app_data = dir.join("app-data");
    write_file(&left, "alpha-content");
    write_file(&right, "beta-content");

    let files = vec![file_info(&left), file_info(&right)];
    let previews = swapped_previews(&files);

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    let prepared = processing_pipeline::prepare_batch_rename_from_previews(
        &conn,
        &app_data,
        None,
        files,
        previews,
    )
    .unwrap();
    processing_pipeline::run_prepared_rename_direct(&conn, None, prepared).unwrap();

    assert_eq!(fs::read_to_string(&left).unwrap(), "beta-content");
    assert_eq!(fs::read_to_string(&right).unwrap(), "alpha-content");
}

#[test]
fn cancel_before_run_skips_remaining_without_renaming() {
    let dir = test_dir("cancel-remaining");
    let one = dir.join("one.txt");
    let two = dir.join("two.txt");
    let app_data = dir.join("app-data");
    write_file(&one, "one");
    write_file(&two, "two");

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    let prepared = processing_pipeline::prepare_batch_rename(
        &conn,
        &app_data,
        None,
        vec![file_info(&one), file_info(&two)],
        regex_pattern("one", "uno"),
    )
    .unwrap();
    let job_id = prepared.job_id.clone();
    processing_pipeline::cancel_job(&job_id).unwrap();
    processing_pipeline::run_prepared_rename_direct(&conn, None, prepared).unwrap();

    assert!(one.exists(), "cancel must not reverse or start skipped files");
    assert!(two.exists());
    assert!(!dir.join("uno.txt").exists());
    let status: String = conn
        .query_row("SELECT status FROM jobs WHERE id = ?1", [&job_id], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(status, "failed");
}

#[test]
fn cancel_after_temp_hop_keeps_registry_aligned_with_disk() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let dir = test_dir("cancel-temp-hop");
    let left = dir.join("alpha.txt");
    let right = dir.join("beta.txt");
    let app_data = dir.join("app-data");
    write_file(&left, "alpha-content");
    write_file(&right, "beta-content");

    let files = vec![file_info(&left), file_info(&right)];
    let previews = swapped_previews(&files);
    let mut map = HashMap::new();
    for file in &files {
        map.insert(file.id.clone(), file.clone());
    }
    let registry = Mutex::new(map);

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    let prepared = processing_pipeline::prepare_batch_rename_from_previews(
        &conn,
        &app_data,
        None,
        files.clone(),
        previews,
    )
    .unwrap();
    let job_id = prepared.job_id.clone();
    processing_pipeline::run_prepared_rename_direct_with_after_hop(
        &conn,
        Some(&registry),
        prepared,
        |step| {
            let is_temp = step
                .to
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(".brp-tmp-"));
            if is_temp {
                processing_pipeline::cancel_job(&job_id).unwrap();
            }
        },
    )
    .unwrap();

    let reg = registry.lock().unwrap();
    assert_eq!(reg.len(), 2);
    for entry in reg.values() {
        assert!(
            Path::new(&entry.original_path).exists(),
            "trusted registry path must exist on disk: {}",
            entry.original_path
        );
    }

    let hopped = reg
        .values()
        .find(|entry| {
            Path::new(&entry.original_path)
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(".brp-tmp-"))
        })
        .expect("cancel after temp hop should leave a file at .brp-tmp-*");
    assert_eq!(fs::read_to_string(&hopped.original_path).unwrap(), "alpha-content");
    assert!(
        !Path::new(&files[0].original_path).exists(),
        "vacated original must not still hold the hopped file"
    );

    let other = reg
        .values()
        .find(|entry| entry.id != hopped.id)
        .expect("swap partner remains in registry");
    assert_eq!(other.original_path, files[1].original_path);
    assert_eq!(fs::read_to_string(&other.original_path).unwrap(), "beta-content");
}

#[test]
fn registry_forget_and_clear_drop_ids_so_hard_cap_counts_live_files_only() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let dir = test_dir("registry-lifecycle");
    let a = dir.join("a.txt");
    let b = dir.join("b.txt");
    let c = dir.join("c.txt");
    write_file(&a, "a");
    write_file(&b, "b");
    write_file(&c, "c");

    let file_a = file_info(&a);
    let file_b = file_info(&b);
    let mut map = HashMap::new();
    map.insert(file_a.id.clone(), file_a.clone());
    map.insert(file_b.id.clone(), file_b.clone());
    let registry = Mutex::new(map);

    assert_eq!(registry.lock().unwrap().len(), 2);
    crate::forget_registry_ids(&registry, &[file_a.id.clone()]);
    assert_eq!(registry.lock().unwrap().len(), 1);
    assert!(!registry.lock().unwrap().contains_key(&file_a.id));

    // Cap of 1: live registry size is 1, so another add is rejected.
    let err = file_service::validate_and_build_file_info(c.to_str().unwrap(), 1, registry.lock().unwrap().len() as u32)
        .unwrap_err();
    assert!(err.starts_with("TOO_MANY_FILES"));

    crate::forget_registry_ids(&registry, &[file_b.id.clone()]);
    crate::clear_registry(&registry);
    assert!(registry.lock().unwrap().is_empty());

    // After clear, cap counts zero live files.
    assert!(file_service::validate_and_build_file_info(c.to_str().unwrap(), 1, registry.lock().unwrap().len() as u32).is_ok());
}

#[test]
fn undo_drops_restored_ids_from_trusted_registry() {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let dir = test_dir("undo-registry");
    let source = dir.join("source.txt");
    let app_data = dir.join("app-data");
    write_file(&source, "content");

    let conn = Connection::open_in_memory().unwrap();
    db::run_migrations_for_test(&conn).unwrap();

    let file = file_info(&source);
    let file_id = file.id.clone();
    let mut initial = HashMap::new();
    initial.insert(file_id.clone(), file.clone());
    let registry = Mutex::new(initial);

    let job_id = processing_pipeline::execute_batch_rename_with_paths(
        &conn,
        &app_data,
        None,
        Some(&registry),
        vec![file],
        regex_pattern("source", "renamed"),
    )
    .unwrap();

    assert!(registry.lock().unwrap().contains_key(&file_id));

    let undo = processing_pipeline::undo_batch_with_emitter(None, &conn, &job_id, Some(&registry))
        .unwrap();
    assert!(undo.success);
    assert!(
        !registry.lock().unwrap().contains_key(&file_id),
        "successful undo must drop the restored id so it is not a hard-cap ghost"
    );
}

