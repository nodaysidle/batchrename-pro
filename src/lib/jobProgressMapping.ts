import type { FileInfo, JobProgressEvent } from '@/types';

export function basename(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] ?? path;
}

export type JobProgressFileUpdate = {
  fileId: string;
  status: FileInfo['status'];
  error?: string;
  originalPath?: string;
  originalName?: string;
};

/** Map a backend job_progress payload into UPDATE_FILE_STATUS fields. */
export function mapJobProgressToFileUpdate(event: JobProgressEvent): JobProgressFileUpdate {
  const status: FileInfo['status'] =
    event.status === 'completed' ? 'done' : event.status === 'failed' ? 'error' : 'processing';

  const trustedPath = event.transformed_path ?? undefined;
  const shouldTrustPath =
    trustedPath !== undefined && (status === 'done' || status === 'error');

  return {
    fileId: event.file_id,
    status,
    error: event.error_message ?? undefined,
    originalPath: shouldTrustPath ? trustedPath : undefined,
    originalName: shouldTrustPath ? basename(trustedPath) : undefined,
  };
}
