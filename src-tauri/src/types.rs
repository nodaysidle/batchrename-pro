use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileType {
    Audio,
    Image,
    Video,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Pending,
    Processing,
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub original_name: String,
    pub original_path: String,
    pub extension: String,
    pub size_bytes: u64,
    pub file_type: FileType,
    pub thumbnail_data_url: Option<String>,
    pub status: FileStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RenameMode {
    Regex,
    Template,
    Numbering,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CaseTransform {
    None,
    Upper,
    Lower,
    Title,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamePattern {
    pub mode: RenameMode,
    pub regex_find: Option<String>,
    pub regex_replace: Option<String>,
    pub template: Option<String>,
    pub start_number: Option<u32>,
    pub zero_pad: Option<u32>,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub case_transform: Option<CaseTransform>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertOptions {
    pub target_format: String,
    pub quality: Option<u8>,
    pub output_dir: Option<String>,
    pub overwrite_existing: bool,
    pub video_codec: Option<String>,
    pub audio_bitrate: Option<String>,
    pub image_resize: Option<ResizeParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeParams {
    pub width: u32,
    pub height: u32,
    pub maintain_aspect: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataChanges {
    pub tags: std::collections::HashMap<String, Option<String>>,
    pub strip_all_exif: bool,
    pub strip_all_id3: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResult {
    pub file_id: String,
    pub original_name: String,
    pub transformed_name: String,
    pub has_conflict: bool,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub created_at: String,
    pub operation_type: String,
    pub status: String,
    pub file_count: i64,
    pub description: String,
    pub can_undo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFileRecord {
    pub id: String,
    pub job_id: String,
    pub original_path: String,
    pub original_name: String,
    pub transformed_name: Option<String>,
    pub transformed_path: Option<String>,
    pub backup_path: Option<String>,
    pub format_from: Option<String>,
    pub format_to: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStartResponse {
    pub job_id: String,
    pub status: String,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgressEvent {
    pub job_id: String,
    pub file_id: String,
    pub file_name: String,
    pub status: String,
    pub progress_percent: f64,
    pub error_message: Option<String>,
    pub files_completed: usize,
    pub files_total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompleteEvent {
    pub job_id: String,
    pub status: String,
    pub files_completed: usize,
    pub files_failed: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub accent_color: String,
    pub default_output_dir: Option<String>,
    pub max_parallel_jobs: u32,
    pub auto_backup: bool,
    pub backup_retention_days: u32,
    pub last_rename_pattern: Option<RenamePattern>,
    pub last_convert_format: Option<String>,
    pub file_hard_cap: u32,
}

// ── Response wrappers for Tauri commands ────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFilesResponse {
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub previews: Vec<PreviewResult>,
    pub total_conflicts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub jobs: Vec<JobRecord>,
    pub total_count: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobWithFiles {
    pub id: String,
    pub created_at: String,
    pub operation_type: String,
    pub status: String,
    pub file_count: i64,
    pub description: String,
    pub can_undo: bool,
    pub files: Vec<JobFileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobDetailResponse {
    pub job: JobWithFiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub settings: Settings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResponse {
    pub success: bool,
    pub files_restored: usize,
    pub files_failed: usize,
    pub errors: Vec<UndoError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoError {
    pub file_id: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    pub cancelled: bool,
    pub files_completed_before_cancel: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResponse {
    pub files_removed: usize,
    pub space_freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePickerResponse {
    pub paths: Vec<String>,
}
