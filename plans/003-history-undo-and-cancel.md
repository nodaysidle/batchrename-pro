# Plan 003: History undo and Cancel during apply

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**: `git diff --stat 06c3362..HEAD -- src/components/ActionFooter.tsx src/components/FileCard.tsx src/hooks/useTauriEvents.ts src/state/AppStateContext.tsx src/lib/commands.ts`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.
>
> **Prerequisite**: `plans/002-apply-state-correctness.md` should be DONE (or equivalent: `START_JOB` with real `job_id` during apply, paths updated after rename, `COMPLETE_JOB` from invoke). If 002 is not done, STOP and report.

## Status

- **Priority**: P1
- **Effort**: S–M
- **Risk**: LOW
- **Depends on**: `plans/002-apply-state-correctness.md`
- **Category**: bug / UI
- **Planned at**: commit `06c3362`, 2026-07-25

## Why this matters

Undo is marketed as a core safety feature, but the UI only undoes `lastCompletedJobId` (lost on clear/relaunch) while History already shows `can_undo` with no action. Cancel IPC exists (`cancelJob`) but there is no Cancel control during apply. Failed files store `error` but FileCard never shows it — users cannot diagnose partial failures.

## Current state

```35:54:src/components/ActionFooter.tsx
  const handleUndo = useCallback(async () => {
    if (!state.lastCompletedJobId) return;
    try {
      const result = await undoJobCmd(state.lastCompletedJobId);
      if (result.success) {
        dispatch({ type: 'CLEAR_FILES' });
      } else { ... }
```

```183:206:src/components/ActionFooter.tsx
              state.history.map((job) => (
                <div key={job.id} className="flex items-center gap-3 ...">
                  ...
                  <p className="text-xs ...">{job.description}</p>
                  <p className="text-[10px] ...">
                    {job.file_count} files · {job.operation_type}
                  </p>
                </div>
              ))
```

```74:76:src/lib/commands.ts
export async function cancelJob(jobId: string): Promise<boolean> {
  return safeInvoke('cancel_job', { jobId });
}
```

```100:112:src/components/FileCard.tsx
      {/* Status icons only — file.error never rendered */}
```

`JobSummary.can_undo` exists in `src/types.ts` (lines 88–96).

**Conventions**: Reuse `undoJob` / `parseError` / `SET_ERROR` patterns from existing `handleUndo`. Match ActionFooter button styling (small ghost buttons). Prefer extracting a small `HistoryDropdown` component **only if** ActionFooter becomes unwieldy — optional, not required.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `npm run typecheck` | exit 0 |
| Build | `npm run build` | exit 0 |
| Rust tests (unchanged) | `cd src-tauri && cargo test` | pass (if you did not change Rust; if 002 already landed, still pass) |

## Scope

**In scope**:
- `src/components/ActionFooter.tsx`
- `src/components/FileCard.tsx`
- Optionally `src/components/HistoryDropdown.tsx` (create) if extracted
- `src/hooks/useTauriEvents.ts` / `src/state/AppStateContext.tsx` only if needed to surface cancel/busy (prefer minimal)
- `plans/README.md`

**Out of scope**:
- Changing Rust undo semantics / backup matching
- Path trust (004), polish pack (001), docs (005)
- Implementing convert
- Persisting `lastCompletedJobId` to disk (History undo replaces that need)

## Git workflow

- Branch: `advisor/003-history-undo-and-cancel`
- Commit example: `feat: undo from history and cancel in-flight renames`
- Do NOT push unless instructed.

## Steps

### Step 1: Per-row History undo

In the History panel:
- Show `job.timestamp` (format simply, e.g. locale string or truncate ISO).
- For each job with `job.can_undo === true`, render an Undo button that calls `undoJob(job.id)`.
- On full success: refresh history via `getJobHistory`, clear or update file list appropriately (existing `CLEAR_FILES` is OK if paths are restored on disk; do not leave “done” cards pointing at old names — after 002, prefer clearing or resetting statuses).
- On partial failure: `SET_ERROR` with joined errors (same as footer Undo).
- Disable Undo while `state.isProcessing`.
- Keep footer Undo for `lastCompletedJobId` as a convenience, or remove it if redundant — either OK if History always works.

**Verify**: `rg "can_undo" src/components/ActionFooter.tsx` (or HistoryDropdown) → used in UI logic. `npm run typecheck` → exit 0.

### Step 2: Cancel while processing

- Import `cancelJob` from `@/lib/commands`.
- When `state.isProcessing && state.activeJobId`, show a Cancel button near Apply.
- On click: `await cancelJob(state.activeJobId)`; on failure show `SET_ERROR`.
- Do not invent job ids — if `activeJobId` is null during processing, STOP and report (002 did not land `START_JOB`).

**Verify**: `rg "cancelJob" src/components/ActionFooter.tsx` → match. `npm run typecheck` → exit 0.

### Step 3: Show `file.error` on FileCard

When `file.status === 'error'` and `file.error`, show a truncated one-line message under the name (reuse conflict text styling). Keep `aria` / `title` with full error if truncated.

**Verify**: `rg "file\\.error" src/components/FileCard.tsx` → match. `npm run typecheck && npm run build` → exit 0.

### Step 4: Update index

Mark 003 DONE in `plans/README.md`.

## Test plan

- No FE test runner — manual:
  1. Complete a rename → History shows Undo → click Undo → files restored (spot-check).
  2. Relaunch app → open History → Undo still available for `can_undo` jobs.
  3. Start Apply on many files → Cancel appears → after cancel some files may be cancelled/failed without hang.
  4. Force a failed file (if possible) → error text visible on card.
- `cd src-tauri && cargo test` still green.

## Done criteria

- [ ] History rows with `can_undo` expose working Undo calling `undoJob(job.id)`
- [ ] Cancel visible during `isProcessing` and calls `cancelJob(activeJobId)`
- [ ] FileCard renders `file.error` when status is error
- [ ] `npm run typecheck` and `npm run build` exit 0
- [ ] Scope respected; README status DONE

## STOP conditions

- `activeJobId` never set during apply → 002 incomplete; stop.
- Undo needs new Rust APIs beyond `undo_job` → stop.
- Drift / verification fails twice.

## Maintenance notes

- Partial undo already refuses overwrite (`processing_pipeline` tests) — UI must surface `UNDO_PARTIAL`.
- Reviewer: ensure Cancel does not deadlock UI if job already finished; disable Cancel when not processing.
- Overlay Escape/outside-click may come from plan 001 — do not regress if both land.
