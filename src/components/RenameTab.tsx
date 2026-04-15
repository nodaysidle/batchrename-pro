import { useCallback, useMemo } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { Hash, Regex, Type } from 'lucide-react';

export function RenameTab() {
  const { state, dispatch } = useAppState();
  const p = state.renamePattern;

  const updatePattern = useCallback(
    (update: Partial<typeof p>) => {
      dispatch({ type: 'SET_RENAME_PATTERN', pattern: update });
    },
    [dispatch]
  );

  const previewCount = useMemo(
    () => state.files.filter((f) => f.transformed_name).length,
    [state.files]
  );

  return (
    <div className="flex flex-col gap-4 p-4">
      {/* Mode selector */}
      <div className="flex gap-1 p-1 bg-slate-800/50 rounded-xl">
        {(
          [
            { key: 'regex', label: 'Regex', icon: Regex },
            { key: 'template', label: 'Template', icon: Type },
            { key: 'numbering', label: 'Numbering', icon: Hash },
          ] as const
        ).map(({ key, label, icon: Icon }) => (
          <button
            key={key}
            onClick={() => updatePattern({ mode: key })}
            className={`
              flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg
              text-xs font-medium transition-all duration-200
              ${
                p.mode === key
                  ? 'bg-[var(--accent)] text-white shadow-lg shadow-[var(--accent)]/20'
                  : 'text-slate-400 hover:text-slate-300 hover:bg-slate-700/50'
              }
            `}
          >
            <Icon className="w-3.5 h-3.5" />
            {label}
          </button>
        ))}
      </div>

      {/* Regex mode */}
      {p.mode === 'regex' && (
        <>
          <div>
            <label className="text-xs text-slate-400 mb-1 block">Find</label>
            <input
              type="text"
              value={p.regex_find}
              onChange={(e) => updatePattern({ regex_find: e.target.value })}
              placeholder="Search pattern..."
              className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 focus:border-[var(--accent)]/50 transition-all"
            />
          </div>
          <div>
            <label className="text-xs text-slate-400 mb-1 block">Replace with</label>
            <input
              type="text"
              value={p.regex_replace}
              onChange={(e) => updatePattern({ regex_replace: e.target.value })}
              placeholder="Replacement..."
              className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 focus:border-[var(--accent)]/50 transition-all"
            />
          </div>
        </>
      )}

      {/* Template mode */}
      {p.mode === 'template' && (
        <>
          <div>
            <label className="text-xs text-slate-400 mb-1 block">Template</label>
            <input
              type="text"
              value={p.template}
              onChange={(e) => updatePattern({ template: e.target.value })}
              placeholder="{original}_{number}"
              className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 focus:border-[var(--accent)]/50 transition-all"
            />
          </div>
          <div className="flex gap-2 flex-wrap">
            {['{original}', '{number}', '{date}', '{ext}'].map((token) => (
              <button
                key={token}
                onClick={() =>
                  updatePattern({ template: (p.template || '') + token })
                }
                className="px-2.5 py-1 bg-slate-800/60 border border-slate-700/50 rounded-lg text-xs text-[var(--accent)] hover:bg-[var(--accent)]/10 hover:border-[var(--accent)]/30 transition-all duration-200 font-mono"
              >
                {token}
              </button>
            ))}
          </div>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="text-xs text-slate-400 mb-1 block">Start #</label>
              <input
                type="number"
                min={0}
                value={p.start_number}
                onChange={(e) =>
                  updatePattern({ start_number: parseInt(e.target.value) || 0 })
                }
                className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all"
              />
            </div>
            <div className="flex-1">
              <label className="text-xs text-slate-400 mb-1 block">Zero pad</label>
              <input
                type="number"
                min={0}
                max={10}
                value={p.zero_pad}
                onChange={(e) =>
                  updatePattern({ zero_pad: parseInt(e.target.value) || 0 })
                }
                className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all"
              />
            </div>
          </div>
        </>
      )}

      {/* Numbering mode */}
      {p.mode === 'numbering' && (
        <>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="text-xs text-slate-400 mb-1 block">Prefix</label>
              <input
                type="text"
                value={p.prefix}
                onChange={(e) => updatePattern({ prefix: e.target.value })}
                placeholder="file"
                className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all"
              />
            </div>
            <div className="flex-1">
              <label className="text-xs text-slate-400 mb-1 block">Suffix</label>
              <input
                type="text"
                value={p.suffix}
                onChange={(e) => updatePattern({ suffix: e.target.value })}
                placeholder=""
                className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all"
              />
            </div>
          </div>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className="text-xs text-slate-400 mb-1 block">Start #</label>
              <input
                type="number"
                min={0}
                value={p.start_number}
                onChange={(e) =>
                  updatePattern({ start_number: parseInt(e.target.value) || 0 })
                }
                className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all"
              />
            </div>
            <div className="flex-1">
              <label className="text-xs text-slate-400 mb-1 block">Zero pad</label>
              <input
                type="number"
                min={0}
                max={10}
                value={p.zero_pad}
                onChange={(e) =>
                  updatePattern({ zero_pad: parseInt(e.target.value) || 0 })
                }
                className="w-full px-3 py-2 bg-slate-800/60 border border-slate-700/50 rounded-lg text-sm text-slate-200 focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 transition-all"
              />
            </div>
          </div>
        </>
      )}

      {/* Case transform */}
      <div>
        <label className="text-xs text-slate-400 mb-1.5 block">Case transform</label>
        <div className="flex gap-1">
          {(['none', 'upper', 'lower', 'title'] as const).map((ct) => (
            <button
              key={ct}
              onClick={() => updatePattern({ case_transform: ct })}
              className={`
                flex-1 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
                ${
                  p.case_transform === ct
                    ? 'bg-slate-700 text-slate-200'
                    : 'text-slate-500 hover:text-slate-400 hover:bg-slate-800/50'
                }
              `}
            >
              {ct === 'none' ? 'None' : ct.charAt(0).toUpperCase() + ct.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Preview count */}
      {previewCount > 0 && (
        <div className="text-xs text-slate-500 text-center pt-1">
          {previewCount} file{previewCount !== 1 ? 's' : ''} previewed
        </div>
      )}
    </div>
  );
}
