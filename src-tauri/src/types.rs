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
    Done,
    Error,
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
    pub transformed_name: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenameMode {
    Regex,
    Template,
    Numbering,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub case_transform: CaseTransform,
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
pub struct PreviewPair {
    pub file_id: String,
    pub original_name: String,
    pub transformed_name: String,
    pub has_conflict: bool,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewResponse {
    pub previews: Vec<PreviewPair>,
    pub total_conflicts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFilesResponse {
    pub files: Vec<FileInfo>,
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
    pub progress_percent: f32,
    pub error_message: Option<String>,
    pub files_completed: u32,
    pub files_total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompleteEvent {
    pub job_id: String,
    pub status: String,
    pub files_completed: u32,
    pub files_failed: u32,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Running,
    Completed,
    Partial,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    Rename,
    Convert,
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSummary {
    pub id: String,
    pub timestamp: String,
    pub operation_type: String,
    pub status: String,
    pub file_count: u32,
    pub description: String,
    pub can_undo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub jobs: Vec<JobSummary>,
    pub total_count: u32,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UndoResponse {
    pub success: bool,
    pub files_restored: u32,
    pub files_failed: u32,
    pub errors: Vec<FileError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileError {
    pub file_id: String,
    pub error: String,
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            accent_color: "blue".into(),
            default_output_dir: None,
            max_parallel_jobs: num_cpus(),
            auto_backup: true,
            backup_retention_days: 30,
            last_rename_pattern: None,
            last_convert_format: None,
            file_hard_cap: 5000,
        }
    }
}

fn num_cpus() -> u32 {
    std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(4)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataField {
    pub key: String,
    pub value: String,
    pub editable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataInfo {
    pub file_id: String,
    pub file_type: String,
    pub fields: Vec<MetadataField>,
}
