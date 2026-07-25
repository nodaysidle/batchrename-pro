import { useEffect } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { onJobProgress, onJobComplete, onJobStarted } from '@/lib/events';

function basename(path: string): string {
  const parts = path.split(/[/\\]/);
  return parts[parts.length - 1] ?? path;
}

export function useTauriEvents() {
  const { dispatch } = useAppState();

  useEffect(() => {
    let unlistenStarted: (() => void) | undefined;
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;

    onJobStarted((event) => {
      dispatch({ type: 'START_JOB', jobId: event.job_id });
    }).then((fn) => (unlistenStarted = fn));

    onJobProgress((event) => {
      const status = event.status === 'completed' ? 'done' : event.status === 'failed' ? 'error' : 'processing';
      dispatch({
        type: 'UPDATE_FILE_STATUS',
        fileId: event.file_id,
        status,
        error: event.error_message ?? undefined,
        originalPath: status === 'done' ? event.transformed_path ?? undefined : undefined,
        originalName:
          status === 'done' && event.transformed_path ? basename(event.transformed_path) : undefined,
      });
    }).then((fn) => (unlistenProgress = fn));

    onJobComplete((event) => {
      dispatch({ type: 'COMPLETE_JOB', jobId: event.job_id });
    }).then((fn) => (unlistenComplete = fn));

    return () => {
      unlistenStarted?.();
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, [dispatch]);
}
