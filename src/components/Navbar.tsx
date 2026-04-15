import { useState } from 'react';
import { useTheme } from '@/contexts/ThemeContext';
import { Settings, Moon, Sun, HelpCircle, Zap } from 'lucide-react';

export function Navbar() {
  const { theme, toggleTheme, accentColor, setAccentColor } = useTheme();
  const [showSettings, setShowSettings] = useState(false);

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
          aria-label="Settings"
        >
          <Settings className="w-4 h-4" />
        </button>

        {/* Help */}
        <button
          className="p-2 rounded-lg text-slate-400 hover:text-slate-300 hover:bg-slate-800/50 transition-all duration-200"
          aria-label="Help"
        >
          <HelpCircle className="w-4 h-4" />
        </button>
      </div>

      {/* Settings dropdown */}
      {showSettings && (
        <div className="absolute top-12 right-6 w-64 bg-slate-800 border border-slate-700/50 rounded-xl shadow-2xl shadow-black/40 p-4 z-50">
          <h3 className="text-sm font-medium text-slate-200 mb-3">Settings</h3>
          <div className="space-y-3">
            <div>
              <label className="text-xs text-slate-400 mb-1 block">Theme</label>
              <div className="flex gap-1">
                {(['dark', 'light'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => { if (theme !== t) toggleTheme(); }}
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
    </nav>
  );
}
