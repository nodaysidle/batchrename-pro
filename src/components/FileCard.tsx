import { useCallback, type CSSProperties } from 'react';
import type { FileInfo, FileType, FileStatus, PreviewPair } from '@/types';
import { Music, Image as ImageIcon, Film, FileText, X, CheckCircle, AlertCircle, Loader2 } from 'lucide-react';

const TYPE_ICONS = {
  audio: Music,
  image: ImageIcon,
  video: Film,
  document: FileText,
};

const TYPE_STYLES: Record<FileType, CSSProperties> = {
  audio: { color: 'var(--type-audio)', backgroundColor: 'var(--type-audio-bg)' },
  image: { color: 'var(--type-image)', backgroundColor: 'var(--type-image-bg)' },
  video: { color: 'var(--type-video)', backgroundColor: 'var(--type-video-bg)' },
  document: { color: 'var(--type-document)', backgroundColor: 'var(--type-document-bg)' },
};

function statusRingStyle(status: FileStatus, hasConflict: boolean): CSSProperties {
  if (hasConflict) {
    return {
      boxShadow: 'inset 0 0 0 1px var(--danger-border)',
      borderColor: 'var(--danger-border)',
    };
  }
  switch (status) {
    case 'pending':
      return {};
    case 'processing':
      return { boxShadow: 'inset 0 0 0 1px color-mix(in srgb, var(--warning) 35%, transparent)' };
    case 'done':
      return { boxShadow: 'inset 0 0 0 1px color-mix(in srgb, var(--success) 35%, transparent)' };
    case 'error':
      return { boxShadow: 'inset 0 0 0 1px color-mix(in srgb, var(--danger) 40%, transparent)' };
    default: {
      const _exhaustive: never = status;
      return _exhaustive;
    }
  }
}

interface FileCardProps {
  file: FileInfo;
  preview?: PreviewPair;
  onRemove: (id: string) => void;
}

export function FileCard({ file, preview, onRemove }: FileCardProps) {
  const Icon = TYPE_ICONS[file.file_type];
  const typeStyle = TYPE_STYLES[file.file_type];
  const hasConflict = Boolean(preview?.has_conflict);
  const ringStyle = statusRingStyle(file.status, hasConflict);

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
      `}
      style={{
        backgroundColor: 'color-mix(in srgb, var(--card) 40%, transparent)',
        borderColor: 'var(--border)',
        ...ringStyle,
      }}
    >
      {/* Thumbnail / Icon */}
      <div
        className="flex-shrink-0 w-10 h-10 rounded-lg flex items-center justify-center"
        style={typeStyle}
      >
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
            <p
              className="text-xs truncate font-medium"
              style={{ color: hasConflict ? 'var(--danger-text)' : 'var(--accent)' }}
            >
              → {file.transformed_name}
            </p>
          )}
          {!file.transformed_name && (
            <span
              className="text-[10px] uppercase tracking-wider px-1.5 py-0.5 rounded-full font-medium"
              style={typeStyle}
            >
              {file.extension || file.file_type}
            </span>
          )}
        </div>
        {hasConflict && (
          <p className="text-[11px] truncate mt-0.5" style={{ color: 'var(--danger-text)' }}>
            {preview?.conflict_reason || 'Conflicting output'}
          </p>
        )}
        {file.status === 'error' && file.error && (
          <p
            className="text-[11px] truncate mt-0.5"
            style={{ color: 'var(--danger-text)' }}
            title={file.error}
          >
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
          <Loader2 className="w-4 h-4 animate-spin" style={{ color: 'var(--warning)' }} />
        )}
        {file.status === 'done' && (
          <CheckCircle className="w-4 h-4" style={{ color: 'var(--success)' }} />
        )}
        {file.status === 'error' && (
          <AlertCircle className="w-4 h-4" style={{ color: 'var(--danger)' }} />
        )}
        {file.status === 'pending' && hasConflict && (
          <AlertCircle className="w-4 h-4" style={{ color: 'var(--danger)' }} />
        )}
      </div>

      {/* Remove button */}
      <button
        onClick={handleRemove}
        aria-label={`Remove ${file.original_name}`}
        className="flex-shrink-0 w-6 h-6 flex items-center justify-center opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 transition-opacity duration-200 text-[var(--text-muted)] hover:text-[var(--danger)]"
      >
        <X className="w-4 h-4" />
      </button>
      {hasConflict && preview?.conflict_reason && (
        <span className="sr-only">{preview.conflict_reason}</span>
      )}
    </div>
  );
}
