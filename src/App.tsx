
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
      className="mx-6 mt-3 flex items-start gap-3 rounded-lg border px-4 py-3 text-sm"
      style={{ borderColor: 'var(--danger-border)', backgroundColor: 'var(--danger-bg)', color: 'var(--danger-text)' }}
    >
      <AlertTriangle className="mt-0.5 h-4 w-4 flex-shrink-0" style={{ color: 'var(--danger-heading)' }} />
      <div className="min-w-0 flex-1">
        <p className="font-medium" style={{ color: 'var(--danger-heading)' }}>{state.appError.code}</p>
        <p className="break-words">{state.appError.message}</p>
      </div>
      <button
        type="button"
        aria-label="Dismiss error"
        onClick={() => dispatch({ type: 'CLEAR_ERROR' })}
        className="rounded p-1 opacity-70 transition-colors hover:opacity-100"
        style={{ color: 'var(--danger-heading)' }}
      >
        <X className="h-4 w-4" />
      </button>
    </div>
  );
}

function AppShell() {
  useTauriEvents();

  return (
    <div className="flex flex-col h-screen overflow-hidden" style={{ backgroundColor: 'var(--bg)', color: 'var(--text)' }}>
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
