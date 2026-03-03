interface ActionFooterProps {
  fileCount: number;
  canApply: boolean;
  isProcessing: boolean;
  progress: number; // 0-100
  lastJobId: string | null;
  onApply: () => void;
  onUndo: () => void;
  onShowHistory: () => void;
}

export default function ActionFooter({
  fileCount,
  canApply,
  isProcessing,
  progress,
  lastJobId,
  onApply,
  onUndo,
  onShowHistory,
}: ActionFooterProps) {
  return (
    <footer className="flex h-14 shrink-0 items-center justify-between border-t border-border bg-bg-card px-4 no-select">
      {/* Left: file counter */}
      <span className="text-sm text-text-secondary">
        {fileCount === 0
          ? "No files loaded"
          : `${fileCount} file${fileCount !== 1 ? "s" : ""} ready`}
      </span>

      {/* Center: Apply button with progress */}
      <div className="flex items-center gap-3">
        {lastJobId && !isProcessing && (
          <button
            onClick={onUndo}
            className="rounded-lg border border-border px-4 py-2 text-sm font-medium text-text-secondary transition-all duration-200 hover:border-warning hover:text-warning"
            aria-label="Undo last operation"
          >
            Undo
          </button>
        )}
        <button
          onClick={onApply}
          disabled={!canApply || isProcessing}
          className={`relative overflow-hidden rounded-lg px-6 py-2 text-sm font-medium text-white transition-all duration-200 ${
            canApply && !isProcessing
              ? "bg-accent hover:bg-accent-hover active:scale-[0.98]"
              : "cursor-not-allowed bg-accent opacity-50"
          }`}
          aria-label="Apply transformation"
        >
          {/* Progress fill */}
          {isProcessing && (
            <div
              className="absolute inset-0 bg-white/20 transition-all duration-300 ease-out"
              style={{ width: `${progress}%` }}
            />
          )}
          <span className="relative z-10">
            {isProcessing ? (
              <span className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 animate-spin"
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                {Math.round(progress)}%
              </span>
            ) : (
              "Apply"
            )}
          </span>
        </button>
      </div>

      {/* Right: History */}
      <div className="flex items-center gap-2">
        <button
          onClick={onShowHistory}
          className="rounded-lg p-2 text-text-secondary transition-colors duration-200 hover:bg-bg-card-hover hover:text-text-primary"
          aria-label="View job history"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
          </svg>
        </button>
      </div>
    </footer>
  );
}
