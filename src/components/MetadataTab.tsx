

export function MetadataTab() {
  return (
    <div className="flex flex-col gap-4 p-4">
      <div className="text-center py-8">
        <p className="text-sm text-slate-400">Select files to view metadata</p>
        <p className="text-xs text-slate-500 mt-1">
          ID3 tags for audio, EXIF for images
        </p>
      </div>

      <div className="pt-2 border-t border-slate-700/30">
        <button className="w-full py-2 px-4 bg-red-500/10 text-red-400 rounded-lg text-sm font-medium hover:bg-red-500/20 transition-all duration-200">
          Strip all metadata (EXIF + ID3)
        </button>
      </div>
    </div>
  );
}
