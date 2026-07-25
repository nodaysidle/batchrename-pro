# Plan 001: Ship a polished rename-only shell (theme, fonts, a11y, density)

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**: `git diff --stat 06c3362..HEAD -- src/App.tsx src/index.css src/main.tsx index.html src/contexts/ThemeContext.tsx src/components/Navbar.tsx src/components/DropZone.tsx src/components/FileCard.tsx src/components/FileList.tsx src/components/TransformationPanel.tsx src/components/ActionFooter.tsx src/components/ConvertTab.tsx src/components/MetadataTab.tsx`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW
- **Depends on**: Prefer `plans/002-apply-state-correctness.md` first (or parallel carefully). Does **not** require 003/004/005.
- **Category**: direction (UX polish) / tech-debt
- **Planned at**: commit `06c3362`, 2026-07-25

## Why this matters

v0.1.0 ships a working rename core, but the UI reads as unfinished vs the NODAYSIDLE 9.7 bar: light theme is a no-op, Inter/Mono are named but never loaded, Convert/Metadata tabs show “Soon” stubs, overlays lack Escape/outside-click, and Task 5 motion/a11y assets were never added. This plan makes the **rename-only** shell feel finished without implementing convert/metadata backends or a full SettingsModal for backup/parallelism.

## Current state

Relevant files:

- `src/App.tsx` — shell hardcodes dark slate (`bg-slate-950 text-slate-100`); always mounts full DropZone.
- `src/index.css` — only `--accent`; `pulse-glow` uses unset `--accent-rgb`; Inter declared without `@font-face`.
- `index.html` — body hardcoded dark; no font links.
- `src/contexts/ThemeContext.tsx` — toggles `html.dark` + `--accent` hex only.
- `src/components/TransformationPanel.tsx` — Convert/Metadata `disabled: true` with “Soon”.
- `src/components/DropZone.tsx` — always `p-12` large zone.
- `src/components/FileCard.tsx` — remove button `opacity-0 group-hover:opacity-100` only.
- `src/components/Navbar.tsx` — Help dialog / Appearance dropdown lack Escape, outside-click, focus restore.
- `src/components/ActionFooter.tsx` — History panel same overlay gaps; no `aria-live` in app.
- `src/main.tsx` — imports `./index.css` only (no `styles/animations.css`).

Excerpts (as of `06c3362`):

```43:51:src/App.tsx
    <div className="flex flex-col h-screen bg-slate-950 text-slate-100 overflow-hidden">
      <Navbar />
      <ErrorBanner />
      ...
          <DropZone />
```

```33:36:src/contexts/ThemeContext.tsx
  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark');
    document.documentElement.style.setProperty('--accent', ACCENT_MAP[accentColor] ?? '#3B82F6');
  }, [theme, accentColor]);
```

```8:12:src/components/TransformationPanel.tsx
const TABS = [
  { key: 'rename' as const, label: 'Rename', icon: ArrowLeftRight, disabled: false },
  { key: 'convert' as const, label: 'Convert', icon: RefreshCw, disabled: true },
  { key: 'metadata' as const, label: 'Metadata', icon: Tag, disabled: true },
];
```

```116:119:src/components/FileCard.tsx
      <button
        ...
        className="... opacity-0 group-hover:opacity-100 ..."
```

```54:61:src/index.css
body {
  font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}
```

**Design tokens to honor** (from root `AGENTS.md` / PRD): dark bg `#0F172A`, cards `#1E293B`, text `#F1F5F9`, accent `#3B82F6` (violet `#A78BFA` allowed); Navbar 48px (`h-12`); transitions ~200ms ease-out; 2px accent focus rings (already in `index.css`).

**Repo conventions**:
- Path alias `@/` → `src/` (see existing imports).
- Tailwind utility classes + CSS variables for theme; keep desktop-first (no mobile responsive rework).
- Error UI pattern: `role="alert"` banner in `App.tsx` — match that tone for live regions.
- **Do not add npm dependencies** without approval (`AGENTS.md`). Prefer local font files under `public/fonts/` + `@font-face`, not a new font package.
- Commit style examples: `feat: ...`, `fix: ...`, `docs: ...` (see `git log`).

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `npm run typecheck` | exit 0 |
| Frontend build | `npm run build` | exit 0 |
| Status scope check | `git status --short` | only intended files under `src/`, `public/`, `index.html`, `plans/` |

## Scope

**In scope** (only modify/create these):
- `src/index.css`
- `src/styles/animations.css` (create)
- `src/main.tsx` (import animations.css)
- `index.html`
- `public/fonts/` (optional Inter + JetBrains Mono woff2 + licenses if bundling fonts)
- `src/contexts/ThemeContext.tsx`
- `src/App.tsx`
- `src/components/Navbar.tsx`
- `src/components/DropZone.tsx`
- `src/components/FileCard.tsx`
- `src/components/FileList.tsx` (stagger class only if needed)
- `src/components/TransformationPanel.tsx`
- `src/components/ActionFooter.tsx` (overlay dismiss + aria-live region only — **not** History undo/Cancel; that is plan 003)
- `src/components/ConvertTab.tsx` / `MetadataTab.tsx` — only if removing imports after hiding tabs (prefer leave files unused rather than delete unless clean)
- `plans/README.md` (status row)

**Out of scope**:
- Full `SettingsModal` with output dir / max parallel / backup retention (direction; not this plan).
- Convert/metadata Rust backends, ffmpeg, id3.
- Applying numbering/path/job correctness (plan 002), History undo/Cancel (plan 003), shell removal (plan 004), AGENTS/TASKS rewrite (plan 005).
- Confetti particle libraries or new npm deps.
- Changing release/signing/CI.

## Git workflow

- Branch: `advisor/001-rename-shell-polish-pack`
- Commit style: `feat: polish rename shell theme fonts and a11y`
- Do NOT push or open a PR unless instructed.

## Steps

### Step 1: Theme CSS variables (dark + light)

In `src/index.css`, define semantic tokens on `:root` (dark default) and `html:not(.dark)` or `html.light` for light:

- `--bg`, `--bg-elevated`, `--card`, `--text`, `--text-muted`, `--border`, `--accent`, `--accent-rgb` (comma-separated RGB for glow).

Map AGENTS tokens for dark: bg `#0F172A`, card `#1E293B`, text `#F1F5F9`. Light: readable light bg/card/text (not purple-on-white novelty; keep slate/neutral).

Update `ThemeContext.tsx` to set `--accent` **and** `--accent-rgb` when accent changes (blue `59, 130, 246`; violet `167, 139, 250`). When theme is light, ensure `document.documentElement.classList` removes `dark` (already toggles) and that body/`App` use vars.

Replace hardcoded `bg-slate-950 text-slate-100` on `App.tsx` shell and `index.html` body with `bg-[var(--bg)] text-[var(--text)]` (or equivalent Tailwind arbitrary values). Update high-traffic surfaces (Navbar, DropZone, FileCard, TransformationPanel, ActionFooter, ErrorBanner) to use `--card` / `--border` / `--text-muted` instead of fixed dark-only slate where light would break contrast.

**Verify**: `npm run typecheck` → exit 0. Manually note: toggling theme in code path sets class without hardcoded body slate-950 remaining as the only bg.

**Grep gate**: `rg "bg-slate-950" src/App.tsx index.html` → no matches (or only comments).

### Step 2: Load Inter + Mono without new npm deps

Preferred: add OFL-licensed `Inter` and `JetBrains Mono` woff2 under `public/fonts/` with `@font-face` in `src/index.css`. Keep existing font-family stacks.

If obtaining font binaries is blocked in the environment: STOP and report **or** change `body` / `.font-mono` to an explicit system stack (`system-ui, -apple-system, …` and `ui-monospace, …`) and remove the false “Inter” claim from CSS comments — do **not** add Google Fonts CDN (offline desktop app).

**Verify**: `ls public/fonts/` shows font files **or** CSS no longer claims Inter without loading. `npm run build` → exit 0.

### Step 3: Create `src/styles/animations.css` and wire it

Create utilities (names from AGENTS Task 5, implement modestly):

- `.transition-default` — 200ms ease-out
- `.hover-lift` — translateY(-2px) + subtle shadow on hover
- `.hover-glow` — accent-tinted shadow using `var(--accent)` / `--accent-rgb`
- `.shake-error` — short horizontal shake keyframes
- `.stagger-in` — fadeInUp with `animation-delay: calc(var(--i, 0) * 50ms)`
- `.progress-fill` — width transition 300ms
- Respect `@media (prefers-reduced-motion: reduce)` by disabling/ shortening animations
- Fix or replace dead `pulse-glow` to use working accent RGB

Import in `src/main.tsx`: `import './styles/animations.css';`

Apply:
- DropZone: drag glow / `hover-glow` when dragging
- FileCard: `hover-lift`; `shake-error` when `file.status === 'error'`
- FileList non-virtualized rows: set `--i` and `stagger-in` (cap delay for large lists, e.g. `min(i, 20)`)
- ActionFooter progress bar: `progress-fill` class on the fill div
- Optional subtle success flash — **not** a heavy confetti library

**Verify**: `test -f src/styles/animations.css` → exists. `rg "animations.css" src/main.tsx` → match. `npm run typecheck` → exit 0.

### Step 4: Hide Convert/Metadata stubs from the tab bar

In `TransformationPanel.tsx`, remove Convert/Metadata from the visible tab list (filter to rename-only). Keep `ConvertTab.tsx` / `MetadataTab.tsx` files on disk but unmounted, **or** leave a single muted “More tools coming later” line under Rename — do **not** show disabled tabs with “Soon”.

Ensure `activeTab` stays `'rename'` (if state can still be convert/metadata from persisted nothing — initial state is rename). If `activeTab !== 'rename'`, force render RenameTab.

**Verify**: `rg "Soon" src/components/TransformationPanel.tsx` → no matches. `npm run typecheck` → exit 0.

### Step 5: Compact DropZone when files exist

In `DropZone.tsx` (or `App.tsx` passing a prop): when `state.files.length > 0`, use a compact horizontal strip (`p-3` / `p-4`, smaller icon, single-line label “Add more files”) instead of `p-12` hero. Empty state keeps the large dashed zone.

**Verify**: `rg "p-12" src/components/DropZone.tsx` → only on empty-state branch. `npm run typecheck` → exit 0.

### Step 6: Overlay dismiss + focus restore (Navbar + History)

For Help dialog, Appearance dropdown (`Navbar.tsx`), and History panel (`ActionFooter.tsx`):

- Close on `Escape`
- Close on outside click / backdrop click (Help backdrop)
- Restore focus to the trigger button on close
- Set `aria-expanded` / `aria-controls` on gear and History triggers
- Help: simple focus trap (Tab cycles within dialog) — keep implementation small; if trap is too heavy, at least Escape + initial focus on dialog + restore focus

Do **not** implement per-row History undo here (plan 003).

**Verify**: `rg "Escape|keydown|aria-expanded" src/components/Navbar.tsx src/components/ActionFooter.tsx` → shows Escape handlers and aria-expanded. `npm run typecheck` → exit 0.

### Step 7: aria-live + prefers-contrast + focus-within remove

- Add a polite `aria-live="polite"` region in `AppShell` (or Footer) that announces processing counts / “Job complete” when `isProcessing` flips or `lastCompletedJobId` changes. Keep text short.
- In `index.css`, add `@media (prefers-contrast: more)` boosting borders/text contrast.
- FileCard remove button: add `group-focus-within:opacity-100` (and/or always `opacity-60` with hover/focus `opacity-100`).

**Verify**: `rg "aria-live" src/` → ≥1 match. `rg "prefers-contrast" src/index.css` → match. `rg "focus-within" src/components/FileCard.tsx` → match. `npm run typecheck && npm run build` → exit 0.

### Step 8: Update plan index

Set plan 001 status to DONE in `plans/README.md`.

**Verify**: `git status --short` → only in-scope paths (+ `plans/README.md`).

## Test plan

- No frontend test runner in repo (`package.json` has no `test`). Do **not** add Vitest in this plan.
- Manual checklist (executor records in PR/commit body):
  1. Toggle light/dark — chrome colors change (not only the moon icon).
  2. With 0 files — large DropZone; with files — compact strip + Rename panel without Soon tabs.
  3. Tab to a FileCard — remove control visible; Escape closes Help/History/Appearance.
  4. `prefers-reduced-motion` does not break layout.
- Rust: unchanged — no need for `cargo test` unless you touched Rust (you must not).

## Done criteria

- [ ] `npm run typecheck` exits 0
- [ ] `npm run build` exits 0
- [ ] `src/styles/animations.css` exists and is imported from `src/main.tsx`
- [ ] `rg "Soon" src/components/TransformationPanel.tsx` has no matches
- [ ] `rg "bg-slate-950" src/App.tsx index.html` has no matches
- [ ] `rg "aria-live" src/` has matches; FileCard has `focus-within` visibility for remove
- [ ] ThemeContext sets `--accent-rgb` (or glow no longer references unset RGB)
- [ ] No files outside Scope modified
- [ ] `plans/README.md` row for 001 → DONE

## STOP conditions

- Drift: in-scope excerpts no longer match.
- Fix appears to require implementing convert/metadata or full SettingsModal → stop; those are out of scope.
- Adding an npm dependency seems “required” for fonts → stop and report (use local fonts or system stack instead).
- Step verification fails twice after a reasonable fix.

## Maintenance notes

- Full SettingsModal (parallel jobs, retention) remains deferred — wire later against existing SQLite settings keys.
- When Convert ships, re-introduce tabs carefully; do not resurrect “Soon” dead tabs.
- Reviewer: check light theme contrast on FileCard/Navbar; ensure reduced-motion path; confirm no CDN fonts.
