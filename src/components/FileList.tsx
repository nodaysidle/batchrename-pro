import { useCallback } from 'react';
import { useAppState, useFileStats } from '@/state/AppStateContext';
import { FixedSizeList as List } from 'react-window';
import { FileCard } from './FileCard';
import { Trash2 } from 'lucide-react';

export function FileList() {
  const { state, dispatch } = useAppState();
  const stats = useFileStats();

  const handleRemove = useCallback(
    (id: string) => {
      dispatch({ type: 'REMOVE_FILE', id });
    },
    [dispatch]
  );

  const handleClearAll = useCallback(() => {
    dispatch({ type: 'CLEAR_FILES' });
  }, [dispatch]);

  if (state.files.length === 0) return null;

  return (
    <div className="flex flex-col gap-3">
      {/* Header */}
      <div className="flex items-center justify-between px-1">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-medium text-slate-300">
            {stats.total} file{stats.total !== 1 ? 's' : ''}
          </h3>
          <div className="flex gap-2 text-xs">
            {stats.pending > 0 && (
              <span className="text-slate-500">{stats.pending} pending</span>
            )}
            {stats.processing > 0 && (
              <span className="text-yellow-400 animate-pulse">
                {stats.processing} processing
              </span>
            )}
            {stats.done > 0 && (
              <span className="text-emerald-400">{stats.done} done</span>
            )}
            {stats.error > 0 && (
              <span className="text-red-400">{stats.error} error</span>
            )}
          </div>
        </div>
        <button
          onClick={handleClearAll}
          aria-label="Clear all files"
          className="flex items-center gap-1 text-xs text-slate-500 hover:text-red-400 transition-colors duration-200"
        >
          <Trash2 className="w-3 h-3" />
          Clear all
        </button>
      </div>

      {/* File list */}
      {state.files.length > 100 ? (
        <List
          height={320}
          itemCount={state.files.length}
          itemSize={72}
          width="100%"
          className="scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent"
        >
          {({ index, style }) => (
            <div style={style}>
              <FileCard
                file={state.files[index]!}
                preview={state.previews.find((preview) => preview.file_id === state.files[index]!.id)}
                onRemove={handleRemove}
              />
            </div>
          )}
        </List>
      ) : (
        <div className="flex flex-col gap-1 max-h-80 overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
          {state.files.map((file, i) => (
            <div
              key={file.id}
              style={{ animationDelay: `${i * 30}ms` }}
              className="animate-fade-in"
            >
              <FileCard
                file={file}
                preview={state.previews.find((preview) => preview.file_id === file.id)}
                onRemove={handleRemove}
              />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
