
import { AppStateProvider } from '@/state/AppStateContext';
import { ThemeProvider } from '@/contexts/ThemeContext';
import { useTauriEvents } from '@/hooks/useTauriEvents';
import { Navbar } from '@/components/Navbar';
import { DropZone } from '@/components/DropZone';
import { FileList } from '@/components/FileList';
import { TransformationPanel } from '@/components/TransformationPanel';
import { ActionFooter } from '@/components/ActionFooter';

function AppShell() {
  useTauriEvents();

  return (
    <div className="flex flex-col h-screen bg-slate-950 text-slate-100 overflow-hidden">
      <Navbar />

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
    <ThemeProvider>
      <AppStateProvider>
        <AppShell />
      </AppStateProvider>
    </ThemeProvider>
  );
}
