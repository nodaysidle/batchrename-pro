import { useCallback } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { RenameTab } from './RenameTab';
import { ConvertTab } from './ConvertTab';
import { MetadataTab } from './MetadataTab';
import { ArrowLeftRight, RefreshCw, Tag } from 'lucide-react';

const TABS = [
  { key: 'rename' as const, label: 'Rename', icon: ArrowLeftRight },
  { key: 'convert' as const, label: 'Convert', icon: RefreshCw },
  { key: 'metadata' as const, label: 'Metadata', icon: Tag },
];

export function TransformationPanel() {
  const { state, dispatch } = useAppState();
  const { activeTab } = state;

  const setTab = useCallback(
    (tab: typeof activeTab) => dispatch({ type: 'SET_ACTIVE_TAB', tab }),
    [dispatch]
  );

  if (state.files.length === 0) return null;

  return (
    <div className="w-80 flex-shrink-0 flex flex-col bg-slate-900/50 border-l border-slate-700/30 rounded-r-2xl overflow-hidden backdrop-blur-sm">
      {/* Tab bar */}
      <div className="flex border-b border-slate-700/30">
        {TABS.map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => setTab(key)}
            className={`
              flex-1 flex items-center justify-center gap-1.5 py-3 text-xs font-medium
              transition-all duration-200 border-b-2
              ${
                activeTab === key
                  ? 'text-[var(--accent)] border-[var(--accent)]'
                  : 'text-slate-500 border-transparent hover:text-slate-400 hover:bg-slate-800/30'
              }
            `}
          >
            <Icon className="w-3.5 h-3.5" />
            {label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex-1 overflow-y-auto scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
        {activeTab === 'rename' && <RenameTab />}
        {activeTab === 'convert' && <ConvertTab />}
        {activeTab === 'metadata' && <MetadataTab />}
      </div>
    </div>
  );
}
