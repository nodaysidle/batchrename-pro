import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { JobProgressEvent, JobCompleteEvent, JobStartedEvent } from '@/types';

/** Subscribe to job-started events, emitted once a batch job has been created */
export function onJobStarted(
  callback: (payload: JobStartedEvent) => void
): Promise<UnlistenFn> {
  return listen<JobStartedEvent>('job_started', (event) => {
    callback(event.payload);
  });
}

/** Subscribe to per-file progress events during a job */
export function onJobProgress(
  callback: (payload: JobProgressEvent) => void
): Promise<UnlistenFn> {
  return listen<JobProgressEvent>('job_progress', (event) => {
    callback(event.payload);
  });
}

/** Subscribe to job completion event */
export function onJobComplete(
  callback: (payload: JobCompleteEvent) => void
): Promise<UnlistenFn> {
  return listen<JobCompleteEvent>('job_complete', (event) => {
    callback(event.payload);
  });
}
