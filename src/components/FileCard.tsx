import type { FileInfo, PreviewResult } from "@/types";

interface FileCardProps {
  file: FileInfo;
  preview?: PreviewResult;
  onRemove: (id: string) => void;
}

const typeIcons: Record<string, string> = {
  audio: "M9 18V5l12-2v13M9 18c0 1.1-1.3 2-3 2s-3-.9-3-2 1.3-2 3-2 3 .9 3 2zM21 16c0 1.1-1.3 2-3 2s-3-.9-3-2 1.3-2 3-2 3 .9 3 2z",
  image: "M4 4h16v16H4zM4 4l16 16M20 4L4 20",
  video: "M23 7l-7 5 7 5V7zM14 5H3c-1 0-2 1-2 2v10c0 1 1 2 2 2h11c1 0 2-1 2-2V7c0-1-1-2-2-2z",
  document: "M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2zM14 2v6h6",
};

const statusColors: Record<string, string> = {
  pending: "bg-text-muted",
  processing: "bg-warning animate-pulse",
  success: "bg-success",
  failed: "bg-error",
  skipped: "bg-text-muted",
};

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function FileCard({ file, preview, onRemove }: FileCardProps) {
  const transformedName = preview?.transformed_name;
  const hasConflict = preview?.has_conflict ?? false;

  return (
    <div className="group relative flex items-center gap-3 rounded-lg border border-border bg-bg-card px-3 py-2 transition-all duration-200 hover:-translate-y-[1px] hover:border-border-hover hover:shadow-md">
      {/* Thumbnail / type icon */}
      <div className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-bg-primary">
        {file.thumbnail_data_url ? (
          <img
            src={file.thumbnail_data_url}
            alt=""
            className="h-full w-full object-cover"
          />
        ) : (
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-text-muted"
            aria-hidden="true"
          >
            <path d={typeIcons[file.file_type] ?? typeIcons.document} />
          </svg>
        )}
      </div>

      {/* File info */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <p className="truncate text-sm font-medium text-text-primary">
            {file.original_name}
          </p>
          {/* Format badge */}
          <span className="shrink-0 rounded bg-bg-primary px-1.5 py-0.5 text-[10px] font-medium uppercase text-text-muted">
            {file.extension}
          </span>
        </div>

        {/* Transformed name preview */}
        {transformedName && (
          <p
            className={`truncate text-xs ${hasConflict ? "text-error" : "text-accent"}`}
          >
            → {transformedName}
            {hasConflict && " (conflict)"}
          </p>
        )}

        {!transformedName && (
          <p className="text-xs text-text-muted">
            {formatBytes(file.size_bytes)}
          </p>
        )}
      </div>

      {/* Status indicator */}
      <div
        className={`h-2 w-2 shrink-0 rounded-full ${statusColors[file.status] ?? statusColors.pending}`}
        aria-label={`Status: ${file.status}`}
      />

      {/* Remove button - visible on hover */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onRemove(file.id);
        }}
        className="absolute -right-1 -top-1 hidden rounded-full bg-error p-0.5 text-white opacity-0 transition-opacity duration-200 group-hover:block group-hover:opacity-100"
        aria-label={`Remove ${file.original_name}`}
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="3"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
        >
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  );
}
