import { useState, useCallback, useRef } from "react";
import { addFiles, openFilePicker } from "@/lib/commands";
import { useAppState } from "@/state/AppStateContext";

export default function DropZone() {
  const [isDragging, setIsDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const dragCounter = useRef(0);
  const { dispatch } = useAppState();

  const handleFiles = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) return;
      setError(null);

      try {
        const response = await addFiles(paths);
        dispatch({ type: "ADD_FILES", files: response.files });
      } catch (err: unknown) {
        const e = err as { message?: string };
        setError(e.message ?? "Failed to add files");
      }
    },
    [dispatch],
  );

  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current++;
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter.current--;
    if (dragCounter.current === 0) {
      setIsDragging(false);
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(
    async (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setIsDragging(false);
      dragCounter.current = 0;

      const paths: string[] = [];
      const items = e.dataTransfer.files;
      for (let i = 0; i < items.length; i++) {
        const file = items[i];
        // Tauri webview exposes file.path as a non-standard property
        const filePath = (file as unknown as Record<string, unknown>).path as string | undefined;
        if (filePath) {
          paths.push(filePath);
        }
      }

      await handleFiles(paths);
    },
    [handleFiles],
  );

  const handleClick = useCallback(async () => {
    setError(null);
    try {
      const response = await openFilePicker();
      await handleFiles(response.paths);
    } catch (err: unknown) {
      const e = err as { message?: string };
      setError(e.message ?? "Failed to open file picker");
    }
  }, [handleFiles]);

  return (
    <div
      onDragEnter={handleDragEnter}
      onDragLeave={handleDragLeave}
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      onClick={handleClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleClick();
        }
      }}
      aria-label="Drop files here or click to browse"
      className={`flex cursor-pointer flex-col items-center justify-center gap-4 rounded-2xl border-2 border-dashed p-12 backdrop-blur-md transition-all duration-200 ${
        isDragging
          ? "border-accent bg-accent/10 shadow-[0_0_30px_rgba(59,130,246,0.15)]"
          : "border-border bg-white/5 hover:border-border-hover hover:bg-white/[0.07]"
      }`}
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
        className={`transition-colors duration-200 ${isDragging ? "text-accent" : "text-text-muted"}`}
        aria-hidden="true"
      >
        <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
        <polyline points="14 2 14 8 20 8" />
        <path d="M12 12v6" />
        <path d="m15 15-3-3-3 3" />
      </svg>

      <div className="text-center">
        <p className={`text-lg transition-colors duration-200 ${isDragging ? "text-accent" : "text-text-secondary"}`}>
          {isDragging ? "Drop files here" : "Drag files here or click to browse"}
        </p>
        <p className="mt-1 text-sm text-text-muted">
          Audio, image, video, and document files
        </p>
      </div>

      {error && (
        <p className="text-sm text-error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
