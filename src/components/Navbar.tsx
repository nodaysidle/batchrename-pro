import { useEffect, useRef, useState } from 'react';
import { useTheme } from '@/contexts/ThemeContext';
import { Palette, Moon, Sun, HelpCircle, Zap, X } from 'lucide-react';

const ACCENT_SWATCH: Record<'volt' | 'graphite', string> = {
  volt: '#C8FF00',
  graphite: '#6B6B80',
};

export function Navbar() {
  const { theme, toggleTheme, accentColor, setAccentColor } = useTheme();
  const [showSettings, setShowSettings] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const settingsTriggerRef = useRef<HTMLButtonElement>(null);
  const settingsMenuRef = useRef<HTMLDivElement>(null);
  const helpTriggerRef = useRef<HTMLButtonElement>(null);
  const helpDialogRef = useRef<HTMLDivElement>(null);
  const helpCloseRef = useRef<HTMLButtonElement>(null);

  const closeSettings = () => {
    setShowSettings(false);
    settingsTriggerRef.current?.focus();
  };

  const closeHelp = () => {
    setShowHelp(false);
    helpTriggerRef.current?.focus();
  };

  useEffect(() => {
    if (!showSettings) return;
    const handlePointer = (event: MouseEvent) => {
      if (
        settingsMenuRef.current &&
        !settingsMenuRef.current.contains(event.target as Node) &&
        !settingsTriggerRef.current?.contains(event.target as Node)
      ) {
        setShowSettings(false);
      }
    };
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        closeSettings();
      }
    };
    document.addEventListener('mousedown', handlePointer);
    document.addEventListener('keydown', handleKey);
    return () => {
      document.removeEventListener('mousedown', handlePointer);
      document.removeEventListener('keydown', handleKey);
    };
  }, [showSettings]);

  useEffect(() => {
    if (!showHelp) return;
    helpCloseRef.current?.focus();
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.stopPropagation();
        closeHelp();
      }
    };
    document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  }, [showHelp]);

  return (
    <nav
      className="sticky top-0 z-40 flex items-center justify-between px-6 h-12 border-b backdrop-blur-xl"
      style={{ backgroundColor: 'color-mix(in srgb, var(--card) 80%, transparent)', borderColor: 'var(--border)' }}
    >
      {/* Logo */}
      <div className="flex items-center gap-2">
        <Zap className="w-5 h-5 text-[var(--accent)]" />
        <span className="text-sm font-semibold tracking-tight" style={{ color: 'var(--text)' }}>
          BatchRename <span className="text-[var(--accent)]">Pro</span>
        </span>
      </div>

      {/* Right actions */}
      <div className="flex items-center gap-1">
        {/* Accent color dots */}
        <div className="flex gap-1.5 mr-2">
          {(['volt', 'graphite'] as const).map((color) => (
            <button
              key={color}
              onClick={() => setAccentColor(color)}
              className={`
                w-4 h-4 rounded-full transition-all duration-200
                ${accentColor === color ? 'scale-125' : 'opacity-40 hover:opacity-70 hover:scale-110'}
              `}
              style={{
                backgroundColor: ACCENT_SWATCH[color],
                ...(accentColor === color
                  ? { boxShadow: `0 0 0 2px var(--bg), 0 0 0 3.5px ${ACCENT_SWATCH[color]}` }
                  : {}),
              }}
              aria-label={`${color} accent`}
            />
          ))}
        </div>

        {/* Theme toggle */}
        <button
          onClick={toggleTheme}
          className="p-2 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] hover:bg-[var(--border)] transition-all duration-200"
          aria-label="Toggle theme"
        >
          {theme === 'dark' ? <Moon className="w-4 h-4" /> : <Sun className="w-4 h-4" />}
        </button>

        {/* Appearance */}
        <button
          ref={settingsTriggerRef}
          onClick={() => setShowSettings((prev) => !prev)}
          className="p-2 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] hover:bg-[var(--border)] transition-all duration-200"
          aria-label="Appearance"
          aria-expanded={showSettings}
          aria-controls="appearance-menu"
          title="Appearance"
        >
          <Palette className="w-4 h-4" />
        </button>

        {/* Help */}
        <button
          ref={helpTriggerRef}
          onClick={() => setShowHelp(true)}
          className="p-2 rounded-lg text-[var(--text-muted)] hover:text-[var(--text)] hover:bg-[var(--border)] transition-all duration-200"
          aria-label="Help"
          aria-expanded={showHelp}
          aria-controls="help-dialog"
        >
          <HelpCircle className="w-4 h-4" />
        </button>
      </div>

      {/* Appearance dropdown — theme/accent only; no full Settings yet */}
      {showSettings && (
        <div
          id="appearance-menu"
          ref={settingsMenuRef}
          role="dialog"
          aria-modal="true"
          aria-label="Appearance"
          className="absolute top-12 right-6 w-64 rounded-xl border p-4 z-50"
          style={{
            backgroundColor: 'var(--card)',
            borderColor: 'var(--border)',
            boxShadow: '0 25px 50px -12px var(--shadow)',
          }}
        >
          <h3 className="text-sm font-medium" style={{ color: 'var(--text)' }}>Appearance</h3>
          <p className="mt-1 mb-3 text-[11px] leading-4" style={{ color: 'var(--text-muted)' }}>
            Theme and accent only. Backup and job options are not configurable in this release.
          </p>
          <div className="space-y-3">
            <div>
              <label className="text-xs mb-1 block" style={{ color: 'var(--text-muted)' }}>Theme</label>
              <div className="flex gap-1">
                {(['dark', 'light'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => { if (theme !== t) toggleTheme(); }}
                    aria-label={`Use ${t} theme`}
                    className="flex-1 py-1.5 rounded-lg text-xs font-medium transition-all"
                    style={
                      theme === t
                        ? { backgroundColor: 'var(--border)', color: 'var(--text)' }
                        : { color: 'var(--text-muted)' }
                    }
                  >
                    {t.charAt(0).toUpperCase() + t.slice(1)}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label className="text-xs mb-1 block" style={{ color: 'var(--text-muted)' }}>Accent color</label>
              <div className="flex gap-2">
                {(['volt', 'graphite'] as const).map((c) => (
                  <button
                    key={c}
                    onClick={() => setAccentColor(c)}
                    aria-label={`Use ${c} accent color`}
                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all"
                    style={
                      accentColor === c
                        ? { backgroundColor: 'var(--border)', color: 'var(--text)' }
                        : { color: 'var(--text-muted)' }
                    }
                  >
                    <span
                      className="w-2.5 h-2.5 rounded-full"
                      style={{ backgroundColor: ACCENT_SWATCH[c] }}
                    />
                    {c.charAt(0).toUpperCase() + c.slice(1)}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {showHelp && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center px-6"
          style={{ backgroundColor: 'var(--overlay)' }}
          onMouseDown={closeHelp}
        >
          <div
            id="help-dialog"
            ref={helpDialogRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby="help-title"
            className="w-full max-w-lg rounded-xl border p-5"
            style={{
              backgroundColor: 'var(--card)',
              borderColor: 'var(--border)',
              boxShadow: '0 25px 50px -12px var(--shadow)',
            }}
            onMouseDown={(event) => event.stopPropagation()}
          >
            <div className="flex items-center justify-between gap-4">
              <h2 id="help-title" className="text-sm font-semibold" style={{ color: 'var(--text)' }}>
                BatchRename Pro Help
              </h2>
              <button
                ref={helpCloseRef}
                type="button"
                aria-label="Close help"
                onClick={closeHelp}
                className="rounded-lg p-2 transition-colors hover:bg-[var(--border)]"
                style={{ color: 'var(--text-muted)' }}
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            <div className="mt-4 space-y-3 text-sm leading-6" style={{ color: 'var(--text-muted)' }}>
              <p>Add files with the picker or by dropping files onto the drop zone.</p>
              <p>Rename preview updates automatically as you edit regex, template, numbering, and case settings.</p>
              <p>Apply is disabled when preview shows a collision (duplicate output names), an invalid name, or an external target that already exists. Occupancy chains and swaps among files in this batch are planned, not blocked.</p>
              <p>Before a rename runs, BatchRename Pro creates backups in the app data directory. Undo restores from those backups and removes only renamed outputs known to belong to that job.</p>
            </div>
          </div>
        </div>
      )}
    </nav>
  );
}
