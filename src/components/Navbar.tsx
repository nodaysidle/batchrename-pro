import { useState } from 'react';
import { useTheme } from '@/contexts/ThemeContext';
import { Settings, Moon, Sun, HelpCircle, Zap, X } from 'lucide-react';

export function Navbar() {
  const { theme, toggleTheme, accentColor, setAccentColor } = useTheme();
  const [showSettings, setShowSettings] = useState(false);
  const [showHelp, setShowHelp] = useState(false);

  return (
    <nav className="sticky top-0 z-40 flex items-center justify-between px-6 h-12 bg-slate-900/80 border-b border-slate-700/30 backdrop-blur-xl">
      {/* Logo */}
      <div className="flex items-center gap-2">
        <Zap className="w-5 h-5 text-[var(--accent)]" />
        <span className="text-sm font-semibold text-slate-200 tracking-tight">
          BatchRename <span className="text-[var(--accent)]">Pro</span>
        </span>
      </div>

      {/* Right actions */}
      <div className="flex items-center gap-1">
        {/* Accent color dots */}
        <div className="flex gap-1.5 mr-2">
          {(['blue', 'violet'] as const).map((color) => (
            <button
              key={color}
              onClick={() => setAccentColor(color)}
              className={`
                w-4 h-4 rounded-full transition-all duration-200
                ${color === 'blue' ? 'bg-blue-500' : 'bg-violet-500'}
                ${accentColor === color ? 'scale-125' : 'opacity-40 hover:opacity-70 hover:scale-110'}
              `}
              style={
                accentColor === color
                  ? { boxShadow: `0 0 0 2px #0F172A, 0 0 0 3.5px ${color === 'blue' ? '#3B82F6' : '#A78BFA'}` }
                  : {}
              }
              aria-label={`${color} accent`}
            />
          ))}
        </div>

        {/* Theme toggle */}
        <button
          onClick={toggleTheme}
          className="p-2 rounded-lg text-slate-400 hover:text-slate-300 hover:bg-slate-800/50 transition-all duration-200"
          aria-label="Toggle theme"
        >
          {theme === 'dark' ? <Moon className="w-4 h-4" /> : <Sun className="w-4 h-4" />}
        </button>

        {/* Settings */}
        <button
          onClick={() => setShowSettings(!showSettings)}
          className="p-2 rounded-lg text-slate-400 hover:text-slate-300 hover:bg-slate-800/50 transition-all duration-200"
          aria-label="Appearance"
        >
          <Settings className="w-4 h-4" />
        </button>

        {/* Help */}
        <button
          onClick={() => setShowHelp(true)}
          className="p-2 rounded-lg text-slate-400 hover:text-slate-300 hover:bg-slate-800/50 transition-all duration-200"
          aria-label="Help"
        >
          <HelpCircle className="w-4 h-4" />
        </button>
      </div>

      {/* Settings dropdown */}
      {showSettings && (
        <div className="absolute top-12 right-6 w-64 bg-slate-800 border border-slate-700/50 rounded-xl shadow-2xl shadow-black/40 p-4 z-50">
          <h3 className="text-sm font-medium text-slate-200 mb-3">Appearance</h3>
          <div className="space-y-3">
            <div>
              <label className="text-xs text-slate-400 mb-1 block">Theme</label>
              <div className="flex gap-1">
                {(['dark', 'light'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => { if (theme !== t) toggleTheme(); }}
                    aria-label={`Use ${t} theme`}
                    className={`flex-1 py-1.5 rounded-lg text-xs font-medium transition-all ${
                      theme === t
                        ? 'bg-slate-700 text-slate-200'
                        : 'text-slate-500 hover:text-slate-400'
                    }`}
                  >
                    {t.charAt(0).toUpperCase() + t.slice(1)}
                  </button>
                ))}
              </div>
            </div>
            <div>
              <label className="text-xs text-slate-400 mb-1 block">Accent color</label>
              <div className="flex gap-2">
                {(['blue', 'violet'] as const).map((c) => (
                  <button
                    key={c}
                    onClick={() => setAccentColor(c)}
                    aria-label={`Use ${c} accent color`}
                    className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all ${
                      accentColor === c
                        ? 'bg-slate-700 text-slate-200'
                        : 'text-slate-500 hover:text-slate-400'
                    }`}
                  >
                    <span
                      className="w-2.5 h-2.5 rounded-full"
                      style={{ backgroundColor: c === 'blue' ? '#3B82F6' : '#A78BFA' }}
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
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 px-6">
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="help-title"
            className="w-full max-w-lg rounded-xl border border-slate-700/60 bg-slate-900 p-5 shadow-2xl shadow-black/50"
          >
            <div className="flex items-center justify-between gap-4">
              <h2 id="help-title" className="text-sm font-semibold text-slate-100">
                BatchRename Pro Help
              </h2>
              <button
                type="button"
                aria-label="Close help"
                onClick={() => setShowHelp(false)}
                className="rounded-lg p-2 text-slate-500 transition-colors hover:bg-slate-800 hover:text-slate-200"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            <div className="mt-4 space-y-3 text-sm leading-6 text-slate-300">
              <p>Add files with the picker or by dropping files onto the drop zone.</p>
              <p>Rename preview updates automatically as you edit regex, template, numbering, and case settings.</p>
              <p>Apply is disabled when the preview has duplicate names, invalid names, or an existing target file conflict.</p>
              <p>Before a rename runs, BatchRename Pro creates backups in the app data directory. Undo restores from those backups and removes only renamed outputs known to belong to that job.</p>
            </div>
          </div>
        </div>
      )}
    </nav>
  );
}
