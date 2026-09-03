import { useCallback, useEffect, useMemo, useRef } from 'react';
import { useAppState } from '@/state/AppStateContext';
import { parseError, previewRename } from '@/lib/commands';
import { Hash, Regex, Type } from 'lucide-react';

const inputClass =
  'w-full px-3 py-2 rounded-lg text-sm border focus:outline-none focus:ring-2 focus:ring-[var(--accent)]/50 focus:border-[var(--accent)]/50 transition-all placeholder:text-[var(--text-muted)] placeholder:opacity-70';

const inputStyle = {
  backgroundColor: 'color-mix(in srgb, var(--card) 70%, transparent)',
  borderColor: 'var(--border)',
  color: 'var(--text)',
} as const;

const labelClass = 'text-xs mb-1 block';
const labelStyle = { color: 'var(--text-muted)' } as const;

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
  const conflictCount = useMemo(
    () => state.previews.filter((preview) => preview.has_conflict).length,
    [state.previews]
  );
  const fileSignature = useMemo(
    () =>
      state.files
        .map((file) => `${file.id}:${file.original_path}:${file.original_name}:${file.extension}`)
        .join('|'),
    [state.files]
  );
  const patternSignature = useMemo(() => JSON.stringify(p), [p]);
  const latestRequestId = useRef(0);

  useEffect(() => {
    const hasPattern =
      (p.mode === 'regex' && p.regex_find.trim()) ||
      (p.mode === 'template' && p.template.trim()) ||
      p.mode === 'numbering';

    if (state.files.length === 0 || !hasPattern) {
      latestRequestId.current += 1;
      dispatch({ type: 'SET_PREVIEWS', previews: [] });
      return;
    }

    const timer = window.setTimeout(() => {
      const requestId = ++latestRequestId.current;
      const fileIds = state.files.map((file) => file.id);
      previewRename(fileIds, state.files, p)
        .then((result) => {
          if (requestId !== latestRequestId.current) return;
          dispatch({ type: 'SET_PREVIEWS', previews: result.previews });
        })
        .catch((err) => {
          if (requestId !== latestRequestId.current) return;
          const parsed = parseError(err);
          dispatch({ type: 'SET_PREVIEW_ERROR', error: parsed.message });
        });
    }, 150);

    return () => {
      window.clearTimeout(timer);
      latestRequestId.current += 1;
    };
  }, [dispatch, fileSignature, patternSignature]);

  return (
    <div className="flex flex-col gap-4 p-4">
      {/* Mode selector */}
      <div
        className="flex gap-1 p-1 rounded-xl"
        style={{ backgroundColor: 'color-mix(in srgb, var(--card) 80%, transparent)' }}
      >
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
            aria-label={`Use ${label} rename mode`}
            className={`
              flex-1 flex items-center justify-center gap-1.5 py-2 px-3 rounded-lg
              text-xs font-medium transition-all duration-200
              ${
                p.mode === key
                  ? 'bg-[var(--accent)] text-white shadow-lg shadow-[var(--accent)]/20'
                  : 'hover:bg-[var(--border)]'
              }
            `}
            style={p.mode === key ? undefined : { color: 'var(--text-muted)' }}
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
            <label className={labelClass} style={labelStyle}>
              Find
            </label>
            <input
              type="text"
              value={p.regex_find}
              onChange={(e) => updatePattern({ regex_find: e.target.value })}
              placeholder="Search pattern..."
              aria-label="Regex find pattern"
              className={inputClass}
              style={inputStyle}
            />
          </div>
          <div>
            <label className={labelClass} style={labelStyle}>
              Replace with
            </label>
            <input
              type="text"
              value={p.regex_replace}
              onChange={(e) => updatePattern({ regex_replace: e.target.value })}
              placeholder="Replacement..."
              aria-label="Regex replacement"
              className={inputClass}
              style={inputStyle}
            />
          </div>
        </>
      )}

      {/* Template mode */}
      {p.mode === 'template' && (
        <>
          <div>
            <label className={labelClass} style={labelStyle}>
              Template
            </label>
            <input
              type="text"
              value={p.template}
              onChange={(e) => updatePattern({ template: e.target.value })}
              placeholder="{original}_{number}"
              aria-label="Rename template"
              className={inputClass}
              style={inputStyle}
            />
          </div>
          <div className="flex gap-2 flex-wrap">
            {['{original}', '{number}', '{date}', '{ext}'].map((token) => (
              <button
                key={token}
                onClick={() =>
                  updatePattern({ template: (p.template || '') + token })
                }
                aria-label={`Insert ${token} token`}
                className="px-2.5 py-1 rounded-lg text-xs text-[var(--accent)] border font-mono transition-all duration-200 hover:bg-[var(--accent)]/10 hover:border-[var(--accent)]/30"
                style={{
                  backgroundColor: 'color-mix(in srgb, var(--card) 70%, transparent)',
                  borderColor: 'var(--border)',
                }}
              >
                {token}
              </button>
            ))}
          </div>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className={labelClass} style={labelStyle}>
                Start #
              </label>
              <input
                type="number"
                min={0}
                value={p.start_number}
                aria-label="Template start number"
                onChange={(e) =>
                  updatePattern({ start_number: parseInt(e.target.value) || 0 })
                }
                className={inputClass}
                style={inputStyle}
              />
            </div>
            <div className="flex-1">
              <label className={labelClass} style={labelStyle}>
                Zero pad
              </label>
              <input
                type="number"
                min={0}
                max={10}
                value={p.zero_pad}
                aria-label="Template zero pad"
                onChange={(e) =>
                  updatePattern({ zero_pad: parseInt(e.target.value) || 0 })
                }
                className={inputClass}
                style={inputStyle}
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
              <label className={labelClass} style={labelStyle}>
                Prefix
              </label>
              <input
                type="text"
                value={p.prefix}
                onChange={(e) => updatePattern({ prefix: e.target.value })}
                placeholder="file"
                aria-label="Numbering prefix"
                className={inputClass}
                style={inputStyle}
              />
            </div>
            <div className="flex-1">
              <label className={labelClass} style={labelStyle}>
                Suffix
              </label>
              <input
                type="text"
                value={p.suffix}
                onChange={(e) => updatePattern({ suffix: e.target.value })}
                placeholder=""
                aria-label="Numbering suffix"
                className={inputClass}
                style={inputStyle}
              />
            </div>
          </div>
          <div className="flex gap-3">
            <div className="flex-1">
              <label className={labelClass} style={labelStyle}>
                Start #
              </label>
              <input
                type="number"
                min={0}
                value={p.start_number}
                aria-label="Numbering start number"
                onChange={(e) =>
                  updatePattern({ start_number: parseInt(e.target.value) || 0 })
                }
                className={inputClass}
                style={inputStyle}
              />
            </div>
            <div className="flex-1">
              <label className={labelClass} style={labelStyle}>
                Zero pad
              </label>
              <input
                type="number"
                min={0}
                max={10}
                value={p.zero_pad}
                aria-label="Numbering zero pad"
                onChange={(e) =>
                  updatePattern({ zero_pad: parseInt(e.target.value) || 0 })
                }
                className={inputClass}
                style={inputStyle}
              />
            </div>
          </div>
        </>
      )}

      {/* Case transform */}
      <div>
        <label className="text-xs mb-1.5 block" style={labelStyle}>
          Case transform
        </label>
        <div className="flex gap-1">
          {(['none', 'upper', 'lower', 'title'] as const).map((ct) => (
            <button
              key={ct}
              onClick={() => updatePattern({ case_transform: ct })}
              aria-label={`Use ${ct} case transform`}
              className={`
                flex-1 py-1.5 rounded-lg text-xs font-medium transition-all duration-200
                ${p.case_transform === ct ? '' : 'hover:bg-[var(--border)]'}
              `}
              style={
                p.case_transform === ct
                  ? { backgroundColor: 'var(--border)', color: 'var(--text)' }
                  : { color: 'var(--text-muted)' }
              }
            >
              {ct === 'none' ? 'None' : ct.charAt(0).toUpperCase() + ct.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Preview count */}
      {state.previewError && (
        <div
          className="rounded-lg border px-3 py-2 text-xs"
          style={{
            borderColor: 'var(--danger-border)',
            backgroundColor: 'var(--danger-bg)',
            color: 'var(--danger-text)',
          }}
        >
          {state.previewError}
        </div>
      )}

      {conflictCount > 0 && (
        <div
          className="rounded-lg border px-3 py-2 text-xs"
          style={{
            borderColor: 'var(--danger-border)',
            backgroundColor: 'var(--danger-bg)',
            color: 'var(--danger-text)',
          }}
        >
          {conflictCount} conflict{conflictCount !== 1 ? 's' : ''} found. Resolve conflicts before applying.
        </div>
      )}

      {previewCount > 0 && (
        <div className="text-xs text-center pt-1" style={{ color: 'var(--text-muted)' }}>
          {previewCount} file{previewCount !== 1 ? 's' : ''} previewed
        </div>
      )}
    </div>
  );
}
