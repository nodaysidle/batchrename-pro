import {
  createContext,
  useContext,
  useReducer,
  useMemo,
  type ReactNode,
  type Dispatch,
} from "react";
import { appReducer, initialState, type AppState, type AppAction } from "./reducer";

interface AppStateContextValue {
  state: AppState;
  dispatch: Dispatch<AppAction>;
}

const AppStateContext = createContext<AppStateContextValue | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(appReducer, initialState);

  const value = useMemo(() => ({ state, dispatch }), [state]);

  return (
    <AppStateContext.Provider value={value}>
      {children}
    </AppStateContext.Provider>
  );
}

export function useAppState(): AppStateContextValue {
  const context = useContext(AppStateContext);
  if (!context) {
    throw new Error("useAppState must be used within AppStateProvider");
  }
  return context;
}

/** Memoized file statistics by status */
export function useFileStats() {
  const { state } = useAppState();

  return useMemo(() => {
    const stats = { total: 0, pending: 0, processing: 0, completed: 0, failed: 0 };
    for (const file of state.files) {
      stats.total++;
      switch (file.status) {
        case "pending": stats.pending++; break;
        case "processing": stats.processing++; break;
        case "success": stats.completed++; break;
        case "failed": stats.failed++; break;
      }
    }
    return stats;
  }, [state.files]);
}
