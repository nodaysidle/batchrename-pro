import { useCallback } from 'react';
import type { FileInfo, PreviewPair } from '@/types';
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
  preview?: PreviewPair;
  onRemove: (id: string) => void;
}

export function FileCard({ file, preview, onRemove }: FileCardProps) {
  const Icon = TYPE_ICONS[file.file_type];
  const colorClass = TYPE_COLORS[file.file_type];
  const statusClass = preview?.has_conflict
    ? 'ring-1 ring-red-400/40 border-red-500/30'
    : STATUS_STYLES[file.status];

  const handleRemove = useCallback(() => onRemove(file.id), [file.id, onRemove]);

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div
      className={`
        group transition-default hover-lift flex items-center gap-3 p-3 rounded-xl border
        ${file.status === 'error' ? 'shake-error' : ''}
        ${statusClass}
      `}
      style={{ backgroundColor: 'color-mix(in srgb, var(--card) 40%, transparent)', borderColor: 'var(--border)' }}
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
        <p className="text-sm truncate font-medium" style={{ color: 'var(--text)' }}>
          {file.original_name}
        </p>
        <div className="flex items-center gap-2 mt-0.5">
          {file.transformed_name && (
            <p className={`text-xs truncate font-medium ${preview?.has_conflict ? 'text-red-300' : 'text-[var(--accent)]'}`}>
              → {file.transformed_name}
            </p>
          )}
          {!file.transformed_name && (
            <span className={`text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded-full font-medium ${colorClass}`}>
              {file.extension || file.file_type}
            </span>
          )}
        </div>
        {preview?.has_conflict && (
          <p className="text-[11px] text-red-300 truncate mt-0.5">
            {preview.conflict_reason || 'Conflicting output'}
          </p>
        )}
        {file.status === 'error' && file.error && (
          <p className="text-[11px] text-red-300 truncate mt-0.5" title={file.error}>
            {file.error}
          </p>
        )}
      </div>

      {/* Size */}
      <span className="text-xs flex-shrink-0" style={{ color: 'var(--text-muted)' }}>
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
        {file.status === 'pending' && preview?.has_conflict && (
          <AlertCircle className="w-4 h-4 text-red-400" />
        )}
      </div>

      {/* Remove button */}
      <button
        onClick={handleRemove}
        aria-label={`Remove ${file.original_name}`}
        className="flex-shrink-0 w-6 h-6 flex items-center justify-center opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 transition-opacity duration-200 hover:text-red-400"
        style={{ color: 'var(--text-muted)' }}
      >
        <X className="w-4 h-4" />
      </button>
      {preview?.has_conflict && (
        <span className="sr-only">{preview.conflict_reason}</span>
      )}
    </div>
  );
}
