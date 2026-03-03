import { useState, useCallback, useMemo, useEffect } from "react";
import { useRenamePreview } from "@/hooks/useRenamePreview";
import { useAppState } from "@/state/AppStateContext";
import type { RenamePattern, RenameMode, CaseTransform } from "@/types";

const defaultPattern: RenamePattern = {
  mode: "regex",
  regex_find: "",
  regex_replace: "",
  template: "{original}",
  start_number: 1,
  zero_pad: 3,
  prefix: "",
  suffix: "",
  case_transform: "none",
};

const templateButtons = [
  { label: "{original}", desc: "Original name" },
  { label: "{number}", desc: "Sequence number" },
  { label: "{date}", desc: "Today's date" },
  { label: "{ext}", desc: "Extension" },
];

interface RenameTabProps {
  onPatternChange?: (pattern: RenamePattern | null) => void;
}

export default function RenameTab({ onPatternChange }: RenameTabProps) {
  const { state } = useAppState();
  const [pattern, setPattern] = useState<RenamePattern>(defaultPattern);
  const [showPreview, setShowPreview] = useState(true);

  // Propagate active pattern to parent for Apply button
  useEffect(() => {
    const active = (() => {
      switch (pattern.mode) {
        case "regex":
          return (pattern.regex_find ?? "").length > 0;
        case "template":
          return (pattern.template ?? "").length > 0;
        case "numbering":
          return true;
      }
    })();
    onPatternChange?.(active ? pattern : null);
  }, [pattern, onPatternChange]);

  const hasInput = useMemo(() => {
    switch (pattern.mode) {
      case "regex":
        return (pattern.regex_find ?? "").length > 0;
      case "template":
        return (pattern.template ?? "").length > 0;
      case "numbering":
        return true;
    }
  }, [pattern]);

  const activePattern = hasInput ? pattern : null;
  const { isLoading, error, totalConflicts } = useRenamePreview(activePattern);

  const updateField = useCallback(
    <K extends keyof RenamePattern>(key: K, value: RenamePattern[K]) => {
      setPattern((prev) => ({ ...prev, [key]: value }));
    },
    [],
  );

  const insertTemplate = useCallback((variable: string) => {
    setPattern((prev) => ({
      ...prev,
      template: (prev.template ?? "") + variable,
    }));
  }, []);

  return (
    <div className="flex flex-col gap-4">
      {/* Mode selector */}
      <div className="flex gap-1 rounded-lg bg-bg-primary p-1">
        {(["regex", "template", "numbering"] as RenameMode[]).map((mode) => (
          <button
            key={mode}
            onClick={() => updateField("mode", mode)}
            className={`flex-1 rounded-md px-2 py-1.5 text-xs font-medium capitalize transition-colors duration-200 ${
              pattern.mode === mode
                ? "bg-accent/15 text-accent"
                : "text-text-secondary hover:text-text-primary"
            }`}
          >
            {mode}
          </button>
        ))}
      </div>

      {/* Regex mode */}
      {pattern.mode === "regex" && (
        <>
          <label className="flex flex-col gap-1">
            <span className="text-xs font-medium text-text-secondary">Find (regex)</span>
            <input
              type="text"
              value={pattern.regex_find ?? ""}
              onChange={(e) => updateField("regex_find", e.target.value)}
              placeholder="e.g. photo_(\d+)"
              className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
              spellCheck={false}
            />
          </label>
          <label className="flex flex-col gap-1">
            <span className="text-xs font-medium text-text-secondary">Replace</span>
            <input
              type="text"
              value={pattern.regex_replace ?? ""}
              onChange={(e) => updateField("regex_replace", e.target.value)}
              placeholder="e.g. image_$1"
              className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
              spellCheck={false}
            />
          </label>
        </>
      )}

      {/* Template mode */}
      {pattern.mode === "template" && (
        <>
          <label className="flex flex-col gap-1">
            <span className="text-xs font-medium text-text-secondary">Template</span>
            <input
              type="text"
              value={pattern.template ?? ""}
              onChange={(e) => updateField("template", e.target.value)}
              placeholder="e.g. {original}_{number}"
              className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
              spellCheck={false}
            />
          </label>
          <div className="flex flex-wrap gap-1.5">
            {templateButtons.map((btn) => (
              <button
                key={btn.label}
                onClick={() => insertTemplate(btn.label)}
                title={btn.desc}
                className="rounded-md border border-border bg-bg-primary px-2 py-1 text-xs text-text-secondary transition-colors duration-200 hover:border-accent hover:text-accent"
              >
                {btn.label}
              </button>
            ))}
          </div>
        </>
      )}

      {/* Numbering mode */}
      {pattern.mode === "numbering" && (
        <>
          <div className="grid grid-cols-2 gap-3">
            <label className="flex flex-col gap-1">
              <span className="text-xs font-medium text-text-secondary">Start</span>
              <input
                type="number"
                min={0}
                value={pattern.start_number ?? 1}
                onChange={(e) => updateField("start_number", parseInt(e.target.value) || 1)}
                className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-xs font-medium text-text-secondary">Zero pad</span>
              <input
                type="number"
                min={1}
                max={10}
                value={pattern.zero_pad ?? 3}
                onChange={(e) => updateField("zero_pad", parseInt(e.target.value) || 1)}
                className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none"
              />
            </label>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <label className="flex flex-col gap-1">
              <span className="text-xs font-medium text-text-secondary">Prefix</span>
              <input
                type="text"
                value={pattern.prefix ?? ""}
                onChange={(e) => updateField("prefix", e.target.value)}
                placeholder="file_"
                className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
              />
            </label>
            <label className="flex flex-col gap-1">
              <span className="text-xs font-medium text-text-secondary">Suffix</span>
              <input
                type="text"
                value={pattern.suffix ?? ""}
                onChange={(e) => updateField("suffix", e.target.value)}
                placeholder="_final"
                className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
              />
            </label>
          </div>
        </>
      )}

      {/* Case transform (shared across modes) */}
      <label className="flex flex-col gap-1">
        <span className="text-xs font-medium text-text-secondary">Case</span>
        <select
          value={pattern.case_transform ?? "none"}
          onChange={(e) => updateField("case_transform", e.target.value as CaseTransform)}
          className="rounded-lg border border-border bg-bg-primary px-3 py-2 text-sm text-text-primary focus:border-accent focus:outline-none"
        >
          <option value="none">No change</option>
          <option value="upper">UPPERCASE</option>
          <option value="lower">lowercase</option>
          <option value="title">Title Case</option>
        </select>
      </label>

      {/* Error display */}
      {error && (
        <p className="rounded-md bg-error/10 px-3 py-2 text-xs text-error" role="alert">
          {error}
        </p>
      )}

      {/* Conflict warning */}
      {totalConflicts > 0 && (
        <p className="rounded-md bg-warning/10 px-3 py-2 text-xs text-warning">
          {totalConflicts} naming conflict{totalConflicts > 1 ? "s" : ""} detected
        </p>
      )}

      {/* Preview toggle */}
      <button
        onClick={() => setShowPreview(!showPreview)}
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
          className={`transition-transform duration-200 ${showPreview ? "rotate-90" : ""}`}
          aria-hidden="true"
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>
        Preview ({state.previews.length} files)
        {isLoading && <span className="text-accent">...</span>}
      </button>

      {/* Preview list */}
      {showPreview && state.previews.length > 0 && (
        <div className="max-h-60 overflow-y-auto rounded-lg border border-border bg-bg-primary">
          {state.previews.map((p) => (
            <div
              key={p.file_id}
              className="flex flex-col border-b border-border/50 px-3 py-1.5 last:border-b-0"
            >
              <span className="truncate text-xs text-text-muted">{p.original_name}</span>
              <span
                className={`truncate text-xs font-medium ${
                  p.has_conflict ? "text-error" : "text-accent"
                }`}
              >
                → {p.transformed_name}
                {p.has_conflict && " (conflict)"}
              </span>
            </div>
          ))}
        </div>
      )}

      {state.files.length === 0 && (
        <p className="text-center text-xs text-text-muted">
          Add files to see rename preview
        </p>
      )}
    </div>
  );
}
