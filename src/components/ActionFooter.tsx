import { useCallback, useState } from 'react';
import { useAppState, useFileStats, useCanApply } from '@/state/AppStateContext';
import { applyRename as applyRenameCmd, undoJob as undoJobCmd, getJobHistory } from '@/lib/commands';
import { Play, Undo2, History, Loader2, CheckCircle, X } from 'lucide-react';

export function ActionFooter() {
  const { state, dispatch } = useAppState();
  const stats = useFileStats();
  const canApply = useCanApply();
  const [showHistory, setShowHistory] = useState(false);
  const [isApplying, setIsApplying] = useState(false);

  const handleApply = useCallback(async () => {
    if (!canApply) return;
    setIsApplying(true);

    try {
      const pattern = {
        ...state.renamePattern,
        mode: state.renamePattern.mode,
      };
      const fileIds = state.files.map((f) => f.id);
      const result = await applyRenameCmd(fileIds, state.files, pattern);
      dispatch({ type: 'START_JOB', jobId: result.job_id });
    } catch (err) {
      console.error('Apply failed:', err);
    } finally {
      setIsApplying(false);
    }
  }, [canApply, state.renamePattern, state.files, dispatch]);

  const handleUndo = useCallback(async () => {
    if (!state.lastCompletedJobId) return;
    try {
      const result = await undoJobCmd(state.lastCompletedJobId);
      if (result.success) {
        // Refresh files state — all done -> pending
        dispatch({ type: 'CLEAR_FILES' });
      }
    } catch (err) {
      console.error('Undo failed:', err);
    }
  }, [state.lastCompletedJobId, dispatch]);

  const handleShowHistory = useCallback(async () => {
    try {
      const result = await getJobHistory(20, 0);
      dispatch({ type: 'SET_HISTORY', history: result.jobs });
      setShowHistory(true);
    } catch (err) {
      console.error('History failed:', err);
    }
  }, [dispatch]);

  if (state.files.length === 0) return null;

  const progressPercent = state.isProcessing
    ? stats.total > 0
      ? ((stats.done + stats.error) / stats.total) * 100
      : 0
    : 0;

  return (
    <div className="sticky bottom-0 left-0 right-0 flex items-center gap-3 px-6 py-3 bg-slate-900/90 border-t border-slate-700/30 backdrop-blur-xl">
      {/* File counter */}
      <div className="flex items-center gap-2 text-sm text-slate-400">
        <span className="font-medium text-slate-200">{stats.total}</span>
        <span>file{stats.total !== 1 ? 's' : ''}</span>
        {state.isProcessing && (
          <span className="text-yellow-400 text-xs">
            {stats.done + stats.error}/{stats.total}
          </span>
        )}
      </div>

      {/* Progress bar (during processing) */}
      {state.isProcessing && (
        <div className="flex-1 h-1.5 bg-slate-800 rounded-full overflow-hidden">
          <div
            className="h-full bg-[var(--accent)] rounded-full transition-all duration-300 ease-out"
            style={{ width: `${progressPercent}%` }}
          />
        </div>
      )}

      {/* Spacer when not processing */}
      {!state.isProcessing && <div className="flex-1" />}

      {/* History button */}
      <button
        onClick={handleShowHistory}
        className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs text-slate-400 hover:text-slate-300 hover:bg-slate-800/50 transition-all duration-200"
      >
        <History className="w-3.5 h-3.5" />
        History
      </button>

      {/* Undo button */}
      {state.lastCompletedJobId && !state.isProcessing && (
        <button
          onClick={handleUndo}
          className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs text-slate-400 hover:text-amber-400 hover:bg-amber-500/10 transition-all duration-200"
        >
          <Undo2 className="w-3.5 h-3.5" />
          Undo
        </button>
      )}

      {/* Apply button */}
      <button
        onClick={handleApply}
        disabled={!canApply || isApplying}
        className={`
          flex items-center gap-2 px-5 py-2 rounded-xl text-sm font-medium
          transition-all duration-200 ease-out
          ${
            canApply && !isApplying
              ? 'bg-[var(--accent)] text-white hover:brightness-110 hover:shadow-lg hover:shadow-[var(--accent)]/20 active:scale-[0.98]'
              : 'bg-slate-700/50 text-slate-500 cursor-not-allowed'
          }
        `}
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

      {/* History dropdown */}
      {showHistory && (
        <div className="fixed bottom-16 right-6 w-80 max-h-96 overflow-y-auto bg-slate-800 border border-slate-700/50 rounded-xl shadow-2xl shadow-black/40 z-50">
          <div className="flex items-center justify-between p-3 border-b border-slate-700/30">
            <h3 className="text-sm font-medium text-slate-200">Job History</h3>
            <button
              onClick={() => setShowHistory(false)}
              className="text-slate-500 hover:text-slate-300"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
          <div className="p-2">
            {state.history.length === 0 ? (
              <p className="text-xs text-slate-500 text-center py-6">
                No jobs yet
              </p>
            ) : (
              state.history.map((job) => (
                <div
                  key={job.id}
                  className="flex items-center gap-3 p-2 rounded-lg hover:bg-slate-700/30 transition-colors"
                >
                  <div className="flex-shrink-0">
                    {job.status === 'completed' ? (
                      <CheckCircle className="w-4 h-4 text-emerald-400" />
                    ) : job.status === 'partial' ? (
                      <CheckCircle className="w-4 h-4 text-yellow-400" />
                    ) : (
                      <X className="w-4 h-4 text-red-400" />
                    )}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-xs text-slate-300 truncate">
                      {job.description}
                    </p>
                    <p className="text-[10px] text-slate-500">
                      {job.file_count} files · {job.operation_type}
                    </p>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}
