import { createContext, useContext, useReducer, type ReactNode } from 'react';
import { appReducer, initialState, type AppAction, type AppState } from './reducer';

export type { AppAction, AppState };

interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);
  return (
    <AppContext.Provider value={{ state, dispatch }}>
      {children}
    </AppContext.Provider>
  );
}

export function useAppState() {
  const ctx = useContext(AppContext);
  if (!ctx) throw new Error('useAppState must be used within AppStateProvider');
  return ctx;
}

export function useFileStats() {
  const { state } = useAppState();
  return {
    total: state.files.length,
    pending: state.files.filter((f) => f.status === 'pending').length,
    processing: state.files.filter((f) => f.status === 'processing').length,
    done: state.files.filter((f) => f.status === 'done').length,
    error: state.files.filter((f) => f.status === 'error').length,
  };
}

export function useCanApply() {
  const { state } = useAppState();
  const p = state.renamePattern;
  const hasPattern =
    (p.mode === 'regex' && p.regex_find.trim()) ||
    (p.mode === 'template' && p.template.trim()) ||
    (p.mode === 'numbering');
  const hasPreviewForEveryFile =
    state.files.length > 0 && state.previews.length === state.files.length;
  const hasConflicts = state.previews.some((preview) => preview.has_conflict);
  return (
    state.files.length > 0 &&
    !!hasPattern &&
    hasPreviewForEveryFile &&
    !hasConflicts &&
    !state.previewError &&
    !state.isProcessing
  );
}
