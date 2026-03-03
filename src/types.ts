// Shared types matching Rust serde structs in src-tauri/src/types.rs

export type FileType = "audio" | "image" | "video" | "document";

export type FileStatus = "pending" | "processing" | "success" | "failed" | "skipped";

export interface FileInfo {
  id: string;
  original_name: string;
  original_path: string;
  extension: string;
  size_bytes: number;
  file_type: FileType;
  thumbnail_data_url: string | null;
  status: FileStatus;
}

export type RenameMode = "regex" | "template" | "numbering";

export type CaseTransform = "none" | "upper" | "lower" | "title";

export interface RenamePattern {
  mode: RenameMode;
  regex_find?: string;
  regex_replace?: string;
  template?: string;
  start_number?: number;
  zero_pad?: number;
  prefix?: string;
  suffix?: string;
  case_transform?: CaseTransform;
}

export interface ConvertOptions {
  target_format: string;
  quality?: number;
  output_dir?: string;
  overwrite_existing: boolean;
  video_codec?: string;
  audio_bitrate?: string;
  image_resize?: ResizeParams;
}

export interface ResizeParams {
  width: number;
  height: number;
  maintain_aspect: boolean;
}

export interface MetadataChanges {
  tags: Record<string, string | null>;
  strip_all_exif: boolean;
  strip_all_id3: boolean;
}

export interface PreviewResult {
  file_id: string;
  original_name: string;
  transformed_name: string;
  has_conflict: boolean;
  conflict_reason: string | null;
}

export interface JobRecord {
  id: string;
  created_at: string;
  operation_type: "rename" | "convert" | "metadata";
  status: "running" | "completed" | "partial" | "failed" | "rolled_back";
  file_count: number;
  description: string;
  can_undo: boolean;
}

export interface JobFileRecord {
  id: string;
  job_id: string;
  original_path: string;
  original_name: string;
  transformed_name: string | null;
  transformed_path: string | null;
  backup_path: string | null;
  format_from: string | null;
  format_to: string | null;
  status: "pending" | "processing" | "success" | "failed" | "skipped";
  error_message: string | null;
}

export interface JobStartResponse {
  job_id: string;
  status: string;
  file_count: number;
}

export interface JobProgressEvent {
  job_id: string;
  file_id: string;
  file_name: string;
  status: string;
  progress_percent: number;
  error_message: string | null;
  files_completed: number;
  files_total: number;
}

export interface JobCompleteEvent {
  job_id: string;
  status: string;
  files_completed: number;
  files_failed: number;
  duration_ms: number;
}

export interface Settings {
  theme: "dark" | "light";
  accent_color: string;
  default_output_dir: string | null;
  max_parallel_jobs: number;
  auto_backup: boolean;
  backup_retention_days: number;
  last_rename_pattern: RenamePattern | null;
  last_convert_format: string | null;
  file_hard_cap: number;
}

export interface MetadataField {
  key: string;
  value: string;
  editable: boolean;
}

export interface MetadataInfo {
  file_id: string;
  file_type: string;
  tags: Record<string, string | number | null>;
  has_exif: boolean;
  has_id3: boolean;
  raw_fields: MetadataField[];
}

export interface IpcError {
  code: string;
  message: string;
}

export interface UndoResponse {
  success: boolean;
  files_restored: number;
  files_failed: number;
  errors: Array<{ file_id: string; error: string }>;
}

export interface HistoryResponse {
  jobs: JobRecord[];
  total_count: number;
  has_more: boolean;
}

export interface JobDetail {
  job: JobRecord & {
    files: JobFileRecord[];
  };
}

export interface CleanupResponse {
  files_removed: number;
  space_freed_bytes: number;
}

export interface CancelResponse {
  cancelled: boolean;
  files_completed_before_cancel: number;
}

export interface AddFilesResponse {
  files: FileInfo[];
}

export interface PreviewResponse {
  previews: PreviewResult[];
  total_conflicts: number;
}

export interface MetadataResponse {
  metadata: MetadataInfo[];
}
