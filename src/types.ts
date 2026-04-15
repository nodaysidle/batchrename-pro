export type FileType = 'audio' | 'image' | 'video' | 'document';
export type FileStatus = 'pending' | 'processing' | 'done' | 'error';

export interface FileInfo {
  id: string;
  original_name: string;
  original_path: string;
  extension: string;
  size_bytes: number;
  file_type: FileType;
  thumbnail_data_url: string | null;
  status: FileStatus;
  transformed_name: string | null;
  error: string | null;
}

export type RenameMode = 'regex' | 'template' | 'numbering';
export type CaseTransform = 'none' | 'upper' | 'lower' | 'title';

export interface RenamePattern {
  mode: RenameMode;
  regex_find?: string;
  regex_replace?: string;
  template?: string;
  start_number?: number;
  zero_pad?: number;
  prefix?: string;
  suffix?: string;
  case_transform: CaseTransform;
}

export interface ConvertOptions {
  target_format: string;
  quality?: number;
  output_dir?: string;
  overwrite_existing: boolean;
  video_codec?: string;
  audio_bitrate?: string;
  image_resize?: {
    width: number;
    height: number;
    maintain_aspect: boolean;
  };
}

export interface PreviewPair {
  file_id: string;
  original_name: string;
  transformed_name: string;
  has_conflict: boolean;
  conflict_reason: string | null;
}

export interface PreviewResponse {
  previews: PreviewPair[];
  total_conflicts: number;
}

export interface AddFilesResponse {
  files: FileInfo[];
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
  status: 'processing' | 'completed' | 'failed';
  progress_percent: number;
  error_message: string | null;
  files_completed: number;
  files_total: number;
}

export interface JobCompleteEvent {
  job_id: string;
  status: 'completed' | 'partial' | 'failed';
  files_completed: number;
  files_failed: number;
  duration_ms: number;
}

export interface JobSummary {
  id: string;
  timestamp: string;
  operation_type: string;
  status: string;
  file_count: number;
  description: string;
  can_undo: boolean;
}

export interface HistoryResponse {
  jobs: JobSummary[];
  total_count: number;
  has_more: boolean;
}

export interface UndoResponse {
  success: boolean;
  files_restored: number;
  files_failed: number;
  errors: Array<{ file_id: string; error: string }>;
}

export interface Settings {
  theme: 'dark' | 'light';
  accent_color: 'blue' | 'violet';
  default_output_dir: string | null;
  max_parallel_jobs: number;
  auto_backup: boolean;
  backup_retention_days: number;
  last_rename_pattern: RenamePattern | null;
  last_convert_format: string | null;
  file_hard_cap: number;
}

export interface AppError {
  code: string;
  message: string;
}

export type TabType = 'rename' | 'convert' | 'metadata';

export interface FileStats {
  total: number;
  pending: number;
  processing: number;
  done: number;
  error: number;
}
