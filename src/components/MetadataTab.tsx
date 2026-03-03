import { useState, useCallback, useEffect } from "react";
import { useAppState } from "@/state/AppStateContext";
import { readMetadata } from "@/lib/commands";
import type { IpcError } from "@/types";

interface MetadataEntry {
  file_id: string;
  file_type: string;
  tags: Record<string, string>;
  has_exif: boolean;
  has_id3: boolean;
  raw_fields: Array<{ key: string; value: string; editable: boolean }>;
}

export default function MetadataTab() {
  const { state } = useAppState();
  const [metadata, setMetadata] = useState<MetadataEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadMetadata = useCallback(async () => {
    if (state.files.length === 0) {
      setMetadata([]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const response = await readMetadata(state.files);
      const data = response as unknown as { metadata: MetadataEntry[] };
      setMetadata(data.metadata ?? []);
    } catch (err: unknown) {
      const ipcErr = err as IpcError;
      setError(ipcErr.message ?? String(err));
    } finally {
      setLoading(false);
    }
  }, [state.files]);

  useEffect(() => {
    loadMetadata();
  }, [loadMetadata]);

  if (state.files.length === 0) {
    return (
      <p className="text-center text-xs text-text-muted">
        Add files to view metadata
      </p>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="h-5 w-5 animate-spin rounded-full border-2 border-accent border-t-transparent" />
      </div>
    );
  }

  if (error) {
    return (
      <p className="rounded-md bg-error/10 px-3 py-2 text-xs text-error" role="alert">
        {error}
      </p>
    );
  }

  const filesWithMetadata = metadata.filter(
    (m) => m.has_id3 || m.has_exif || m.raw_fields.length > 0,
  );

  if (filesWithMetadata.length === 0) {
    return (
      <div className="flex flex-col gap-3">
        <p className="text-center text-xs text-text-muted">
          No metadata found in selected files
        </p>
        <button
          onClick={loadMetadata}
          className="mx-auto rounded-lg border border-border px-4 py-1.5 text-xs text-text-secondary transition-colors duration-200 hover:border-accent hover:text-accent"
        >
          Refresh
        </button>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {/* Bulk actions */}
      <div className="flex gap-2">
        <button className="flex-1 rounded-lg border border-border bg-bg-primary px-3 py-1.5 text-xs text-text-secondary transition-colors duration-200 hover:border-warning hover:text-warning">
          Strip EXIF
        </button>
        <button className="flex-1 rounded-lg border border-border bg-bg-primary px-3 py-1.5 text-xs text-text-secondary transition-colors duration-200 hover:border-warning hover:text-warning">
          Strip ID3
        </button>
      </div>

      {/* Metadata display per file */}
      {filesWithMetadata.map((m) => {
        const file = state.files.find((f) => f.id === m.file_id);
        return (
          <div key={m.file_id} className="rounded-lg border border-border bg-bg-primary">
            <div className="border-b border-border/50 px-3 py-1.5">
              <p className="truncate text-xs font-medium text-text-primary">
                {file?.original_name ?? m.file_id}
              </p>
              <div className="flex gap-2 text-[10px] text-text-muted">
                {m.has_id3 && <span>ID3</span>}
                {m.has_exif && <span>EXIF</span>}
              </div>
            </div>
            <div className="max-h-48 overflow-y-auto">
              {m.raw_fields.map((field) => (
                <div
                  key={field.key}
                  className="flex items-center justify-between border-b border-border/30 px-3 py-1 last:border-b-0"
                >
                  <span className="text-[11px] text-text-muted">{field.key}</span>
                  {field.editable ? (
                    <input
                      type="text"
                      defaultValue={field.value}
                      className="w-32 rounded border border-transparent bg-transparent px-1 py-0.5 text-right text-[11px] text-text-primary focus:border-accent focus:outline-none"
                    />
                  ) : (
                    <span className="max-w-32 truncate text-[11px] text-text-secondary">
                      {field.value}
                    </span>
                  )}
                </div>
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
