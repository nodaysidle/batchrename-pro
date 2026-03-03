import { invoke } from "@tauri-apps/api/core";
import type {
  AddFilesResponse,
  CancelResponse,
  CleanupResponse,
  ConvertOptions,
  FileInfo,
  HistoryResponse,
  IpcError,
  JobDetail,
  JobStartResponse,
  MetadataChanges,
  MetadataResponse,
  PreviewResponse,
  RenamePattern,
  Settings,
  UndoResponse,
} from "@/types";

/** Parse Rust error string "ERROR_CODE: message" into structured error */
function parseError(error: unknown): IpcError {
  const str = String(error);
  const colonIdx = str.indexOf(":");
  if (colonIdx > 0 && colonIdx < 30) {
    return {
      code: str.slice(0, colonIdx).trim(),
      message: str.slice(colonIdx + 1).trim(),
    };
  }
  return { code: "UNKNOWN", message: str };
}

async function ipcCall<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw parseError(error);
  }
}

/** Add files from drag-drop or file picker */
export async function addFiles(paths: string[]): Promise<AddFilesResponse> {
  return ipcCall<AddFilesResponse>("add_files", { paths });
}

/** Generate rename preview without modifying files */
export async function previewRename(
  files: FileInfo[],
  pattern: RenamePattern,
): Promise<PreviewResponse> {
  return ipcCall<PreviewResponse>("preview_rename", { files, pattern });
}

/** Apply rename operation with backup creation */
export async function applyRename(
  files: FileInfo[],
  pattern: RenamePattern,
): Promise<JobStartResponse> {
  return ipcCall<JobStartResponse>("apply_rename", { files, pattern });
}

/** Convert files to target format */
export async function convertFiles(
  files: FileInfo[],
  options: ConvertOptions,
): Promise<JobStartResponse> {
  return ipcCall<JobStartResponse>("convert_files", { files, options });
}

/** Read metadata from selected files */
export async function readMetadata(files: FileInfo[]): Promise<MetadataResponse> {
  return ipcCall<MetadataResponse>("read_metadata", { files });
}

/** Write metadata changes to files */
export async function writeMetadata(
  files: FileInfo[],
  changes: MetadataChanges,
): Promise<JobStartResponse> {
  return ipcCall<JobStartResponse>("write_metadata", { files, changes });
}

/** Get paginated job history */
export async function getJobHistory(
  limit: number = 50,
  offset: number = 0,
  search?: string,
): Promise<HistoryResponse> {
  return ipcCall<HistoryResponse>("get_job_history", { limit, offset, search });
}

/** Get detailed job information */
export async function getJobDetail(jobId: string): Promise<JobDetail> {
  return ipcCall<JobDetail>("get_job_detail", { jobId });
}

/** Undo a completed job by restoring from backups */
export async function undoJob(jobId: string): Promise<UndoResponse> {
  return ipcCall<UndoResponse>("undo_job", { jobId });
}

/** Cancel an active job */
export async function cancelJob(jobId: string): Promise<CancelResponse> {
  return ipcCall<CancelResponse>("cancel_job", { jobId });
}

/** Get application settings */
export async function getSettings(): Promise<{ settings: Settings }> {
  return ipcCall<{ settings: Settings }>("get_settings");
}

/** Update application settings */
export async function updateSettings(settings: Partial<Settings>): Promise<void> {
  return ipcCall<void>("update_settings", { settings });
}

/** Open native file picker dialog */
export async function openFilePicker(
  filters?: Array<{ name: string; extensions: string[] }>,
): Promise<{ paths: string[] }> {
  return ipcCall<{ paths: string[] }>("open_file_picker", { filters });
}

/** Clean up old backup files */
export async function cleanupBackups(olderThanDays?: number): Promise<CleanupResponse> {
  return ipcCall<CleanupResponse>("cleanup_backups", { olderThanDays });
}
