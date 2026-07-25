import { useEffect } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { RenameTab } from './RenameTab';
import { ArrowLeftRight } from 'lucide-react';

export function TransformationPanel() {
  const { state, dispatch } = useAppState();
  const { activeTab } = state;

  useEffect(() => {
    if (activeTab !== 'rename') {
      dispatch({ type: 'SET_ACTIVE_TAB', tab: 'rename' });
    }
  }, [activeTab, dispatch]);

  if (state.files.length === 0) return null;

  return (
    <div
      className="w-80 flex-shrink-0 flex flex-col border-l rounded-r-2xl overflow-hidden backdrop-blur-sm"
      style={{ backgroundColor: 'color-mix(in srgb, var(--card) 50%, transparent)', borderColor: 'var(--border)' }}
    >
      {/* Tab bar — rename-only for this release */}
      <div className="flex border-b" style={{ borderColor: 'var(--border)' }}>
        <div
          className="flex-1 flex items-center justify-center gap-1.5 py-3 text-xs font-medium border-b-2"
          style={{ color: 'var(--accent)', borderColor: 'var(--accent)' }}
        >
          <ArrowLeftRight className="w-3.5 h-3.5" />
          Rename
        </div>
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
        <RenameTab />
      </div>
    </div>
  );
}
