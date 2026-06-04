import { invoke } from '@tauri-apps/api/core';
import type {
  AddFilesResponse,
  PreviewResponse,
  RenamePattern,
  JobStartResponse,
  FileInfo,
  UndoResponse,
  HistoryResponse,
  Settings,
  AppError,
} from '@/types';

export function parseError(err: unknown): AppError {
  if (
    typeof err === 'object' &&
    err !== null &&
    'code' in err &&
    'message' in err
  ) {
    const appError = err as AppError;
    return {
      code: String(appError.code || 'UNKNOWN'),
      message: String(appError.message || 'Unknown error'),
    };
  }
  const str = typeof err === 'string' ? err : String(err);
  const colonIdx = str.indexOf(':');
  if (colonIdx > 0) {
    return {
      code: str.substring(0, colonIdx).trim(),
      message: str.substring(colonIdx + 1).trim(),
    };
  }
  return { code: 'UNKNOWN', message: str };
}

async function safeInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err) {
    throw parseError(err);
  }
}

/** Add files from absolute paths (drag-drop or file picker) */
export async function addFiles(paths: string[]): Promise<AddFilesResponse> {
  return safeInvoke('add_files', { paths });
}

/** Preview rename transformations without applying */
export async function previewRename(
  fileIds: string[],
  files: FileInfo[],
  pattern: RenamePattern
): Promise<PreviewResponse> {
  return safeInvoke('preview_rename', { fileIds, files, pattern });
}

/** Apply rename to files with backup and history recording */
export async function applyRename(
  fileIds: string[],
  files: FileInfo[],
  pattern: RenamePattern
): Promise<JobStartResponse> {
  return safeInvoke('apply_rename', { fileIds, files, pattern });
}

/** Undo a completed job, restoring original files from backup */
export async function undoJob(jobId: string): Promise<UndoResponse> {
  return safeInvoke('undo_job', { jobId });
}

/** Cancel a running job */
export async function cancelJob(jobId: string): Promise<boolean> {
  return safeInvoke('cancel_job', { jobId });
}

/** Get paginated job history with optional search */
export async function getJobHistory(
  limit = 50,
  offset = 0,
  search?: string
): Promise<HistoryResponse> {
  return safeInvoke('get_job_history', { limit, offset, search });
}

/** Get all app settings */
export async function getSettings(): Promise<Settings> {
  return safeInvoke('get_settings');
}

/** Update app settings (partial update) */
export async function updateSettings(
  settings: Partial<Settings>
): Promise<boolean> {
  // Convert Settings to simple key-value for Rust
  const kv: Record<string, string> = {};
  for (const [key, value] of Object.entries(settings)) {
    if (value !== null && value !== undefined) {
      kv[key] = typeof value === 'object' ? JSON.stringify(value) : String(value);
    }
  }
  return safeInvoke('update_settings', { settings: kv });
}

/** Open native file picker dialog */
export async function openFilePicker(): Promise<string[]> {
  return safeInvoke('open_file_picker');
}
