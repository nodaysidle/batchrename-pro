import { useState, useMemo } from "react";
import { useAppState } from "@/state/AppStateContext";

const formatsByType: Record<string, string[]> = {
  audio: ["mp3", "wav", "flac", "m4a"],
  image: ["jpg", "png", "webp", "avif"],
  video: ["mp4", "webm", "mkv"],
};

export default function ConvertTab() {
  const { state } = useAppState();
  const [targetFormat, setTargetFormat] = useState("");
  const [quality, setQuality] = useState(85);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const availableFormats = useMemo(() => {
    if (state.files.length === 0) return [];

    // Determine the dominant file type
    const typeCounts: Record<string, number> = {};
    for (const f of state.files) {
      typeCounts[f.file_type] = (typeCounts[f.file_type] || 0) + 1;
    }
    const dominantType = Object.entries(typeCounts).sort((a, b) => b[1] - a[1])[0]?.[0];
    if (!dominantType) return [];

    const formats = formatsByType[dominantType] ?? [];
    // Filter out the current extensions
    const currentExts = new Set(state.files.map((f) => f.extension.toLowerCase()));
    return formats.filter((f) => !currentExts.has(f));
  }, [state.files]);

  if (state.files.length === 0) {
    return (
      <p className="text-center text-xs text-text-muted">
        Add files to see conversion options
      </p>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      {/* Target format */}
      <label className="flex flex-col gap-1">
        <span className="text-xs font-medium text-text-secondary">Target Format</span>
        <select
          value={targetFormat}
          onChange={(e) => setTargetFormat(e.target.value)}
          className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none"
        >
          <option value="">Select format...</option>
          {availableFormats.map((fmt) => (
            <option key={fmt} value={fmt}>
              {fmt.toUpperCase()}
            </option>
          ))}
        </select>
      </label>

      {/* Quality slider */}
      <label className="flex flex-col gap-1">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-text-secondary">Quality</span>
          <span className="text-xs text-text-muted">{quality}%</span>
        </div>
        <input
          type="range"
          min={1}
          max={100}
          value={quality}
          onChange={(e) => setQuality(parseInt(e.target.value))}
          className="w-full accent-accent"
        />
        <div className="flex justify-between text-[10px] text-text-muted">
          <span>Smaller</span>
          <span>Better</span>
        </div>
      </label>

      {/* Advanced options toggle */}
      <button
        onClick={() => setShowAdvanced(!showAdvanced)}
        className="flex items-center gap-2 text-xs text-text-secondary transition-colors duration-200 hover:text-text-primary"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="12"
          height="12"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={`transition-transform duration-200 ${showAdvanced ? "rotate-90" : ""}`}
          aria-hidden="true"
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>
        Advanced Options
      </button>

      {showAdvanced && (
        <div className="flex flex-col gap-3 rounded-lg border border-border/50 bg-bg-primary p-3">
          <p className="text-xs text-text-muted">
            Codec selection, bitrate, and resize options will be available when format conversion pipeline is fully wired.
          </p>
        </div>
      )}

      {!targetFormat && (
        <p className="text-center text-xs text-text-muted">
          Select a target format to enable conversion
        </p>
      )}
    </div>
  );
}
