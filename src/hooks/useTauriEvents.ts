import { useEffect } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { onJobProgress, onJobComplete } from '@/lib/events';

export function useTauriEvents() {
  const { dispatch } = useAppState();

  useEffect(() => {
    let unlistenProgress: (() => void) | undefined;
    let unlistenComplete: (() => void) | undefined;

    onJobProgress((event) => {
      dispatch({
        type: 'UPDATE_FILE_STATUS',
        fileId: event.file_id,
        status: event.status === 'completed' ? 'done' : event.status === 'failed' ? 'error' : 'processing',
        error: event.error_message ?? undefined,
      });
    }).then((fn) => (unlistenProgress = fn));

    onJobComplete((event) => {
      dispatch({ type: 'COMPLETE_JOB', jobId: event.job_id });
    }).then((fn) => (unlistenComplete = fn));

    return () => {
      unlistenProgress?.();
      unlistenComplete?.();
    };
  }, [dispatch]);
}
