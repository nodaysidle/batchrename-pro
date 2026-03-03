CREATE TABLE IF NOT EXISTS jobs (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    operation_type TEXT NOT NULL CHECK(operation_type IN ('rename', 'convert', 'metadata')),
    status TEXT NOT NULL CHECK(status IN ('running', 'completed', 'partial', 'failed', 'rolled_back')),
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
    status TEXT NOT NULL CHECK(status IN ('pending', 'processing', 'success', 'failed', 'skipped')),
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

-- FTS5 for searching job history
CREATE VIRTUAL TABLE IF NOT EXISTS job_search USING fts5(
    job_id,
    description,
    file_names
);

-- Default settings
INSERT OR IGNORE INTO settings (key, value, updated_at) VALUES
    ('theme', 'dark', datetime('now')),
    ('accent_color', 'blue', datetime('now')),
    ('max_parallel_jobs', '0', datetime('now')),
    ('auto_backup', 'true', datetime('now')),
    ('backup_retention_days', '30', datetime('now')),
    ('file_hard_cap', '5000', datetime('now'));
