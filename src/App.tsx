
import { AppStateProvider } from '@/state/AppStateContext';
import { useAppState } from '@/state/AppStateContext';
import { ThemeProvider } from '@/contexts/ThemeContext';
import { useTauriEvents } from '@/hooks/useTauriEvents';
import { Navbar } from '@/components/Navbar';
import { DropZone } from '@/components/DropZone';
import { FileList } from '@/components/FileList';
import { TransformationPanel } from '@/components/TransformationPanel';
import { ActionFooter } from '@/components/ActionFooter';
import { AlertTriangle, X } from 'lucide-react';

function ErrorBanner() {
  const { state, dispatch } = useAppState();
  if (!state.appError) return null;

  return (
    <div
      role="alert"
      className="mx-6 mt-3 flex items-start gap-3 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-100"
    >
      <AlertTriangle className="mt-0.5 h-4 w-4 flex-shrink-0 text-red-300" />
      <div className="min-w-0 flex-1">
        <p className="font-medium text-red-200">{state.appError.code}</p>
        <p className="break-words text-red-100/90">{state.appError.message}</p>
      </div>
      <button
        type="button"
        aria-label="Dismiss error"
        onClick={() => dispatch({ type: 'CLEAR_ERROR' })}
        className="rounded p-1 text-red-200/70 transition-colors hover:bg-red-500/10 hover:text-red-100"
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

function AppShell() {
  useTauriEvents();

  return (
    <div className="flex flex-col h-screen bg-slate-950 text-slate-100 overflow-hidden">
      <Navbar />
      <ErrorBanner />

      <div className="flex flex-1 overflow-hidden">
        {/* Main content area */}
        <main className="flex-1 flex flex-col p-6 gap-6 overflow-hidden">
          {/* Drop zone — always visible */}
          <DropZone />

          {/* File list — visible when files exist */}
          <div className="flex-1 min-h-0 overflow-hidden">
            <FileList />
          </div>
        </main>

        {/* Right sidebar — transformation panel */}
        <TransformationPanel />
      </div>

      {/* Sticky action footer */}
      <ActionFooter />
    </div>
  );
}

export default function App() {
  return (
    <AppStateProvider>
      <ThemeProvider>
        <AppShell />
      </ThemeProvider>
    </AppStateProvider>
  );
}
