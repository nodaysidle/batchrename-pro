import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from 'react';
import { getSettings, parseError, updateSettings } from '@/lib/commands';
import { useAppState } from '@/state/AppStateContext';
import type { AccentColor } from '@/types';

interface ThemeContextValue {
  theme: 'dark' | 'light';
  accentColor: AccentColor;
  toggleTheme: () => void;
  setAccentColor: (color: AccentColor) => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

const ACCENT_MAP: Record<AccentColor, string> = {
  volt: '#C8FF00',
  graphite: '#6B6B80',
};

const ACCENT_RGB_MAP: Record<AccentColor, string> = {
  volt: '200, 255, 0',
  graphite: '107, 107, 128',
};

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { dispatch } = useAppState();
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [accentColor, setAccent] = useState<AccentColor>('volt');

  useEffect(() => {
    getSettings()
      .then((s) => {
        setTheme(s.theme);
        // Normalize stale persisted values to a valid AccentColor
        const accent: AccentColor =
          s.accent_color === 'volt' || s.accent_color === 'graphite'
            ? s.accent_color
            : 'volt';
        setAccent(accent);
      })
      .catch((err) => dispatch({ type: 'SET_ERROR', error: parseError(err) }));
  }, [dispatch]);

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    document.documentElement.style.setProperty('--accent', ACCENT_MAP[accentColor] ?? '#C8FF00');
    document.documentElement.style.setProperty(
      '--accent-rgb',
      ACCENT_RGB_MAP[accentColor] ?? '200, 255, 0'
    );
  }, [theme, accentColor]);

  const toggleTheme = useCallback(() => {
    const next = theme === 'dark' ? 'light' : 'dark';
    setTheme(next);
    updateSettings({ theme: next }).catch((err) =>
      dispatch({ type: 'SET_ERROR', error: parseError(err) })
    );
  }, [dispatch, theme]);

  const setAccentColor = useCallback((color: AccentColor) => {
    setAccent(color);
    updateSettings({ accent_color: color }).catch((err) =>
      dispatch({ type: 'SET_ERROR', error: parseError(err) })
    );
  }, [dispatch]);

  return (
    <ThemeContext.Provider value={{ theme, accentColor, toggleTheme, setAccentColor }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider');
  return ctx;
}
