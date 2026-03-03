import { useState, useEffect, useRef, useCallback } from "react";
import { previewRename } from "@/lib/commands";
import { useAppState } from "@/state/AppStateContext";
import type { RenamePattern, IpcError } from "@/types";

export function useRenamePreview(pattern: RenamePattern | null) {
  const { state, dispatch } = useAppState();
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [totalConflicts, setTotalConflicts] = useState(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const fetchPreview = useCallback(async () => {
    if (!pattern || state.files.length === 0) {
      dispatch({ type: "SET_PREVIEWS", previews: [] });
      setTotalConflicts(0);
      setError(null);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const response = await previewRename(state.files, pattern);
      dispatch({ type: "SET_PREVIEWS", previews: response.previews });
      setTotalConflicts(response.total_conflicts);
    } catch (err: unknown) {
      const ipcErr = err as IpcError;
      setError(ipcErr.message ?? String(err));
      dispatch({ type: "SET_PREVIEWS", previews: [] });
      setTotalConflicts(0);
    } finally {
      setIsLoading(false);
    }
  }, [pattern, state.files, dispatch]);

  // Debounce preview calls by 150ms
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(fetchPreview, 150);
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [fetchPreview]);

  return { isLoading, error, totalConflicts };
}
