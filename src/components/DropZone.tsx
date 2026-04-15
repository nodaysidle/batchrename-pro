import { useCallback, useState, useEffect } from 'react';
import { addFiles as addFilesCmd } from '@/lib/commands';
import { useAppState } from '@/state/AppStateContext';
import { open } from '@tauri-apps/plugin-dialog';
import { FolderOpen, FileUp } from 'lucide-react';

export function DropZone() {
  const { dispatch } = useAppState();
  const [isDragging, setIsDragging] = useState(false);
  const [isLoading, setIsLoading] = useState(false);

  const handleFiles = useCallback(
    async (paths: string[]) => {
      if (!paths.length) return;
      setIsLoading(true);
      try {
        const result = await addFilesCmd(paths);
        dispatch({ type: 'ADD_FILES', files: result.files });
      } catch (err) {
        console.error('Failed to add files:', err);
      } finally {
        setIsLoading(false);
      }
    },
    [dispatch]
  );

  // Tauri native file drop — gives real filesystem paths
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    import('@tauri-apps/api/event').then(({ listen }) => {
      listen<string[]>('tauri://file-drop', (event) => {
        setIsDragging(false);
        handleFiles(event.payload);
      }).then((fn) => (unlisten = fn));
    });

    return () => unlisten?.();
  }, [handleFiles]);

  // Drag visual feedback
  useEffect(() => {
    let unlistenEnter: (() => void) | undefined;
    let unlistenLeave: (() => void) | undefined;

    import('@tauri-apps/api/event').then(({ listen }) => {
      listen('tauri://drag-enter', () => setIsDragging(true)).then(
        (fn) => (unlistenEnter = fn)
      );
      listen('tauri://drag-leave', () => setIsDragging(false)).then(
        (fn) => (unlistenLeave = fn)
      );
    });

    return () => {
      unlistenEnter?.();
      unlistenLeave?.();
    };
  }, []);

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
      console.error('File picker error:', err);
    }
  }, [handleFiles]);

  return (
    <div
      onClick={handleClick}
      className={`
        relative flex flex-col items-center justify-center gap-4
        rounded-2xl border-2 border-dashed p-12
        cursor-pointer transition-all duration-200 ease-out
        backdrop-blur-md
        ${
          isDragging
            ? 'border-[var(--accent)] bg-[var(--accent)]/10 scale-[1.02]'
            : 'border-slate-600/50 bg-white/[0.03] hover:border-slate-500/70 hover:bg-white/[0.05]'
        }
        ${isLoading ? 'opacity-60 pointer-events-none' : ''}
      `}
    >
      <div
        className={`
          p-4 rounded-full transition-all duration-200
          ${isDragging ? 'bg-[var(--accent)]/20 scale-110' : 'bg-slate-800/50'}
        `}
      >
        {isDragging ? (
          <FileUp className="w-8 h-8 text-[var(--accent)]" />
        ) : (
          <FolderOpen className="w-8 h-8 text-slate-400" />
        )}
      </div>

      <div className="text-center">
        <p className="text-slate-300 text-sm font-medium">
          {isDragging
            ? 'Release to add files'
            : isLoading
            ? 'Adding files...'
            : 'Drag files here or click to browse'}
        </p>
        <p className="text-slate-500 text-xs mt-1">
          Audio, Image, Video — up to 5,000 files
        </p>
      </div>
    </div>
  );
}
