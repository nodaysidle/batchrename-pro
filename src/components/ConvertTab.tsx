

export function ConvertTab() {
  return (
    <div className="flex flex-col gap-4 p-4">
      <div>
        <label className="text-xs text-slate-400 mb-1 block">Target format</label>
        <select className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all">
          <option value="">Select format...</option>
          <optgroup label="Audio">
            <option value="mp3">MP3</option>
            <option value="wav">WAV</option>
            <option value="flac">FLAC</option>
            <option value="m4a">M4A</option>
          </optgroup>
          <optgroup label="Image">
            <option value="jpg">JPG</option>
            <option value="png">PNG</option>
            <option value="webp">WebP</option>
            <option value="avif">AVIF</option>
          </optgroup>
          <optgroup label="Video">
            <option value="mp4">MP4</option>
            <option value="webm">WebM</option>
            <option value="mkv">MKV</option>
          </optgroup>
        </select>
      </div>

      <div>
        <div className="flex justify-between items-center mb-1">
          <label className="text-xs text-slate-400">Quality</label>
          <span className="text-xs text-slate-500">85%</span>
        </div>
        <input
          type="range"
          min={0}
          max={100}
          defaultValue={85}
          className="w-full h-1.5 bg-slate-700 rounded-full appearance-none cursor-pointer accent-[var(--accent)]"
        />
        <div className="flex justify-between text-[10px] text-slate-600 mt-0.5">
          <span>Smallest</span>
          <span>Best quality</span>
        </div>
      </div>

      <div className="pt-2 border-t border-slate-700/30">
        <p className="text-xs text-slate-500 text-center py-4">
          Conversion service coming in next build
        </p>
      </div>
    </div>
  );
}
