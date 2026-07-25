import { useCallback, useState, useEffect } from 'react';
import { addFiles as addFilesCmd, parseError } from '@/lib/commands';
import { useAppState } from '@/state/AppStateContext';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { FolderOpen, FileUp, Plus } from 'lucide-react';

export function DropZone() {
  const { state, dispatch } = useAppState();
  const [isDragging, setIsDragging] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const hasFiles = state.files.length > 0;

  const handleFiles = useCallback(
    async (paths: string[]) => {
      if (!paths.length) return;
      setIsLoading(true);
      try {
        const result = await addFilesCmd(paths);
        dispatch({ type: 'ADD_FILES', files: result.files });
      } catch (err) {
        dispatch({ type: 'SET_ERROR', error: parseError(err) });
      } finally {
        setIsLoading(false);
      }
    },
    [dispatch]
  );

  // Tauri 2 native file drop — gives real filesystem paths.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let mounted = true;

    getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          setIsDragging(true);
          return;
        }
        if (event.payload.type === 'leave') {
          setIsDragging(false);
          return;
        }
        setIsDragging(false);
        handleFiles(event.payload.paths);
      })
      .then((fn) => {
        if (mounted) {
          unlisten = fn;
        } else {
          fn();
        }
      })
      .catch((err) => dispatch({ type: 'SET_ERROR', error: parseError(err) }));

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [dispatch, handleFiles]);

  // Click to browse — Tauri native dialog
  const handleClick = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [
          {
            name: 'All Supported',
            extensions: ['mp3', 'wav', 'flac', 'm4a', 'jpg', 'jpeg', 'png', 'webp', 'avif', 'mp4', 'webm', 'mkv'],
          },
          { name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'm4a'] },
          { name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'webp', 'avif'] },
          { name: 'Video', extensions: ['mp4', 'webm', 'mkv'] },
        ],
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        handleFiles(paths);
      }
    } catch (err) {
      dispatch({ type: 'SET_ERROR', error: parseError(err) });
    }
  }, [dispatch, handleFiles]);

  if (hasFiles) {
    return (
      <div
        onClick={handleClick}
        role="button"
        tabIndex={0}
        aria-label="Add more files"
        onKeyDown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            handleClick();
          }
        }}
        className={`
          transition-default hover-glow flex items-center justify-between gap-3
          rounded-xl border border-dashed p-3
          cursor-pointer backdrop-blur-md
          ${isDragging ? 'border-[var(--accent)] pulse-glow' : ''}
          ${isLoading ? 'opacity-60 pointer-events-none' : ''}
        `}
        style={{
          borderColor: isDragging ? 'var(--accent)' : 'var(--border)',
          backgroundColor: isDragging
            ? 'color-mix(in srgb, var(--accent) 10%, transparent)'
            : 'color-mix(in srgb, var(--card) 30%, transparent)',
        }}
      >
        <div className="flex items-center gap-2">
          <Plus className="w-4 h-4" style={{ color: isDragging ? 'var(--accent)' : 'var(--text-muted)' }} />
          <p className="text-xs font-medium" style={{ color: 'var(--text-muted)' }}>
            {isDragging ? 'Release to add files' : isLoading ? 'Adding files...' : 'Add more files'}
          </p>
        </div>
        <p className="text-[11px]" style={{ color: 'var(--text-muted)' }}>Drag & drop supported</p>
      </div>
    );
  }

  return (
    <div
      onClick={handleClick}
      role="button"
      tabIndex={0}
      aria-label="Add files"
      onKeyDown={(event) => {
        if (event.key === 'Enter' || event.key === ' ') {
          event.preventDefault();
          handleClick();
        }
      }}
      className={`
        transition-default relative flex flex-col items-center justify-center gap-4
        rounded-2xl border-2 border-dashed p-12
        cursor-pointer backdrop-blur-md
        ${isDragging ? 'scale-[1.02] pulse-glow' : ''}
        ${isLoading ? 'opacity-60 pointer-events-none' : ''}
      `}
      style={{
        borderColor: isDragging ? 'var(--accent)' : 'var(--border)',
        backgroundColor: isDragging
          ? 'color-mix(in srgb, var(--accent) 10%, transparent)'
          : 'color-mix(in srgb, var(--card) 15%, transparent)',
      }}
    >
      <div
        className={`transition-default p-4 rounded-full ${isDragging ? 'scale-110' : ''}`}
        style={{
          backgroundColor: isDragging
            ? 'color-mix(in srgb, var(--accent) 20%, transparent)'
            : 'color-mix(in srgb, var(--card) 60%, transparent)',
        }}
      >
        {isDragging ? (
          <FileUp className="w-8 h-8 text-[var(--accent)]" />
        ) : (
          <FolderOpen className="w-8 h-8" style={{ color: 'var(--text-muted)' }} />
        )}
      </div>

      <div className="text-center">
        <p className="text-sm font-medium" style={{ color: 'var(--text)' }}>
          {isDragging
            ? 'Release to add files'
            : isLoading
            ? 'Adding files...'
            : 'Drag files here or click to browse'}
        </p>
        <p className="text-xs mt-1" style={{ color: 'var(--text-muted)' }}>
          Audio, Image, Video — up to 5,000 files
        </p>
      </div>
    </div>
  );
}
