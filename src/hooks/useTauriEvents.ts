import { useEffect } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { onJobProgress, onJobComplete, onJobStarted } from '@/lib/events';
import { mapJobProgressToFileUpdate } from '@/lib/jobProgressMapping';

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
      const update = mapJobProgressToFileUpdate(event);
      dispatch({
        type: 'UPDATE_FILE_STATUS',
        fileId: update.fileId,
        status: update.status,
        error: update.error,
        originalPath: update.originalPath,
        originalName: update.originalName,
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
