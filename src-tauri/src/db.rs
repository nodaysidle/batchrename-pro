use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use crate::types::{JobSummary, HistoryResponse, Settings};

pub fn init_db(app_data_dir: &Path) -> SqlResult<Connection> {
    let db_path = app_data_dir.join("batchrename.db");
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    let current_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                operation_type TEXT NOT NULL CHECK(operation_type IN ('rename','convert','metadata')),
                status TEXT NOT NULL CHECK(status IN ('running','completed','partial','failed','rolled_back')),
                file_count INTEGER NOT NULL,
                description TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS job_files (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL REFERENCES jobs(id),
                original_path TEXT NOT NULL,
                original_name TEXT NOT NULL,
                transformed_name TEXT,
                transformed_path TEXT,
                backup_path TEXT,
                format_from TEXT,
                format_to TEXT,
                status TEXT NOT NULL CHECK(status IN ('pending','processing','success','failed','skipped')),
                error_message TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_job_files_job_id ON job_files(job_id);
            CREATE INDEX IF NOT EXISTS idx_jobs_created_at ON jobs(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
            CREATE VIRTUAL TABLE IF NOT EXISTS job_search USING fts5(
                job_id, description, file_names
            );",
        )?;
        conn.execute(
            "INSERT INTO migrations (version, applied_at) VALUES (1, datetime('now'))",
            [],
        )?;
    }

    Ok(())
}

// --- Jobs CRUD ---

pub fn create_job(
    conn: &Connection,
    id: &str,
    operation_type: &str,
    file_count: u32,
    description: &str,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO jobs (id, created_at, operation_type, status, file_count, description) VALUES (?1, datetime('now'), ?2, 'running', ?3, ?4)",
        [id, operation_type, &file_count.to_string(), description],
    )?;
    Ok(())
}

pub fn update_job_status(conn: &Connection, job_id: &str, status: &str) -> SqlResult<()> {
    conn.execute(
        "UPDATE jobs SET status = ?1 WHERE id = ?2",
        [status, job_id],
    )?;
    Ok(())
}

pub fn add_job_file(
    conn: &Connection,
    id: &str,
    job_id: &str,
    original_path: &str,
    original_name: &str,
    transformed_name: Option<&str>,
    transformed_path: Option<&str>,
    backup_path: Option<&str>,
    format_from: Option<&str>,
    format_to: Option<&str>,
    status: &str,
) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO job_files (id, job_id, original_path, original_name, transformed_name, transformed_path, backup_path, format_from, format_to, status, error_message, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, datetime('now'))",
        [id, job_id, original_path, original_name,
         transformed_name.unwrap_or(""),
         transformed_path.unwrap_or(""),
         backup_path.unwrap_or(""),
         format_from.unwrap_or(""),
         format_to.unwrap_or(""),
         status],
    )?;
    Ok(())
}

pub fn update_file_status(
    conn: &Connection,
    file_id: &str,
    status: &str,
    error: Option<&str>,
) -> SqlResult<()> {
    conn.execute(
        "UPDATE job_files SET status = ?1, error_message = ?2 WHERE id = ?3",
        [status, error.unwrap_or(""), file_id],
    )?;
    Ok(())
}

pub fn get_history(
    conn: &Connection,
    limit: u32,
    offset: u32,
    search: Option<&str>,
) -> SqlResult<HistoryResponse> {
    let (jobs, total_count) = if let Some(q) = search {
        let like = format!("%{}%", q);
        let total: u32 = conn.query_row(
            "SELECT COUNT(*) FROM jobs WHERE description LIKE ?1 OR id IN (SELECT job_id FROM job_files WHERE original_name LIKE ?1)",
            [&like],
            |row| row.get(0),
        )?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT j.id, j.created_at, j.operation_type, j.status, j.file_count, j.description FROM jobs j LEFT JOIN job_files jf ON j.id = jf.job_id WHERE j.description LIKE ?1 OR jf.original_name LIKE ?1 ORDER BY j.created_at DESC LIMIT ?2 OFFSET ?3"
        )?;
        let rows = stmt.query_map([&like, &limit.to_string(), &offset.to_string()], map_job_summary)?;
        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row?);
        }
        (jobs, total)
    } else {
        let total: u32 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |row| row.get(0))?;
        let mut stmt = conn.prepare(
            "SELECT id, created_at, operation_type, status, file_count, description FROM jobs ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
        )?;
        let rows = stmt.query_map([&limit.to_string(), &offset.to_string()], map_job_summary)?;
        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(row?);
        }
        (jobs, total)
    };

    Ok(HistoryResponse {
        has_more: (offset + limit) < total_count,
        total_count,
        jobs,
    })
}

fn map_job_summary(row: &rusqlite::Row) -> SqlResult<JobSummary> {
    let status: String = row.get(3)?;
    Ok(JobSummary {
        id: row.get(0)?,
        timestamp: row.get(1)?,
        operation_type: row.get(2)?,
        can_undo: status == "completed" || status == "partial",
        status,
        file_count: row.get(4)?,
        description: row.get(5)?,
    })
}

pub fn mark_rolled_back(conn: &Connection, job_id: &str) -> SqlResult<()> {
    conn.execute(
        "UPDATE jobs SET status = 'rolled_back' WHERE id = ?1",
        [job_id],
    )?;
    Ok(())
}

pub fn get_job_backup_paths(conn: &Connection, job_id: &str) -> SqlResult<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT original_path, backup_path FROM job_files WHERE job_id = ?1 AND backup_path IS NOT NULL AND backup_path != ''"
    )?;
    let rows = stmt.query_map([job_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut paths = Vec::new();
    for row in rows {
        paths.push(row?);
    }
    Ok(paths)
}

// --- Settings ---

pub fn get_setting(conn: &Connection, key: &str) -> SqlResult<Option<String>> {
    let result = conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, datetime('now'))",
        [key, value],
    )?;
    Ok(())
}

pub fn get_all_settings(conn: &Connection) -> SqlResult<Settings> {
    let get = |key: &str| -> Option<String> {
        get_setting(conn, key).ok().flatten()
    };

    let mut settings = Settings::default();
    if let Some(v) = get("theme") { settings.theme = v; }
    if let Some(v) = get("accent_color") { settings.accent_color = v; }
    if let Some(v) = get("default_output_dir") { settings.default_output_dir = Some(v); }
    if let Some(v) = get("max_parallel_jobs") { settings.max_parallel_jobs = v.parse().unwrap_or(4); }
    if let Some(v) = get("auto_backup") { settings.auto_backup = v == "true"; }
    if let Some(v) = get("backup_retention_days") { settings.backup_retention_days = v.parse().unwrap_or(30); }
    if let Some(v) = get("file_hard_cap") { settings.file_hard_cap = v.parse().unwrap_or(5000); }
    if let Some(v) = get("last_convert_format") { settings.last_convert_format = Some(v); }
    Ok(settings)
}

pub fn insert_search_entry(conn: &Connection, job_id: &str, description: &str, file_names: &str) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO job_search (job_id, description, file_names) VALUES (?1, ?2, ?3)",
        [job_id, description, file_names],
    )?;
    Ok(())
}
