

export function ConvertTab() {
  return (
    <div className="flex flex-col gap-3 p-4">
      <div className="rounded-xl border border-slate-700/50 bg-slate-800/30 p-4">
        <p className="text-sm font-medium text-slate-300">Conversion is coming soon</p>
        <p className="mt-1 text-xs leading-5 text-slate-500">
          This build is release-ready for safe batch renaming only. Format conversion is disabled until the local Rust media pipeline is complete.
        </p>
      </div>
    </div>
  );
}
