# Technical Requirements Document

## 🧭 System Context
BatchRename Pro is a cross-platform Tauri 2 desktop app combining batch file renaming, format conversion, and metadata editing. Two-process model: Vite 6 + TypeScript + React 19 frontend in a webview renderer communicating via Tauri IPC commands with a Rust backend. All processing is local-only with no network dependencies. SQLite (rusqlite) persists job history with FTS5 search. Parallel processing via Rayon for CPU-bound tasks, tokio for async I/O. Ships as a single bundle under 10MB for macOS, Windows, and Linux.

## 🔌 API Contracts
### add_files
- **Method:** INVOKE
- **Path:** invoke('add_files', { paths: string[] })
- **Auth:** None (local IPC)
- **Request:** { paths: string[] } — absolute file paths from drag-drop or file picker
- **Response:** { files: Array<{ id: string, original_name: string, original_path: string, extension: string, size_bytes: number, file_type: 'audio' | 'image' | 'video' | 'document', thumbnail_data_url: string | null }> }
- **Errors:** FILE_NOT_FOUND: One or more paths do not exist, UNSUPPORTED_TYPE: File type not supported for any operation, PERMISSION_DENIED: Cannot read file at path, TOO_MANY_FILES: Exceeds configurable hard cap (default 5000)

### preview_rename
- **Method:** INVOKE
- **Path:** invoke('preview_rename', { file_ids: string[], pattern: RenamePattern })
- **Auth:** None (local IPC)
- **Request:** { file_ids: string[], pattern: { mode: 'regex' | 'template' | 'numbering', regex_find?: string, regex_replace?: string, template?: string, start_number?: number, zero_pad?: number, prefix?: string, suffix?: string, case_transform?: 'none' | 'upper' | 'lower' | 'title' } }
- **Response:** { previews: Array<{ file_id: string, original_name: string, transformed_name: string, has_conflict: boolean, conflict_reason: string | null }>, total_conflicts: number }
- **Errors:** INVALID_REGEX: Regex pattern failed to compile, INVALID_TEMPLATE: Template contains unknown placeholders, EMPTY_RESULT: Pattern produces empty filename for one or more files

### apply_rename
- **Method:** INVOKE
- **Path:** invoke('apply_rename', { file_ids: string[], pattern: RenamePattern })
- **Auth:** None (local IPC)
- **Request:** { file_ids: string[], pattern: RenamePattern } — same pattern shape as preview_rename
- **Response:** { job_id: string, status: 'started', file_count: number }
- **Errors:** INVALID_REGEX: Regex pattern failed to compile, NAME_CONFLICT: Transformed names collide and overwrite_policy is 'reject', BACKUP_FAILED: Could not create backup of original files before rename, PARTIAL_FAILURE: Some files renamed, others failed — job_id still created for partial rollback

### convert_files
- **Method:** INVOKE
- **Path:** invoke('convert_files', { file_ids: string[], options: ConvertOptions })
- **Auth:** None (local IPC)
- **Request:** { file_ids: string[], options: { target_format: string, quality?: number (0-100), output_dir?: string, overwrite_existing: boolean, video_codec?: string, audio_bitrate?: string, image_resize?: { width: number, height: number, maintain_aspect: boolean } } }
- **Response:** { job_id: string, status: 'started', file_count: number }
- **Errors:** UNSUPPORTED_CONVERSION: Source format cannot be converted to target format, FFMPEG_ERROR: Underlying ffmpeg operation failed with details, DISK_SPACE: Insufficient disk space for output files, OUTPUT_DIR_INVALID: Specified output directory does not exist or is not writable, BACKUP_FAILED: Could not create backup before conversion

### read_metadata
- **Method:** INVOKE
- **Path:** invoke('read_metadata', { file_ids: string[] })
- **Auth:** None (local IPC)
- **Request:** { file_ids: string[] }
- **Response:** { metadata: Array<{ file_id: string, file_type: string, tags: Record<string, string | number | null>, has_exif: boolean, has_id3: boolean, raw_fields: Array<{ key: string, value: string, editable: boolean }> }> }
- **Errors:** NO_METADATA: File contains no readable metadata, UNSUPPORTED_FORMAT: Metadata reading not supported for this file type, CORRUPTED_METADATA: Metadata block is malformed

### write_metadata
- **Method:** INVOKE
- **Path:** invoke('write_metadata', { file_ids: string[], changes: MetadataChanges })
- **Auth:** None (local IPC)
- **Request:** { file_ids: string[], changes: { tags: Record<string, string | number | null>, strip_all_exif: boolean, strip_all_id3: boolean } }
- **Response:** { job_id: string, status: 'started', file_count: number }
- **Errors:** WRITE_FAILED: Could not write metadata to file, READ_ONLY_TAG: Attempted to modify a non-editable field, BACKUP_FAILED: Could not create backup before metadata write

### get_job_history
- **Method:** INVOKE
- **Path:** invoke('get_job_history', { limit: number, offset: number, search?: string })
- **Auth:** None (local IPC)
- **Request:** { limit: number (default 50, max 200), offset: number (default 0), search?: string (FTS5 query against job descriptions and file names) }
- **Response:** { jobs: Array<{ id: string, timestamp: string (ISO 8601), operation_type: 'rename' | 'convert' | 'metadata', status: 'completed' | 'partial' | 'failed' | 'rolled_back', file_count: number, description: string, can_undo: boolean }>, total_count: number, has_more: boolean }
- **Errors:** DB_ERROR: SQLite query failed, INVALID_SEARCH: FTS5 query syntax error

### get_job_detail
- **Method:** INVOKE
- **Path:** invoke('get_job_detail', { job_id: string })
- **Auth:** None (local IPC)
- **Request:** { job_id: string }
- **Response:** { job: { id: string, timestamp: string, operation_type: string, status: string, file_count: number, description: string, can_undo: boolean, files: Array<{ id: string, original_name: string, transformed_name: string, original_path: string, transformed_path: string, backup_path: string | null, format_from: string | null, format_to: string | null, status: 'success' | 'failed' | 'skipped', error_message: string | null }> } }
- **Errors:** JOB_NOT_FOUND: No job exists with this ID, DB_ERROR: SQLite query failed

### undo_job
- **Method:** INVOKE
- **Path:** invoke('undo_job', { job_id: string })
- **Auth:** None (local IPC)
- **Request:** { job_id: string }
- **Response:** { success: boolean, files_restored: number, files_failed: number, errors: Array<{ file_id: string, error: string }> }
- **Errors:** JOB_NOT_FOUND: No job exists with this ID, ALREADY_ROLLED_BACK: Job was already undone, BACKUP_MISSING: Backup files no longer exist on disk, PARTIAL_RESTORE: Some files restored, others failed — details in errors array

### get_settings
- **Method:** INVOKE
- **Path:** invoke('get_settings')
- **Auth:** None (local IPC)
- **Request:** {}
- **Response:** { settings: { theme: 'dark' | 'light', accent_color: 'blue' | 'violet', default_output_dir: string | null, max_parallel_jobs: number, auto_backup: boolean, backup_retention_days: number, last_rename_pattern: RenamePattern | null, last_convert_format: string | null, file_hard_cap: number } }
- **Errors:** DB_ERROR: SQLite read failed

### update_settings
- **Method:** INVOKE
- **Path:** invoke('update_settings', { settings: Partial<Settings> })
- **Auth:** None (local IPC)
- **Request:** { settings: Partial<Settings> } — any subset of settings fields to update
- **Response:** { success: boolean }
- **Errors:** INVALID_VALUE: Setting value is out of valid range, DB_ERROR: SQLite write failed

### subscribe_job_progress
- **Method:** LISTEN
- **Path:** listen('job_progress', callback)
- **Auth:** None (local IPC)
- **Request:** Frontend subscribes via Tauri listen() — no request payload, events pushed by backend during active jobs
- **Response:** Event payload: { job_id: string, file_id: string, file_name: string, status: 'processing' | 'completed' | 'failed', progress_percent: number (0-100), error_message: string | null, files_completed: number, files_total: number }

### subscribe_job_complete
- **Method:** LISTEN
- **Path:** listen('job_complete', callback)
- **Auth:** None (local IPC)
- **Request:** Frontend subscribes via Tauri listen() — fires once when entire job finishes
- **Response:** Event payload: { job_id: string, status: 'completed' | 'partial' | 'failed', files_completed: number, files_failed: number, duration_ms: number }

### cancel_job
- **Method:** INVOKE
- **Path:** invoke('cancel_job', { job_id: string })
- **Auth:** None (local IPC)
- **Request:** { job_id: string }
- **Response:** { cancelled: boolean, files_completed_before_cancel: number }
- **Errors:** JOB_NOT_FOUND: No active job with this ID, ALREADY_COMPLETE: Job finished before cancel was processed

### open_file_picker
- **Method:** INVOKE
- **Path:** invoke('open_file_picker', { filters?: FileFilter[] })
- **Auth:** None (local IPC)
- **Request:** { filters?: Array<{ name: string, extensions: string[] }> } — e.g. [{ name: 'Audio', extensions: ['mp3','wav','flac','m4a'] }]
- **Response:** { paths: string[] } — selected file paths, empty array if cancelled
- **Errors:** DIALOG_ERROR: Native file dialog failed to open

### cleanup_backups
- **Method:** INVOKE
- **Path:** invoke('cleanup_backups', { older_than_days?: number })
- **Auth:** None (local IPC)
- **Request:** { older_than_days?: number (defaults to backup_retention_days setting) }
- **Response:** { files_removed: number, space_freed_bytes: number }
- **Errors:** CLEANUP_FAILED: Could not remove some backup files

## 🧱 Modules
### ui-shell
- **Responsibilities:**
- Render top navbar with app branding, settings gear, and help icon
- Manage global layout: drag-drop zone, file list, transformation panel, action footer
- Handle theme switching (dark/light) via Tailwind CSS 4 theme tokens and CSS custom properties
- Provide keyboard navigation framework with visible 2px accent focus rings
- Manage responsive breakpoints: sidebar right on desktop, collapsed to bottom on tablet
- **Interfaces:**
- AppShell component: wraps entire app, provides ThemeContext and AppStateContext
- Navbar component: renders sticky top bar, emits onSettingsClick and onHelpClick
- useTheme() hook: returns { theme, toggleTheme, accentColor }
- **Dependencies:**
- state-management

### drag-drop-zone
- **Responsibilities:**
- Render glassmorphic drop zone with dashed border and animated file icon when empty
- Handle HTML5 drag-and-drop events: dragenter, dragover, dragleave, drop
- Extract file paths from drop events and invoke add_files Tauri command
- Trigger open_file_picker command on click as fallback
- Show visual feedback: border glow on drag-over, background shift
- Validate dropped items are files (not directories or URLs) before processing
- **Interfaces:**
- DropZone component: props { onFilesAdded: (files: FileInfo[]) => void, isEmpty: boolean }
- useDragDrop() hook: returns { isDragging, dragRef, handleDrop }
- **Dependencies:**
- tauri-ipc-layer

### file-list
- **Responsibilities:**
- Render horizontally scrollable list of file cards in center-bottom area
- Display per-file: thumbnail/icon, original name, transformed name (accent color), format badge, status indicator
- Update file card status in real-time as job_progress events arrive
- Support hover state with lift effect and delete icon reveal
- Handle file selection for batch operations
- Show staggered list item animations during processing (50ms offset per item)
- **Interfaces:**
- FileList component: props { files: FileInfo[], onRemoveFile: (id: string) => void, onSelectFiles: (ids: string[]) => void }
- FileCard component: props { file: FileInfo, isSelected: boolean, onClick, onRemove }
- FormatBadge component: props { format: string }
- **Dependencies:**
- state-management
- tauri-ipc-layer

### transformation-panel
- **Responsibilities:**
- Render collapsible right sidebar with three tabs: Rename, Convert, Metadata
- Rename tab: regex input, template builder with quick-buttons ({date}, {number}, {original}), live preview toggle
- Convert tab: format dropdown, quality slider, collapsible advanced options
- Metadata tab: tag editor fields, bulk strip button
- Call preview_rename on every pattern change (debounced 150ms) for live preview
- Validate inputs and show inline error states
- **Interfaces:**
- TransformationPanel component: props { files: FileInfo[], activeTab: TabType, onTabChange }
- RenameTab component: props { files, onPatternChange: (pattern: RenamePattern) => void, previews: PreviewResult[] }
- ConvertTab component: props { files, onOptionsChange: (options: ConvertOptions) => void }
- MetadataTab component: props { files, metadata: MetadataInfo[], onChanges: (changes: MetadataChanges) => void }
- useRenamePreview(files, pattern) hook: returns { previews, conflicts, isLoading }
- **Dependencies:**
- state-management
- tauri-ipc-layer

### action-footer
- **Responsibilities:**
- Render sticky bottom bar with file counter, Apply button, history dropdown, and undo button
- Disable Apply when no files loaded or no transformation configured
- Show progress bar within Apply button during active job
- Render history dropdown with recent jobs on click
- Show Undo button immediately after a successful apply, call undo_job on click
- Display confetti particle effect on job success (optional, toggleable)
- **Interfaces:**
- ActionFooter component: props { fileCount: number, canApply: boolean, isProcessing: boolean, lastJobId: string | null, onApply, onUndo, onShowHistory }
- HistoryDropdown component: props { jobs: JobSummary[], onSelectJob, onClose }
- **Dependencies:**
- state-management
- tauri-ipc-layer

### state-management
- **Responsibilities:**
- Provide AppStateContext with useReducer for global state: files list, active job, rename previews, current tab, processing status
- Define action types: ADD_FILES, REMOVE_FILE, SET_PREVIEWS, START_JOB, UPDATE_FILE_STATUS, COMPLETE_JOB, SET_SETTINGS
- Sync Tauri backend events (job_progress, job_complete) into React state via useTauriEvent hook
- Persist user settings to backend on change via update_settings command
- Memoize derived state: file counts by status, conflict count, canApply boolean
- **Interfaces:**
- AppStateContext: provides { state: AppState, dispatch: Dispatch<AppAction> }
- AppState type: { files: FileInfo[], previews: PreviewResult[], activeJob: ActiveJob | null, settings: Settings, history: JobSummary[] }
- useAppState() hook: returns { state, dispatch }
- useTauriEvent(eventName, handler) hook: subscribes to Tauri events, cleans up on unmount
- useFileStats() hook: returns memoized { total, pending, processing, completed, failed }
- **Dependencies:**
- tauri-ipc-layer

### tauri-ipc-layer
- **Responsibilities:**
- Wrap all Tauri invoke() calls in typed async functions with error handling
- Wrap all Tauri listen() subscriptions in typed helpers that return cleanup functions
- Normalize Rust error strings into structured TypeScript error objects with error codes
- Provide a single commands.ts module exporting every IPC function with full TypeScript types
- Provide an events.ts module exporting every event listener with typed payloads
- **Interfaces:**
- commands.ts: export async function addFiles(paths: string[]): Promise<AddFilesResponse>
- commands.ts: export async function previewRename(fileIds: string[], pattern: RenamePattern): Promise<PreviewResponse>
- commands.ts: export async function applyRename(fileIds: string[], pattern: RenamePattern): Promise<JobStartResponse>
- commands.ts: export async function convertFiles(fileIds: string[], options: ConvertOptions): Promise<JobStartResponse>
- commands.ts: export async function readMetadata(fileIds: string[]): Promise<MetadataResponse>
- commands.ts: export async function writeMetadata(fileIds: string[], changes: MetadataChanges): Promise<JobStartResponse>
- commands.ts: export async function getJobHistory(limit, offset, search?): Promise<HistoryResponse>
- commands.ts: export async function undoJob(jobId: string): Promise<UndoResponse>
- commands.ts: export async function cancelJob(jobId: string): Promise<CancelResponse>
- commands.ts: export async function getSettings(): Promise<SettingsResponse>
- commands.ts: export async function updateSettings(settings: Partial<Settings>): Promise<void>
- commands.ts: export async function cleanupBackups(olderThanDays?: number): Promise<CleanupResponse>
- events.ts: export function onJobProgress(callback: (payload: JobProgressEvent) => void): UnlistenFn
- events.ts: export function onJobComplete(callback: (payload: JobCompleteEvent) => void): UnlistenFn
- types.ts: all shared TypeScript interfaces and type aliases for IPC payloads

### rust-command-handlers
- **Responsibilities:**
- Define all #[tauri::command] functions as the entry point for frontend IPC calls
- Deserialize incoming JSON args into Rust structs using serde
- Delegate to appropriate service modules (FileService, ConversionService, etc.)
- Return Result<T, String> serialized back to frontend as JSON
- Emit progress events via app_handle.emit() for long-running operations
- Register all commands in tauri::Builder::default().invoke_handler()
- **Interfaces:**
- #[tauri::command] async fn add_files(paths: Vec<String>) -> Result<AddFilesResponse, String>
- #[tauri::command] async fn preview_rename(file_ids: Vec<String>, pattern: RenamePattern) -> Result<PreviewResponse, String>
- #[tauri::command] async fn apply_rename(app: tauri::AppHandle, file_ids: Vec<String>, pattern: RenamePattern) -> Result<JobStartResponse, String>
- #[tauri::command] async fn convert_files(app: tauri::AppHandle, file_ids: Vec<String>, options: ConvertOptions) -> Result<JobStartResponse, String>
- #[tauri::command] async fn read_metadata(file_ids: Vec<String>) -> Result<MetadataResponse, String>
- #[tauri::command] async fn write_metadata(app: tauri::AppHandle, file_ids: Vec<String>, changes: MetadataChanges) -> Result<JobStartResponse, String>
- #[tauri::command] async fn get_job_history(limit: u32, offset: u32, search: Option<String>) -> Result<HistoryResponse, String>
- #[tauri::command] async fn undo_job(app: tauri::AppHandle, job_id: String) -> Result<UndoResponse, String>
- #[tauri::command] async fn cancel_job(job_id: String) -> Result<CancelResponse, String>
- #[tauri::command] async fn get_settings() -> Result<SettingsResponse, String>
- #[tauri::command] async fn update_settings(settings: PartialSettings) -> Result<(), String>
- #[tauri::command] async fn cleanup_backups(older_than_days: Option<u32>) -> Result<CleanupResponse, String>
- **Dependencies:**
- file-service
- conversion-service
- metadata-service
- history-service
- processing-pipeline

### file-service
- **Responsibilities:**
- Validate file paths exist and are readable
- Detect file type from extension and magic bytes
- Generate thumbnail data URLs for images (resized to 64x64) and placeholder icons for audio/video/docs
- Execute batch rename by copying files to new names (not in-place move, backup-first strategy)
- Create timestamped backup copies of original files before any destructive operation
- Restore files from backups during undo/rollback operations
- Detect and report naming conflicts before execution
- **Interfaces:**
- pub fn validate_paths(paths: &[String]) -> Result<Vec<FileInfo>, FileServiceError>
- pub fn generate_thumbnail(path: &str, file_type: &FileType) -> Option<String>
- pub fn create_backup(path: &str, backup_dir: &Path) -> Result<String, FileServiceError>
- pub fn restore_from_backup(backup_path: &str, original_path: &str) -> Result<(), FileServiceError>
- pub fn execute_rename(file: &FileInfo, new_name: &str, backup_dir: &Path) -> Result<RenameResult, FileServiceError>
- pub fn detect_conflicts(files: &[FileInfo], new_names: &[String]) -> Vec<ConflictInfo>
- **Dependencies:**
- history-service

### conversion-service
- **Responsibilities:**
- Audio conversion: MP3, WAV, FLAC, M4A via ffmpeg-next crate bindings
- Image conversion: JPG, PNG, WebP, AVIF via image crate (pure Rust, no ffmpeg needed)
- Video conversion: MP4, WebM, MKV via ffmpeg-next crate bindings
- Apply quality settings: bitrate for audio, compression level for images, codec params for video
- Validate source-to-target format compatibility before processing
- Report per-file progress percentage during conversion
- Backup original files before writing converted output
- **Interfaces:**
- pub fn convert_audio(input: &Path, output: &Path, format: AudioFormat, quality: Option<u8>) -> Result<(), ConversionError>
- pub fn convert_image(input: &Path, output: &Path, format: ImageFormat, quality: Option<u8>, resize: Option<ResizeParams>) -> Result<(), ConversionError>
- pub fn convert_video(input: &Path, output: &Path, format: VideoFormat, codec: Option<String>, bitrate: Option<String>) -> Result<(), ConversionError>
- pub fn validate_conversion(source_format: &str, target_format: &str) -> bool
- pub fn supported_formats() -> SupportedFormats

### metadata-service
- **Responsibilities:**
- Read ID3v2 tags from audio files using id3 crate
- Write and update ID3v2 tags on audio files
- Read EXIF data from image files using kamadak-exif crate
- Strip all EXIF data from images by rewriting without metadata
- Identify which metadata fields are editable vs read-only
- Support bulk metadata operations across multiple files
- **Interfaces:**
- pub fn read_id3_tags(path: &Path) -> Result<HashMap<String, TagValue>, MetadataError>
- pub fn write_id3_tags(path: &Path, tags: &HashMap<String, TagValue>) -> Result<(), MetadataError>
- pub fn read_exif(path: &Path) -> Result<HashMap<String, String>, MetadataError>
- pub fn strip_exif(path: &Path) -> Result<(), MetadataError>
- pub fn strip_id3(path: &Path) -> Result<(), MetadataError>
- pub fn get_editable_fields(file_type: &FileType) -> Vec<FieldDescriptor>

### preview-service
- **Responsibilities:**
- Apply regex patterns to file names in-memory without disk access
- Apply template patterns with variable substitution: {date}, {number}, {original}, {ext}
- Apply sequential numbering with configurable start, zero-padding, prefix, suffix
- Apply case transforms: uppercase, lowercase, title case
- Detect output name conflicts within the batch
- Return before/after pairs for all files within 100ms for up to 500 files
- **Interfaces:**
- pub fn generate_previews(files: &[FileInfo], pattern: &RenamePattern) -> Result<Vec<PreviewPair>, PreviewError>
- pub fn validate_pattern(pattern: &RenamePattern) -> Result<(), PreviewError>
- pub fn detect_output_conflicts(previews: &[PreviewPair]) -> Vec<ConflictInfo>

### history-service
- **Responsibilities:**
- Initialize SQLite database and run forward-only migrations on app startup
- Insert job records with operation type, timestamp, file count, and description
- Insert per-file records linked to jobs with original/transformed paths and backup locations
- Query job history with pagination and FTS5 full-text search
- Mark jobs as rolled_back after successful undo operations
- Clean up old backup files based on retention policy
- Track whether a job can be undone (backup files still exist on disk)
- **Interfaces:**
- pub fn init_db(app_data_dir: &Path) -> Result<Connection, DbError>
- pub fn run_migrations(conn: &Connection) -> Result<(), DbError>
- pub fn create_job(conn: &Connection, job: &NewJob) -> Result<String, DbError>
- pub fn add_job_file(conn: &Connection, job_id: &str, file: &JobFileRecord) -> Result<(), DbError>
- pub fn update_job_status(conn: &Connection, job_id: &str, status: JobStatus) -> Result<(), DbError>
- pub fn update_file_status(conn: &Connection, file_id: &str, status: FileStatus, error: Option<String>) -> Result<(), DbError>
- pub fn get_history(conn: &Connection, limit: u32, offset: u32, search: Option<&str>) -> Result<HistoryPage, DbError>
- pub fn get_job_detail(conn: &Connection, job_id: &str) -> Result<JobDetail, DbError>
- pub fn mark_rolled_back(conn: &Connection, job_id: &str) -> Result<(), DbError>
- pub fn check_backups_exist(conn: &Connection, job_id: &str) -> Result<bool, DbError>

### processing-pipeline
- **Responsibilities:**
- Manage a thread pool via Rayon for CPU-bound parallel file processing
- Accept a batch job (rename, convert, or metadata) and distribute across worker threads
- Track per-file status: pending, processing, completed, failed
- Emit job_progress Tauri events as each file completes or fails
- Emit job_complete Tauri event when all files in a batch are processed
- Support job cancellation by checking a shared AtomicBool flag between iterations
- Create backup copies of all files before processing begins
- Coordinate with HistoryService to persist job and file records
- **Interfaces:**
- pub fn execute_batch_rename(app: &AppHandle, files: Vec<FileInfo>, pattern: RenamePattern, conn: &Connection) -> Result<String, PipelineError>
- pub fn execute_batch_convert(app: &AppHandle, files: Vec<FileInfo>, options: ConvertOptions, conn: &Connection) -> Result<String, PipelineError>
- pub fn execute_batch_metadata(app: &AppHandle, files: Vec<FileInfo>, changes: MetadataChanges, conn: &Connection) -> Result<String, PipelineError>
- pub fn cancel_job(job_id: &str) -> Result<bool, PipelineError>
- pub fn get_active_job() -> Option<ActiveJobInfo>
- **Dependencies:**
- file-service
- conversion-service
- metadata-service
- history-service

## 🗃 Data Model Notes
- TABLE jobs: id TEXT PRIMARY KEY (UUID v4), created_at TEXT NOT NULL (ISO 8601), operation_type TEXT NOT NULL CHECK(operation_type IN ('rename','convert','metadata')), status TEXT NOT NULL CHECK(status IN ('running','completed','partial','failed','rolled_back')), file_count INTEGER NOT NULL, description TEXT NOT NULL

- TABLE job_files: id TEXT PRIMARY KEY (UUID v4), job_id TEXT NOT NULL REFERENCES jobs(id), original_path TEXT NOT NULL, original_name TEXT NOT NULL, transformed_name TEXT, transformed_path TEXT, backup_path TEXT, format_from TEXT, format_to TEXT, status TEXT NOT NULL CHECK(status IN ('pending','processing','success','failed','skipped')), error_message TEXT, created_at TEXT NOT NULL

- TABLE settings: key TEXT PRIMARY KEY, value TEXT NOT NULL, updated_at TEXT NOT NULL — key-value store for theme, accent_color, default_output_dir, max_parallel_jobs, auto_backup, backup_retention_days, last_rename_pattern (JSON), last_convert_format, file_hard_cap

- TABLE migrations: version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL — tracks which migrations have been applied

- VIRTUAL TABLE job_search USING fts5(job_id, description, file_names, content=job_files, content_rowid=rowid) — FTS5 index for searching job history by description and file names

- INDEX idx_job_files_job_id ON job_files(job_id) — fast lookup of files belonging to a job

- INDEX idx_jobs_created_at ON jobs(created_at DESC) — fast reverse-chronological history queries

- INDEX idx_jobs_status ON jobs(status) — fast filtering by job status

- All IDs are UUID v4 strings generated in Rust via uuid crate

- Timestamps are ISO 8601 strings in UTC stored as TEXT (SQLite has no native datetime type)

- backup_path in job_files stores the absolute path to the backup copy; NULL if no backup was created (e.g., metadata read-only operations)

- In-memory state on the frontend mirrors a subset: files array with id, names, paths, status, preview data. Not persisted in SQLite until a job is executed

## 🔐 Validation & Security
- All file paths are canonicalized (std::fs::canonicalize) before any operation to prevent path traversal attacks
- File paths are validated to exist and be readable before adding to the file list
- Regex patterns are compiled with a 100ms timeout (regex crate's size limit) to prevent ReDoS
- Template variables are restricted to a whitelist: {date}, {number}, {original}, {ext}, {parent} — unknown variables produce a validation error
- Format conversion target is validated against an explicit allowlist of supported format pairs
- SQLite queries use prepared statements exclusively — no string concatenation for SQL
- FTS5 search queries are sanitized: special characters are escaped before passing to MATCH
- File hard cap (default 5000) is enforced on add_files to prevent memory exhaustion
- Output filenames are sanitized: control characters, path separators, and OS-reserved names (CON, PRN, NUL on Windows) are rejected
- Backup directory is within the Tauri app data directory; user cannot specify arbitrary backup locations
- No network calls are made by any module — all processing is local. CSP headers in Tauri config restrict webview to local resources only
- Tauri allowlist restricts IPC commands to only the explicitly registered command set
- Frontend validates all user inputs (pattern, format selection, quality range 0-100) before sending to backend; backend re-validates independently

## 🧯 Error Handling Strategy
All Rust commands return Result<T, String> where the error string follows a structured format: 'ERROR_CODE: Human readable message'. The frontend IPC layer (commands.ts) parses this into { code: string, message: string } objects. UI components display errors contextually: inline for field validation (red border + message below input), toast notifications for operation-level errors (3s auto-dismiss with manual close), and modal dialogs for critical failures (backup failure, partial job failure requiring user decision). Partial failures in batch operations are tracked per-file: the job completes with status 'partial', and the job detail view shows which files succeeded and which failed with individual error messages. Undo of partial jobs restores only the files that were successfully modified. All errors are logged to the Rust tracing subscriber for debugging. Panics in worker threads are caught by Rayon and converted to per-file errors rather than crashing the app.

## 🔭 Observability
- **Logging:** Rust backend uses the tracing crate with tracing-subscriber for structured logging. Log levels: ERROR for operation failures, WARN for recoverable issues (backup cleanup failures, unsupported file skipped), INFO for job lifecycle events (start, complete, undo), DEBUG for per-file processing details. Logs are written to a rotating file in the Tauri app log directory (platform-specific) with a 10MB max file size and 5 file rotation. Frontend logs critical IPC errors to console.error with structured JSON.
- **Tracing:** Not applicable for a local desktop app — no distributed tracing. Job IDs (UUID v4) serve as correlation IDs across all log entries for a single batch operation. All log lines for a job include the job_id field for filtering. Per-file log lines include both job_id and file_id for granular tracing.
- **Metrics:**
- Job execution duration (ms) per operation type, logged at INFO level on job completion
- Files processed per job (count), logged with job completion
- Per-file processing time (ms), logged at DEBUG level
- Error rate per job (files_failed / files_total), logged at INFO level
- App startup time (ms) from process start to webview ready, logged at INFO level
- SQLite query duration for history queries exceeding 50ms, logged at WARN level
- Backup disk usage (bytes) tracked in cleanup_backups response

## ⚡ Performance Notes
- Rayon thread pool defaults to num_cpus::get() threads for CPU-bound file processing; configurable via max_parallel_jobs setting
- Rename preview (preview_rename) runs entirely in-memory with no disk I/O; must complete in under 100ms for 500 files — benchmark in CI
- Image conversion uses the pure-Rust image crate instead of ffmpeg for lower overhead and smaller binary size
- Video and audio conversion uses ffmpeg-next with codec-specific optimizations: hardware acceleration where available (VideoToolbox on macOS, VAAPI on Linux)
- Large video files use memory-mapped I/O (memmap2 crate) to avoid loading entire files into RAM
- Frontend debounces preview_rename calls by 150ms to avoid flooding the IPC bridge during rapid typing
- File thumbnails are generated once on add_files and cached in the frontend state; not regenerated on re-render
- SQLite WAL mode enabled for non-blocking reads during writes; connection pool size of 1 writer + 3 readers via r2d2-sqlite
- FTS5 index is rebuilt incrementally on job insertion, not full rebuild
- Tauri IPC serialization uses serde_json; large payloads (500+ file previews) are benchmarked to ensure sub-100ms round-trip
- Frontend file list uses virtualized rendering (react-window) if file count exceeds 100 to prevent DOM bloat
- All CSS transitions use 200ms ease-out with will-change hints on animated properties to leverage GPU compositing
- App bundle size target under 10MB achieved by: stripping debug symbols, including only required ffmpeg codecs (libmp3lame, libvorbis, libvpx, libx264, libopus), LTO enabled in release builds

## 🧪 Testing Strategy
### Unit
- preview-service: Test regex application, template substitution, numbering sequences, case transforms, conflict detection — pure functions with no I/O, fast execution
- file-service: Test path validation, conflict detection, file type detection from extension — mock filesystem where needed
- conversion-service: Test format validation (supported pairs), option normalization — mock actual ffmpeg calls
- metadata-service: Test ID3 tag parsing, EXIF field extraction — use fixture files checked into repo
- history-service: Test all SQL operations against an in-memory SQLite database — migrations, CRUD, FTS5 queries, pagination
- Frontend commands.ts: Test error parsing from Rust error strings into typed objects — mock Tauri invoke
- Frontend state reducer: Test all action types produce correct state transitions — pure reducer function tests
- Frontend useRenamePreview hook: Test debounce behavior and preview result integration — mock IPC layer
### Integration
- Rust command handlers: Test full invoke flow from deserialized args through service calls to serialized response — use temp directories and real files
- Processing pipeline: Test batch rename of 10 real files with backup creation, progress event emission, and history record insertion — temp directory with fixture files
- Processing pipeline: Test batch convert of audio files (MP3→WAV) end-to-end with progress tracking — requires ffmpeg in CI
- Undo flow: Test apply_rename followed by undo_job restores all original files byte-for-byte — temp directory comparison
- History + FTS5: Test creating jobs, querying history with search terms, verifying FTS5 results match inserted descriptions
- Frontend + IPC: Test React components rendering correctly when Tauri invoke returns success and error responses — mock Tauri runtime
### E2E
- Drag-drop flow: Launch app, simulate file drop, verify files appear in list with correct names and types
- Rename flow: Add files, enter regex pattern, verify preview updates, click Apply, verify files renamed on disk and job appears in history
- Convert flow: Add MP3 files, select WAV target, click Apply, verify WAV files created with correct format, verify progress bar updates
- Undo flow: Complete a rename job, click Undo, verify original files restored and job status is rolled_back
- History search: Complete multiple jobs, open history, search by filename, verify correct job appears
- Error handling: Add read-only file, attempt rename, verify error displayed inline on file card without crashing app
- Theme toggle: Switch from dark to light mode, verify all components re-render with correct theme tokens

## 🚀 Rollout Plan
- Phase 1 — MVP (Week 1): Implement drag-drop file input, preview-service for rename preview (regex + templates), apply_rename with backup strategy, basic audio conversion (MP3↔WAV via ffmpeg-next), file list UI with status indicators, action footer with Apply button, SQLite history with job logging. Target: functional rename + convert with history.

- Phase 2 — Core Polish (Week 2): Add full audio format support (FLAC, M4A), image conversion (JPG, PNG, WebP, AVIF via image crate), metadata reading and display (ID3 + EXIF), undo/rollback via backup restoration, history dropdown with job detail view, FTS5 search in history, real-time progress events with per-file status updates.

- Phase 3 — Power Features (Week 3): Add video conversion (MP4, WebM, MKV), metadata writing and bulk strip, parallel processing pipeline with Rayon, job cancellation, advanced convert options (quality slider, codec selection, image resize), template quick-buttons in rename tab, keyboard accessibility audit and ARIA labels.

- Phase 4 — Ship (Week 4): Cross-platform CI/CD with GitHub Actions (macOS, Windows, Linux matrix builds), auto-updater configuration with GitHub Releases manifest, app signing for macOS and Windows, backup cleanup scheduler, performance benchmarking (preview < 100ms, startup < 2s), light mode theme, final UI polish (animations, confetti, glassmorphism refinement), beta testing with 5-10 target users.

## ❓ Open Questions
- Should ffmpeg be statically linked into the binary (larger bundle, zero user setup) or dynamically loaded with a runtime check and installation prompt? Static linking simplifies UX but may push bundle over 10MB target for video codecs.
- What is the backup retention policy default? 7 days, 30 days, or unlimited until manual cleanup? Longer retention is safer for undo but consumes disk space proportional to file sizes processed.
- Should the app support processing directories recursively (drag a folder, process all matching files inside) or strictly individual files? Recursive adds complexity in UI presentation and conflict detection.
- For video conversion, should hardware acceleration (VideoToolbox, VAAPI, NVENC) be auto-detected and used when available, or should it be an explicit user setting? Auto-detection is better UX but harder to test cross-platform.
- Should the file hard cap (default 5000) be a hard rejection or a soft warning that the user can override? Hard cap prevents memory issues but may frustrate power users with large batches.
- Is a portable/no-install version needed for users who cannot install apps (e.g., corporate machines)? This affects packaging strategy and app data directory location.