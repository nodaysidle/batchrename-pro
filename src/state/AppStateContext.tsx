import { createContext, useContext, useReducer, type ReactNode } from 'react';
import type { FileInfo, PreviewPair, JobSummary, Settings, TabType } from '@/types';

// --- State ---

export interface AppState {
  files: FileInfo[];
  previews: PreviewPair[];
  activeJobId: string | null;
  isProcessing: boolean;
  activeTab: TabType;
  history: JobSummary[];
  settings: Settings | null;
  lastCompletedJobId: string | null;
  renamePattern: {
    mode: 'regex' | 'template' | 'numbering';
    regex_find: string;
    regex_replace: string;
    template: string;
    start_number: number;
    zero_pad: number;
    prefix: string;
    suffix: string;
    case_transform: 'none' | 'upper' | 'lower' | 'title';
  };
}

const initialState: AppState = {
  files: [],
  previews: [],
  activeJobId: null,
  isProcessing: false,
  activeTab: 'rename',
  history: [],
  settings: null,
  lastCompletedJobId: null,
  renamePattern: {
    mode: 'regex',
    regex_find: '',
    regex_replace: '',
    template: '{original}_{number}',
    start_number: 1,
    zero_pad: 3,
    prefix: '',
    suffix: '',
    case_transform: 'none',
  },
};

// --- Actions ---

type AppAction =
  | { type: 'ADD_FILES'; files: FileInfo[] }
  | { type: 'REMOVE_FILE'; id: string }
  | { type: 'CLEAR_FILES' }
  | { type: 'SET_PREVIEWS'; previews: PreviewPair[] }
  | { type: 'START_JOB'; jobId: string }
  | { type: 'UPDATE_FILE_STATUS'; fileId: string; status: FileInfo['status']; transformedName?: string; error?: string }
  | { type: 'COMPLETE_JOB'; jobId: string }
  | { type: 'SET_ACTIVE_TAB'; tab: TabType }
  | { type: 'SET_HISTORY'; history: JobSummary[] }
  | { type: 'SET_SETTINGS'; settings: Settings }
  | { type: 'SET_RENAME_PATTERN'; pattern: Partial<AppState['renamePattern']> }
  | { type: 'SET_PROCESSING'; isProcessing: boolean };

function reducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'ADD_FILES':
      return { ...state, files: [...state.files, ...action.files] };

    case 'REMOVE_FILE':
      return {
        ...state,
        files: state.files.filter((f) => f.id !== action.id),
        previews: state.previews.filter((p) => p.file_id !== action.id),
      };

    case 'CLEAR_FILES':
      return { ...state, files: [], previews: [], lastCompletedJobId: null };

    case 'SET_PREVIEWS':
      return {
        ...state,
        previews: action.previews,
        files: state.files.map((f) => {
          const preview = action.previews.find((p) => p.file_id === f.id);
          return preview
            ? { ...f, transformed_name: preview.transformed_name }
            : { ...f, transformed_name: null };
        }),
      };

    case 'START_JOB':
      return { ...state, activeJobId: action.jobId, isProcessing: true, lastCompletedJobId: null };

    case 'UPDATE_FILE_STATUS':
      return {
        ...state,
        files: state.files.map((f) =>
          f.id === action.fileId
            ? {
                ...f,
                status: action.status,
                transformed_name: action.transformedName ?? f.transformed_name,
                error: action.error ?? null,
              }
            : f
        ),
      };

    case 'COMPLETE_JOB':
      return {
        ...state,
        activeJobId: null,
        isProcessing: false,
        lastCompletedJobId: action.jobId,
      };

    case 'SET_ACTIVE_TAB':
      return { ...state, activeTab: action.tab };

    case 'SET_HISTORY':
      return { ...state, history: action.history };

    case 'SET_SETTINGS':
      return { ...state, settings: action.settings };

    case 'SET_RENAME_PATTERN':
      return { ...state, renamePattern: { ...state.renamePattern, ...action.pattern } };

    case 'SET_PROCESSING':
      return { ...state, isProcessing: action.isProcessing };

    default:
      return state;
  }
}

// --- Context ---

interface AppContextValue {
  state: AppState;
  dispatch: React.Dispatch<AppAction>;
}

const AppContext = createContext<AppContextValue | null>(null);

export function AppStateProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(reducer, initialState);
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

// --- Derived hooks ---

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
    (p.mode === 'regex' && p.regex_find) ||
    (p.mode === 'template' && p.template) ||
    (p.mode === 'numbering');
  return state.files.length > 0 && !!hasPattern && !state.isProcessing;
}
