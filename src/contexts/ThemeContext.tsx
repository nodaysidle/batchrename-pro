import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from 'react';
import { getSettings, parseError, updateSettings } from '@/lib/commands';
import { useAppState } from '@/state/AppStateContext';

interface ThemeContextValue {
  theme: 'dark' | 'light';
  accentColor: string;
  toggleTheme: () => void;
  setAccentColor: (color: string) => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

const ACCENT_MAP: Record<string, string> = {
  blue: '#3B82F6',
  violet: '#A78BFA',
};

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { dispatch } = useAppState();
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');
  const [accentColor, setAccent] = useState('blue');

  useEffect(() => {
    getSettings()
      .then((s) => {
        setTheme(s.theme);
        setAccent(s.accent_color);
      })
      .catch((err) => dispatch({ type: 'SET_ERROR', error: parseError(err) }));
  }, [dispatch]);

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    document.documentElement.style.setProperty('--accent', ACCENT_MAP[accentColor] ?? '#3B82F6');
  }, [theme, accentColor]);

  const toggleTheme = useCallback(() => {
    const next = theme === 'dark' ? 'light' : 'dark';
    setTheme(next);
    updateSettings({ theme: next }).catch((err) =>
      dispatch({ type: 'SET_ERROR', error: parseError(err) })
    );
  }, [dispatch, theme]);

  const setAccentColor = useCallback((color: string) => {
    setAccent(color);
    updateSettings({ accent_color: color as 'blue' | 'violet' }).catch((err) =>
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
