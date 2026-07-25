import { useCallback, useEffect, useRef, useState } from 'react';
import { useAppState, useFileStats, useCanApply } from '@/state/AppStateContext';
import {
  applyRename as applyRenameCmd,
  undoJob as undoJobCmd,
  cancelJob as cancelJobCmd,
  getJobHistory,
  parseError,
} from '@/lib/commands';
import { Play, Undo2, History, Loader2, CheckCircle, X, Ban } from 'lucide-react';

export function ActionFooter() {
  const { state, dispatch } = useAppState();
  const stats = useFileStats();
  const canApply = useCanApply();
  const [showHistory, setShowHistory] = useState(false);
  const [isApplying, setIsApplying] = useState(false);
  const [isCancelling, setIsCancelling] = useState(false);
  const [undoingJobId, setUndoingJobId] = useState<string | null>(null);
  const conflictCount = state.previews.filter((preview) => preview.has_conflict).length;
  const historyTriggerRef = useRef<HTMLButtonElement>(null);
  const historyPanelRef = useRef<HTMLDivElement>(null);
  const hasFiles = state.files.length > 0;

  const handleApply = useCallback(async () => {
    if (!canApply) return;
    setIsApplying(true);
    dispatch({ type: 'CLEAR_ERROR' });
    dispatch({ type: 'SET_PROCESSING', isProcessing: true });

    try {
      const pattern = {
        ...state.renamePattern,
        mode: state.renamePattern.mode,
      };
      const fileIds = state.files.map((f) => f.id);
      const result = await applyRenameCmd(fileIds, state.files, pattern);
      // applyRename only resolves once the batch has fully run, but job_complete
      // may have raced or been missed — completing from the invoke result
      // guarantees isProcessing/lastCompletedJobId never get stuck.
      dispatch({ type: 'COMPLETE_JOB', jobId: result.job_id });
    } catch (err) {
      dispatch({ type: 'SET_PROCESSING', isProcessing: false });
      dispatch({ type: 'SET_ERROR', error: parseError(err) });
    } finally {
      setIsApplying(false);
    }
  }, [canApply, state.renamePattern, state.files, dispatch]);

  const refreshHistory = useCallback(async () => {
    try {
      const result = await getJobHistory(20, 0);
      dispatch({ type: 'SET_HISTORY', history: result.jobs });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', error: parseError(err) });
    }
  }, [dispatch]);

  const runUndo = useCallback(
    async (jobId: string) => {
      setUndoingJobId(jobId);
      try {
        const result = await undoJobCmd(jobId);
        if (result.success) {
          // Refresh files state — all done -> pending
          dispatch({ type: 'CLEAR_FILES' });
          await refreshHistory();
        } else {
          dispatch({
            type: 'SET_ERROR',
            error: {
              code: 'UNDO_PARTIAL',
              message: result.errors.map((error) => error.error).join('; ') || 'Undo could not restore every file',
            },
          });
        }
      } catch (err) {
        dispatch({ type: 'SET_ERROR', error: parseError(err) });
      } finally {
        setUndoingJobId(null);
      }
    },
    [dispatch, refreshHistory]
  );

  const handleUndo = useCallback(() => {
    if (!state.lastCompletedJobId) return;
    return runUndo(state.lastCompletedJobId);
  }, [state.lastCompletedJobId, runUndo]);

  const handleCancel = useCallback(async () => {
    if (!state.activeJobId) return;
    setIsCancelling(true);
    try {
      await cancelJobCmd(state.activeJobId);
    } catch (err) {
      dispatch({ type: 'SET_ERROR', error: parseError(err) });
    } finally {
      setIsCancelling(false);
    }
  }, [state.activeJobId, dispatch]);

  const handleShowHistory = useCallback(async () => {
    await refreshHistory();
    setShowHistory(true);
  }, [refreshHistory]);

  const closeHistory = useCallback(() => {
    setShowHistory(false);
    historyTriggerRef.current?.focus();
  }, []);

  useEffect(() => {
    if (!showHistory) return;
    const handlePointer = (event: MouseEvent) => {
      if (
        historyPanelRef.current &&
        !historyPanelRef.current.contains(event.target as Node) &&
        !historyTriggerRef.current?.contains(event.target as Node)
      ) {
        setShowHistory(false);
      }
    };
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        closeHistory();
      }
    };
    document.addEventListener('mousedown', handlePointer);
    document.addEventListener('keydown', handleKey);
    return () => {
      document.removeEventListener('mousedown', handlePointer);
      document.removeEventListener('keydown', handleKey);
    };
  }, [showHistory, closeHistory]);

  const progressPercent = state.isProcessing
    ? stats.total > 0
      ? ((stats.done + stats.error) / stats.total) * 100
      : 0
    : 0;

  const statusAnnouncement = state.isProcessing
    ? `Processing ${stats.done + stats.error} of ${stats.total} files`
    : state.lastCompletedJobId
    ? 'Rename job complete'
    : '';

  const historyPanel = showHistory ? (
    <div
      id="history-panel"
      ref={historyPanelRef}
      role="dialog"
      aria-modal="true"
      aria-label="Job history"
      className="fixed bottom-16 right-6 w-80 max-h-96 overflow-y-auto rounded-xl border z-50 scrollbar-thin"
      style={{
        backgroundColor: 'var(--card)',
        borderColor: 'var(--border)',
        boxShadow: '0 25px 50px -12px var(--shadow)',
      }}
    >
      <div className="flex items-center justify-between p-3 border-b" style={{ borderColor: 'var(--border)' }}>
        <h3 className="text-sm font-medium" style={{ color: 'var(--text)' }}>Job History</h3>
        <button
          onClick={closeHistory}
          aria-label="Close history"
          className="rounded-lg p-1 transition-colors hover:bg-[var(--border)]"
          style={{ color: 'var(--text-muted)' }}
        >
          <X className="w-4 h-4" />
        </button>
      </div>
      <div className="p-2">
        {state.history.length === 0 ? (
          <p className="text-xs text-center py-6" style={{ color: 'var(--text-muted)' }}>
            No jobs yet
          </p>
        ) : (
          state.history.map((job) => (
            <div
              key={job.id}
              className="flex items-center gap-3 p-2 rounded-lg transition-colors hover:bg-[var(--border)]"
            >
              <div className="flex-shrink-0">
                {job.status === 'completed' ? (
                  <CheckCircle className="w-4 h-4" style={{ color: 'var(--success)' }} />
                ) : job.status === 'partial' ? (
                  <CheckCircle className="w-4 h-4" style={{ color: 'var(--warning)' }} />
                ) : (
                  <X className="w-4 h-4" style={{ color: 'var(--danger)' }} />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-xs truncate" style={{ color: 'var(--text)' }}>
                  {job.description}
                </p>
                <p className="text-[10px]" style={{ color: 'var(--text-muted)' }}>
                  {job.file_count} files · {job.operation_type} ·{' '}
                  {new Date(job.timestamp).toLocaleString()}
                </p>
              </div>
              {job.can_undo && (
                <button
                  onClick={() => runUndo(job.id)}
                  disabled={state.isProcessing || undoingJobId === job.id}
                  aria-label={`Undo job from ${new Date(job.timestamp).toLocaleString()}`}
                  className="flex-shrink-0 flex items-center gap-1 px-2 py-1 rounded-md text-[11px] text-[var(--text-muted)] hover:text-[var(--undo)] hover:bg-[var(--undo-bg)] transition-all duration-200 disabled:opacity-50"
                >
                  <Undo2 className="w-3 h-3" />
                  Undo
                </button>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  ) : null;

  const historyButton = (
    <button
      ref={historyTriggerRef}
      onClick={handleShowHistory}
      aria-label="Open job history"
      aria-expanded={showHistory}
      aria-controls="history-panel"
      className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs transition-all duration-200 hover:bg-[var(--border)]"
      style={{ color: 'var(--text-muted)' }}
    >
      <History className="w-3.5 h-3.5" />
      History
    </button>
  );

  const undoButton =
    state.lastCompletedJobId && !state.isProcessing ? (
      <button
        onClick={handleUndo}
        disabled={undoingJobId === state.lastCompletedJobId}
        aria-label="Undo last completed rename job"
        className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs text-[var(--text-muted)] hover:text-[var(--undo)] hover:bg-[var(--undo-bg)] transition-all duration-200 disabled:opacity-50"
      >
        <Undo2 className="w-3.5 h-3.5" />
        Undo
      </button>
    ) : null;

  // Slim footer when no files — keep History (and Undo) reachable
  if (!hasFiles) {
    return (
      <div
        className="sticky bottom-0 left-0 right-0 flex items-center justify-end gap-3 px-6 py-2.5 border-t backdrop-blur-xl"
        style={{ backgroundColor: 'color-mix(in srgb, var(--card) 90%, transparent)', borderColor: 'var(--border)' }}
      >
        {historyButton}
        {undoButton}
        {historyPanel}
      </div>
    );
  }

  return (
    <div
      className="sticky bottom-0 left-0 right-0 flex items-center gap-3 px-6 py-3 border-t backdrop-blur-xl"
      style={{ backgroundColor: 'color-mix(in srgb, var(--card) 90%, transparent)', borderColor: 'var(--border)' }}
    >
      <span className="sr-only" role="status" aria-live="polite">
        {statusAnnouncement}
      </span>

      {/* File counter */}
      <div className="flex items-center gap-2 text-sm" style={{ color: 'var(--text-muted)' }}>
        <span className="font-medium" style={{ color: 'var(--text)' }}>{stats.total}</span>
        <span>file{stats.total !== 1 ? 's' : ''}</span>
        {state.isProcessing && (
          <span className="text-xs" style={{ color: 'var(--warning)' }}>
            {stats.done + stats.error}/{stats.total}
          </span>
        )}
      </div>

      {/* Progress bar (during processing) */}
      {state.isProcessing && (
        <div className="flex-1 h-1.5 rounded-full overflow-hidden" style={{ backgroundColor: 'var(--border)' }}>
          <div
            className="progress-fill h-full bg-[var(--accent)] rounded-full"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      )}

      {/* Spacer when not processing */}
      {!state.isProcessing && <div className="flex-1" />}

      {historyButton}

      {undoButton}

      {/* Cancel button (during processing) */}
      {state.isProcessing && state.activeJobId && (
        <button
          onClick={handleCancel}
          disabled={isCancelling}
          aria-label="Cancel in-progress rename job"
          className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs text-[var(--text-muted)] hover:text-[var(--danger)] hover:bg-[var(--danger-bg)] transition-all duration-200 disabled:opacity-50"
        >
          <Ban className="w-3.5 h-3.5" />
          Cancel
        </button>
      )}

      {/* Apply button */}
      <button
        onClick={handleApply}
        disabled={!canApply || isApplying}
        aria-label={
          conflictCount > 0
            ? 'Resolve rename conflicts before applying'
            : state.previewError
            ? 'Fix rename pattern before applying'
            : 'Apply rename'
        }
        className={`
          flex items-center gap-2 px-5 py-2 rounded-xl text-sm font-medium
          transition-all duration-200 ease-out
          ${
            canApply && !isApplying
              ? 'bg-[var(--accent)] text-white hover:brightness-110 hover:shadow-lg hover:shadow-[var(--accent)]/20 active:scale-[0.98]'
              : 'cursor-not-allowed'
          }
        `}
        style={
          canApply && !isApplying
            ? undefined
            : { backgroundColor: 'var(--border)', color: 'var(--text-muted)' }
        }
      >
        {isApplying ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : state.isProcessing ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <Play className="w-4 h-4" />
        )}
        {state.isProcessing ? 'Processing...' : 'Apply'}
      </button>
      {!canApply && !state.isProcessing && (
        <p className="max-w-56 text-[11px]" style={{ color: 'var(--text-muted)' }}>
          {conflictCount > 0
            ? 'Resolve conflicts first'
            : state.previewError
            ? 'Fix pattern first'
            : state.previews.length !== state.files.length
            ? 'Waiting for preview'
            : ''}
        </p>
      )}

      {historyPanel}
    </div>
  );
}
