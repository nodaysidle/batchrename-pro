import { useState, useEffect, useCallback } from "react";
import { useAppState } from "@/state/AppStateContext";
import { getSettings, updateSettings } from "@/lib/commands";
import type { Settings } from "@/types";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const { dispatch } = useAppState();
  const [local, setLocal] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    getSettings()
      .then((res) => {
        setLocal(res.settings);
        dispatch({ type: "SET_SETTINGS", settings: res.settings });
      })
      .catch(() => {});
  }, [isOpen, dispatch]);

  const update = useCallback(
    async (patch: Partial<Settings>) => {
      if (!local) return;
      const next = { ...local, ...patch };
      setLocal(next);
      dispatch({ type: "SET_SETTINGS", settings: next });

      setSaving(true);
      try {
        await updateSettings(patch);
      } catch {
        // revert on failure
      } finally {
        setSaving(false);
      }
    },
    [local, dispatch],
  );

  if (!isOpen || !local) return null;

  const accentColors = [
    { name: "blue", color: "#3b82f6" },
    { name: "purple", color: "#8b5cf6" },
    { name: "green", color: "#22c55e" },
    { name: "orange", color: "#f97316" },
    { name: "pink", color: "#ec4899" },
    { name: "cyan", color: "#06b6d4" },
  ];

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="w-[440px] rounded-xl border border-border bg-bg-primary shadow-2xl">
        {/* Header */}
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h2 className="text-sm font-semibold text-text-primary">Settings</h2>
          <div className="flex items-center gap-2">
            {saving && (
              <span className="text-[10px] text-text-muted">Saving...</span>
            )}
            <button
              onClick={onClose}
              className="rounded-md p-1 text-text-secondary transition-colors hover:bg-bg-card hover:text-text-primary"
              aria-label="Close settings"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="flex flex-col gap-5 p-4">
          {/* Theme toggle */}
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-text-primary">Theme</p>
              <p className="text-xs text-text-muted">Switch between dark and light mode</p>
            </div>
            <button
              onClick={() =>
                update({ theme: local.theme === "dark" ? "light" : "dark" })
              }
              className="rounded-lg border border-border px-3 py-1.5 text-xs text-text-secondary transition-colors hover:border-accent hover:text-accent"
            >
              {local.theme === "dark" ? "Dark" : "Light"}
            </button>
          </div>

          {/* Accent color */}
          <div>
            <p className="mb-2 text-sm font-medium text-text-primary">Accent Color</p>
            <div className="flex gap-2">
              {accentColors.map((ac) => (
                <button
                  key={ac.name}
                  onClick={() => update({ accent_color: ac.name })}
                  className={`h-7 w-7 rounded-full border-2 transition-transform hover:scale-110 ${
                    local.accent_color === ac.name
                      ? "border-text-primary scale-110"
                      : "border-transparent"
                  }`}
                  style={{ backgroundColor: ac.color }}
                  aria-label={`${ac.name} accent color`}
                />
              ))}
            </div>
          </div>

          {/* Max parallel jobs */}
          <label className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-text-primary">Max Parallel Jobs</p>
              <p className="text-xs text-text-muted">0 = auto (use all cores)</p>
            </div>
            <input
              type="number"
              min={0}
              max={32}
              value={local.max_parallel_jobs}
              onChange={(e) =>
                update({ max_parallel_jobs: parseInt(e.target.value) || 0 })
              }
              className="w-16 rounded-lg border border-border bg-bg-card px-2 py-1 text-center text-sm text-text-primary focus:border-accent focus:outline-none"
            />
          </label>

          {/* Auto backup */}
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-text-primary">Auto Backup</p>
              <p className="text-xs text-text-muted">Create backups before operations</p>
            </div>
            <button
              onClick={() => update({ auto_backup: !local.auto_backup })}
              className={`relative h-6 w-11 rounded-full transition-colors ${
                local.auto_backup ? "bg-accent" : "bg-border"
              }`}
              role="switch"
              aria-checked={local.auto_backup}
            >
              <span
                className={`absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white transition-transform ${
                  local.auto_backup ? "translate-x-5" : ""
                }`}
              />
            </button>
          </div>

          {/* Backup retention */}
          <label className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-text-primary">Backup Retention</p>
              <p className="text-xs text-text-muted">Days to keep backup files</p>
            </div>
            <input
              type="number"
              min={1}
              max={365}
              value={local.backup_retention_days}
              onChange={(e) =>
                update({
                  backup_retention_days: parseInt(e.target.value) || 30,
                })
              }
              className="w-16 rounded-lg border border-border bg-bg-card px-2 py-1 text-center text-sm text-text-primary focus:border-accent focus:outline-none"
            />
          </label>

          {/* File hard cap */}
          <label className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-text-primary">File Limit</p>
              <p className="text-xs text-text-muted">Maximum files per batch</p>
            </div>
            <input
              type="number"
              min={100}
              max={10000}
              step={100}
              value={local.file_hard_cap}
              onChange={(e) =>
                update({ file_hard_cap: parseInt(e.target.value) || 5000 })
              }
              className="w-20 rounded-lg border border-border bg-bg-card px-2 py-1 text-center text-sm text-text-primary focus:border-accent focus:outline-none"
            />
          </label>
        </div>
      </div>
    </div>
  );
}
