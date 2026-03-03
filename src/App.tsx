import { useState, useCallback, useMemo } from "react";
import Navbar from "@/components/Navbar";
import TransformationPanel from "@/components/TransformationPanel";
import ActionFooter from "@/components/ActionFooter";
import DropZone from "@/components/DropZone";
import FileList from "@/components/FileList";
import HistoryPanel from "@/components/HistoryPanel";
import SettingsModal from "@/components/SettingsModal";
import { AppStateProvider, useAppState, useFileStats } from "@/state/AppStateContext";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { applyRename, undoJob } from "@/lib/commands";
import type { JobProgressEvent, JobCompleteEvent, RenamePattern } from "@/types";

function AppContent() {
  const [panelOpen, setPanelOpen] = useState(true);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const { state, dispatch } = useAppState();
  const fileStats = useFileStats();

  // Store the current rename pattern from the RenameTab
  const [renamePattern, setRenamePattern] = useState<RenamePattern | null>(null);

  useTauriEvent<JobProgressEvent>("job_progress", (event) => {
    dispatch({ type: "UPDATE_FILE_STATUS", event });
  });

  useTauriEvent<JobCompleteEvent>("job_complete", (event) => {
    dispatch({ type: "COMPLETE_JOB", event });
  });

  const handleApply = useCallback(async () => {
    if (!renamePattern || state.files.length === 0) return;

    // Dispatch START_JOB before IPC so progress events are handled
    dispatch({
      type: "START_JOB",
      jobId: "pending",
      jobType: "rename",
      filesTotal: state.files.length,
    });

    try {
      await applyRename(state.files, renamePattern);
    } catch (err) {
      console.error("Apply failed:", err);
      // Clear active job on error
      dispatch({
        type: "COMPLETE_JOB",
        event: {
          job_id: "pending",
          status: "failed",
          files_completed: 0,
          files_failed: state.files.length,
          duration_ms: 0,
        },
      });
    }
  }, [state.files, renamePattern, dispatch]);

  const handleUndo = useCallback(async () => {
    if (!state.lastCompletedJobId) return;
    try {
      await undoJob(state.lastCompletedJobId);
    } catch (err) {
      console.error("Undo failed:", err);
    }
  }, [state.lastCompletedJobId]);

  const handleRemoveFile = useCallback(
    (fileId: string) => {
      dispatch({ type: "REMOVE_FILE", fileId });
    },
    [dispatch],
  );

  const canApply = fileStats.total > 0 && state.activeJob === null && renamePattern !== null;

  const progress = useMemo(() => {
    if (!state.activeJob) return 0;
    const { filesCompleted, filesFailed, filesTotal } = state.activeJob;
    if (filesTotal === 0) return 0;
    return ((filesCompleted + filesFailed) / filesTotal) * 100;
  }, [state.activeJob]);

  return (
    <div className="flex h-screen flex-col bg-bg-primary text-text-primary">
      <Navbar onSettingsClick={() => setSettingsOpen(true)} />

      <div className="relative flex min-h-0 flex-1">
        {/* Center canvas */}
        <main className="flex flex-1 flex-col p-4">
          {fileStats.total === 0 ? (
            <div className="flex flex-1 items-center justify-center">
              <DropZone />
            </div>
          ) : (
            <div className="flex flex-1 flex-col gap-3">
              <div className="shrink-0">
                <DropZone />
              </div>
              <div className="min-h-0 flex-1">
                <FileList
                  files={state.files}
                  previews={state.previews}
                  onRemoveFile={handleRemoveFile}
                />
              </div>
            </div>
          )}
        </main>

        {/* Right sidebar */}
        <TransformationPanel
          isOpen={panelOpen}
          onToggle={() => setPanelOpen(!panelOpen)}
          onPatternChange={setRenamePattern}
        />
      </div>

      <ActionFooter
        fileCount={fileStats.total}
        canApply={canApply}
        isProcessing={state.activeJob !== null}
        progress={progress}
        lastJobId={state.lastCompletedJobId}
        onApply={handleApply}
        onUndo={handleUndo}
        onShowHistory={() => setHistoryOpen(true)}
      />

      <HistoryPanel isOpen={historyOpen} onClose={() => setHistoryOpen(false)} />
      <SettingsModal isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
}

function App() {
  return (
    <AppStateProvider>
      <AppContent />
    </AppStateProvider>
  );
}

export default App;
