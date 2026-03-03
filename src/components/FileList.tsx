import { useMemo } from "react";
import { List } from "react-window";
import type { RowComponentProps } from "react-window";
import FileCard from "./FileCard";
import type { FileInfo, PreviewResult } from "@/types";

const ROW_HEIGHT = 56;
const VIRTUALIZE_THRESHOLD = 100;

interface FileListProps {
  files: FileInfo[];
  previews: PreviewResult[];
  onRemoveFile: (id: string) => void;
}

interface RowData {
  files: FileInfo[];
  previewMap: Map<string, PreviewResult>;
  onRemoveFile: (id: string) => void;
}

function Row({ index, style, ...rowProps }: RowComponentProps<RowData>) {
  const { files, previewMap, onRemoveFile } = rowProps as RowData;
  const file = files[index];
  return (
    <div style={style} className="px-2 py-1">
      <FileCard
        file={file}
        preview={previewMap.get(file.id)}
        onRemove={onRemoveFile}
      />
    </div>
  );
}

export default function FileList({ files, previews, onRemoveFile }: FileListProps) {
  const previewMap = useMemo(
    () => new Map(previews.map((p) => [p.file_id, p])),
    [previews],
  );

  if (files.length === 0) return null;

  // Use virtualization for large lists
  if (files.length >= VIRTUALIZE_THRESHOLD) {
    return (
      <div className="h-full w-full">
        <List
          rowComponent={Row}
          rowCount={files.length}
          rowHeight={ROW_HEIGHT}
          rowProps={{ files, previewMap, onRemoveFile }}
        />
      </div>
    );
  }

  // Simple list for small file counts
  return (
    <div className="flex h-full w-full flex-col gap-1 overflow-y-auto px-2 py-1">
      {files.map((file) => (
        <FileCard
          key={file.id}
          file={file}
          preview={previewMap.get(file.id)}
          onRemove={onRemoveFile}
        />
      ))}
    </div>
  );
}
