use rusqlite::{Connection, params};
use std::path::Path;
use crate::types::{JobRecord, JobFileRecord, Settings};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(app_data_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = app_data_dir.join("batchrename.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self { conn })
    }

    pub fn run_migrations(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );"
        )?;

        let current_version: i64 = self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migrations",
                [],
                |row| row.get(0),
            )?;

        let migrations: Vec<(i64, &str)> = vec![
            (1, include_str!("migrations/001_initial.sql")),
        ];

        for (version, sql) in migrations {
            if version > current_version {
                self.conn.execute_batch(sql)?;
                self.conn.execute(
                    "INSERT INTO migrations (version, applied_at) VALUES (?1, datetime('now'))",
                    params![version],
                )?;
                tracing::info!("Applied migration v{}", version);
            }
        }

        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    // ── Job CRUD ────────────────────────────────────────────

    pub fn create_job(
        &self,
        id: &str,
        operation_type: &str,
        file_count: i64,
        description: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO jobs (id, created_at, operation_type, status, file_count, description)
                 VALUES (?1, datetime('now'), ?2, 'running', ?3, ?4)",
                params![id, operation_type, file_count, description],
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(())
    }

    pub fn add_job_file(&self, file: &JobFileRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO job_files (id, job_id, original_path, original_name, transformed_name, transformed_path, backup_path, format_from, format_to, status, error_message, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, datetime('now'))",
                params![
                    file.id, file.job_id, file.original_path, file.original_name,
                    file.transformed_name, file.transformed_path, file.backup_path,
                    file.format_from, file.format_to, file.status, file.error_message,
                ],
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(())
    }

    pub fn update_job_status(&self, job_id: &str, status: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE jobs SET status = ?1 WHERE id = ?2",
                params![status, job_id],
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(())
    }

    pub fn update_file_status(
        &self,
        file_id: &str,
        status: &str,
        error: Option<&str>,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE job_files SET status = ?1, error_message = ?2 WHERE id = ?3",
                params![status, error, file_id],
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(())
    }

    // ── History queries ─────────────────────────────────────

    pub fn get_history(
        &self,
        limit: u32,
        offset: u32,
        search: Option<&str>,
    ) -> Result<(Vec<JobRecord>, i64, bool), String> {
        let limit = limit.min(200);

        if let Some(query) = search {
            return self.search_history(query, limit, offset);
        }

        let total: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))
            .map_err(|e| format!("DB_ERROR: {}", e))?;

        let mut stmt = self.conn
            .prepare(
                "SELECT id, created_at, operation_type, status, file_count, description
                 FROM jobs ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;

        let jobs: Vec<JobRecord> = stmt
            .query_map(params![limit, offset], |row| {
                let status: String = row.get(3)?;
                Ok(JobRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    operation_type: row.get(2)?,
                    status: row.get(3)?,
                    file_count: row.get(4)?,
                    description: row.get(5)?,
                    can_undo: status != "rolled_back" && status != "failed",
                })
            })
            .map_err(|e| format!("DB_ERROR: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let has_more = (offset as i64 + jobs.len() as i64) < total;
        Ok((jobs, total, has_more))
    }

    fn search_history(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<JobRecord>, i64, bool), String> {
        // Sanitize FTS5 special chars
        let sanitized = query
            .replace('"', "\"\"")
            .replace('*', "")
            .replace(':', "");
        let fts_query = format!("\"{}\"", sanitized);

        let total: i64 = self.conn
            .query_row(
                "SELECT COUNT(*) FROM job_search WHERE job_search MATCH ?1",
                params![fts_query],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let mut stmt = self.conn
            .prepare(
                "SELECT j.id, j.created_at, j.operation_type, j.status, j.file_count, j.description
                 FROM jobs j
                 INNER JOIN job_search js ON j.id = js.job_id
                 WHERE job_search MATCH ?1
                 ORDER BY j.created_at DESC LIMIT ?2 OFFSET ?3"
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;

        let jobs: Vec<JobRecord> = stmt
            .query_map(params![fts_query, limit, offset], |row| {
                let status: String = row.get(3)?;
                Ok(JobRecord {
                    id: row.get(0)?,
                    created_at: row.get(1)?,
                    operation_type: row.get(2)?,
                    status: row.get(3)?,
                    file_count: row.get(4)?,
                    description: row.get(5)?,
                    can_undo: status != "rolled_back" && status != "failed",
                })
            })
            .map_err(|e| format!("DB_ERROR: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        let has_more = (offset as i64 + jobs.len() as i64) < total;
        Ok((jobs, total, has_more))
    }

    pub fn get_job_detail(&self, job_id: &str) -> Result<(JobRecord, Vec<JobFileRecord>), String> {
        let job = self.conn
            .query_row(
                "SELECT id, created_at, operation_type, status, file_count, description
                 FROM jobs WHERE id = ?1",
                params![job_id],
                |row| {
                    let status: String = row.get(3)?;
                    Ok(JobRecord {
                        id: row.get(0)?,
                        created_at: row.get(1)?,
                        operation_type: row.get(2)?,
                        status: row.get(3)?,
                        file_count: row.get(4)?,
                        description: row.get(5)?,
                        can_undo: status != "rolled_back" && status != "failed",
                    })
                },
            )
            .map_err(|_| "JOB_NOT_FOUND: No job exists with this ID".to_string())?;

        let mut stmt = self.conn
            .prepare(
                "SELECT id, job_id, original_path, original_name, transformed_name,
                        transformed_path, backup_path, format_from, format_to, status, error_message
                 FROM job_files WHERE job_id = ?1"
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;

        let files: Vec<JobFileRecord> = stmt
            .query_map(params![job_id], |row| {
                Ok(JobFileRecord {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    original_path: row.get(2)?,
                    original_name: row.get(3)?,
                    transformed_name: row.get(4)?,
                    transformed_path: row.get(5)?,
                    backup_path: row.get(6)?,
                    format_from: row.get(7)?,
                    format_to: row.get(8)?,
                    status: row.get(9)?,
                    error_message: row.get(10)?,
                })
            })
            .map_err(|e| format!("DB_ERROR: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok((job, files))
    }

    pub fn mark_rolled_back(&self, job_id: &str) -> Result<(), String> {
        let current_status: String = self.conn
            .query_row(
                "SELECT status FROM jobs WHERE id = ?1",
                params![job_id],
                |row| row.get(0),
            )
            .map_err(|_| "JOB_NOT_FOUND: No job exists with this ID".to_string())?;

        if current_status == "rolled_back" {
            return Err("ALREADY_ROLLED_BACK: Job was already undone".to_string());
        }

        self.update_job_status(job_id, "rolled_back")
    }

    pub fn check_backups_exist(&self, job_id: &str) -> Result<bool, String> {
        let count: i64 = self.conn
            .query_row(
                "SELECT COUNT(*) FROM job_files WHERE job_id = ?1 AND backup_path IS NOT NULL",
                params![job_id],
                |row| row.get(0),
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(count > 0)
    }

    /// Insert FTS5 search entry for a completed job
    pub fn index_job_for_search(
        &self,
        job_id: &str,
        description: &str,
        file_names: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO job_search (job_id, description, file_names) VALUES (?1, ?2, ?3)",
                params![job_id, description, file_names],
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(())
    }

    // ── Settings ────────────────────────────────────────────

    pub fn get_settings(&self) -> Result<Settings, String> {
        let get = |key: &str, default: &str| -> String {
            self.conn
                .query_row(
                    "SELECT value FROM settings WHERE key = ?1",
                    params![key],
                    |row| row.get::<_, String>(0),
                )
                .unwrap_or_else(|_| default.to_string())
        };

        let last_rename_pattern = {
            let val = get("last_rename_pattern", "");
            if val.is_empty() {
                None
            } else {
                serde_json::from_str(&val).ok()
            }
        };

        let last_convert_format = {
            let val = get("last_convert_format", "");
            if val.is_empty() { None } else { Some(val) }
        };

        Ok(Settings {
            theme: get("theme", "dark"),
            accent_color: get("accent_color", "blue"),
            default_output_dir: {
                let val = get("default_output_dir", "");
                if val.is_empty() { None } else { Some(val) }
            },
            max_parallel_jobs: get("max_parallel_jobs", "0").parse().unwrap_or(0),
            auto_backup: get("auto_backup", "true") == "true",
            backup_retention_days: get("backup_retention_days", "30").parse().unwrap_or(30),
            last_rename_pattern,
            last_convert_format,
            file_hard_cap: get("file_hard_cap", "5000").parse().unwrap_or(5000),
        })
    }

    pub fn update_setting(&self, key: &str, value: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))
                 ON CONFLICT(key) DO UPDATE SET value = ?2, updated_at = datetime('now')",
                params![key, value],
            )
            .map_err(|e| format!("DB_ERROR: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn test_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        let db = Database { conn };
        db.run_migrations().unwrap();
        db
    }

    #[test]
    fn test_create_and_query_job() {
        let db = test_db();
        db.create_job("job-1", "rename", 5, "Renamed 5 files").unwrap();

        let (jobs, total, _) = db.get_history(50, 0, None).unwrap();
        assert_eq!(total, 1);
        assert_eq!(jobs[0].id, "job-1");
        assert_eq!(jobs[0].operation_type, "rename");
        assert_eq!(jobs[0].file_count, 5);
    }

    #[test]
    fn test_job_file_crud() {
        let db = test_db();
        db.create_job("job-2", "rename", 1, "Test job").unwrap();

        let file = JobFileRecord {
            id: "file-1".to_string(),
            job_id: "job-2".to_string(),
            original_path: "/tmp/test.txt".to_string(),
            original_name: "test.txt".to_string(),
            transformed_name: Some("test_renamed.txt".to_string()),
            transformed_path: Some("/tmp/test_renamed.txt".to_string()),
            backup_path: Some("/backups/test.txt".to_string()),
            format_from: None,
            format_to: None,
            status: "success".to_string(),
            error_message: None,
        };
        db.add_job_file(&file).unwrap();

        let (job, files) = db.get_job_detail("job-2").unwrap();
        assert_eq!(job.id, "job-2");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].original_name, "test.txt");
    }

    #[test]
    fn test_rollback_status() {
        let db = test_db();
        db.create_job("job-3", "rename", 1, "Will rollback").unwrap();
        db.update_job_status("job-3", "completed").unwrap();
        db.mark_rolled_back("job-3").unwrap();

        let (job, _) = db.get_job_detail("job-3").unwrap();
        assert_eq!(job.status, "rolled_back");

        // Double rollback should fail
        let result = db.mark_rolled_back("job-3");
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_crud() {
        let db = test_db();
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.backup_retention_days, 30);

        db.update_setting("theme", "light").unwrap();
        let settings = db.get_settings().unwrap();
        assert_eq!(settings.theme, "light");
    }

    #[test]
    fn test_job_not_found() {
        let db = test_db();
        let result = db.get_job_detail("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JOB_NOT_FOUND"));
    }

    #[test]
    fn test_pagination() {
        let db = test_db();
        for i in 0..10 {
            db.create_job(&format!("job-{}", i), "rename", 1, &format!("Job {}", i)).unwrap();
            db.update_job_status(&format!("job-{}", i), "completed").unwrap();
        }

        let (jobs, total, has_more) = db.get_history(3, 0, None).unwrap();
        assert_eq!(total, 10);
        assert_eq!(jobs.len(), 3);
        assert!(has_more);

        let (jobs, _, has_more) = db.get_history(3, 9, None).unwrap();
        assert_eq!(jobs.len(), 1);
        assert!(!has_more);
    }
}
