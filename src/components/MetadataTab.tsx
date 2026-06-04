

export function MetadataTab() {
  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="rounded-xl border border-slate-700/50 bg-slate-800/30 p-4">
        <p className="text-sm font-medium text-slate-300">Metadata editing is coming soon</p>
        <p className="mt-1 text-xs leading-5 text-slate-500">
          This build does not read, write, or strip metadata. Those controls are disabled until the local ID3/EXIF implementation is complete.
        </p>
      </div>
    </div>
  );
}
