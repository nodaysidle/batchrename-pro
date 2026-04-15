import { useCallback } from 'react';
import type { FileInfo } from '@/types';
import { Music, Image as ImageIcon, Film, FileText, X, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';

const TYPE_ICONS = {
  audio: Music,
  image: ImageIcon,
  video: Film,
  document: FileText,
};

const TYPE_COLORS = {
  audio: 'text-purple-400 bg-purple-500/10',
  image: 'text-emerald-400 bg-emerald-500/10',
  video: 'text-orange-400 bg-orange-500/10',
  document: 'text-slate-400 bg-slate-500/10',
};

const STATUS_STYLES = {
  pending: '',
  processing: 'ring-1 ring-yellow-400/30',
  done: 'ring-1 ring-emerald-400/30',
  error: 'ring-1 ring-red-400/30',
};

interface FileCardProps {
  file: FileInfo;
  onRemove: (id: string) => void;
}

export function FileCard({ file, onRemove }: FileCardProps) {
  const Icon = TYPE_ICONS[file.file_type];
  const colorClass = TYPE_COLORS[file.file_type];
  const statusClass = STATUS_STYLES[file.status];

  const handleRemove = useCallback(() => onRemove(file.id), [file.id, onRemove]);

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div
      className={`
        group flex items-center gap-3 p-3 rounded-xl
        bg-slate-800/40 border border-slate-700/30
        hover:bg-slate-800/60 hover:border-slate-600/40 hover:-translate-y-[1px]
        transition-all duration-200 ease-out
        ${statusClass}
      `}
    >
      {/* Thumbnail / Icon */}
      <div className={`flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center ${colorClass}`}>
        {file.thumbnail_data_url ? (
          <img
            src={file.thumbnail_data_url}
            alt=""
            className="w-10 h-10 rounded-lg object-cover"
          />
        ) : (
          <Icon className="w-5 h-5" />
        )}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <p className="text-sm text-slate-200 truncate font-medium">
          {file.original_name}
        </p>
        <div className="flex items-center gap-2 mt-0.5">
          {file.transformed_name && (
            <p className="text-xs text-[var(--accent)] truncate font-medium">
              → {file.transformed_name}
            </p>
          )}
          {!file.transformed_name && (
            <span className={`text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded-full font-medium ${colorClass}`}>
              {file.extension || file.file_type}
            </span>
          )}
        </div>
      </div>

      {/* Size */}
      <span className="text-xs text-slate-500 flex-shrink-0">
        {formatSize(file.size_bytes)}
      </span>

      {/* Status */}
      <div className="flex-shrink-0 w-6 h-6 flex items-center justify-center">
        {file.status === 'processing' && (
          <Loader2 className="w-4 h-4 text-yellow-400 animate-spin" />
        )}
        {file.status === 'done' && (
          <CheckCircle className="w-4 h-4 text-emerald-400" />
        )}
        {file.status === 'error' && (
          <AlertCircle className="w-4 h-4 text-red-400" />
        )}
      </div>

      {/* Remove button */}
      <button
        onClick={handleRemove}
        className="flex-shrink-0 w-6 h-6 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-200 text-slate-500 hover:text-red-400"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
