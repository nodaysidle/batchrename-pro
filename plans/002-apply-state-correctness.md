# Plan 002: Fix apply/preview state correctness

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**: `git diff --stat 06c3362..HEAD -- src-tauri/src/preview_service.rs src-tauri/src/processing_pipeline.rs src-tauri/src/types.rs src-tauri/src/tests/mod.rs src/hooks/useTauriEvents.ts src/state/AppStateContext.tsx src/components/RenameTab.tsx src/components/ActionFooter.tsx src/types.ts`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: LOW–MED
- **Depends on**: none (blocks plan 003)
- **Category**: bug
- **Planned at**: commit `06c3362`, 2026-07-25

## Why this matters

Four correctness bugs undermine the rename product: (1) after a successful rename the UI keeps stale `original_path`/`original_name`, so a second Apply fails; (2) numbering mode applies prefix/suffix twice and empty prefix skips the `"file"` default; (3) debounced preview can apply an older IPC response over a newer pattern; (4) Apply completion and Undo id depend only on `job_complete` events while `applyRename` already returns `job_id` synchronously — missed events leave the UI stuck in Processing.

## Current state

- `src-tauri/src/preview_service.rs` — `apply_numbering` embeds prefix/suffix; `apply_pattern` applies them again.
- `src/hooks/useTauriEvents.ts` — progress updates status/error only; no path update.
- `src/state/AppStateContext.tsx` — `UPDATE_FILE_STATUS` does not touch `original_path` / `original_name`.
- `src/components/RenameTab.tsx` — debounced preview with no generation token.
- `src/components/ActionFooter.tsx` — awaits `applyRename` but discards `JobStartResponse`; only `SET_PROCESSING`.
- `src-tauri/src/types.rs` / `src/types.ts` — `JobProgressEvent` has no `transformed_path` field.

Excerpts:

```92:98:src-tauri/src/preview_service.rs
    // Apply prefix/suffix
    if let Some(prefix) = &pattern.prefix {
        result = format!("{}{}", prefix, result);
    }
    if let Some(suffix) = &pattern.suffix {
        result = format!("{}{}", result, suffix);
    }
```

```176:188:src-tauri/src/preview_service.rs
fn apply_numbering(...) {
    let prefix = pattern.prefix.as_deref().unwrap_or("file");
    let suffix = pattern.suffix.as_deref().unwrap_or("");
    ...
}
```

```12:18:src/hooks/useTauriEvents.ts
    onJobProgress((event) => {
      dispatch({
        type: 'UPDATE_FILE_STATUS',
        fileId: event.file_id,
        status: ...,
        error: event.error_message ?? undefined,
      });
```

```14:26:src/components/ActionFooter.tsx
    dispatch({ type: 'SET_PROCESSING', isProcessing: true });
    try {
      ...
      await applyRenameCmd(fileIds, state.files, pattern);
```

```45:54:src/components/RenameTab.tsx
    const timer = window.setTimeout(() => {
      previewRename(...)
        .then((result) => {
          dispatch({ type: 'SET_PREVIEWS', previews: result.previews });
        })
```

**Conventions**:
- Rust errors use `CODE: message` strings (see `file_service.rs` / `preview_service.rs`).
- Frontend `parseError` splits on first colon (`src/lib/commands.ts`).
- Rust tests live in `src-tauri/src/tests/mod.rs` — model new tests after `preview_regex_replace` / `preview_template_tokens_and_zero_pad_numbering`.
- Commit style: `fix: ...`

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Typecheck | `npm run typecheck` | exit 0 |
| Build FE | `npm run build` | exit 0 |
| Rust tests | `cd src-tauri && cargo test` | all pass |
| Preview numbering filter | `cd src-tauri && cargo test preview_numbering -- --nocapture` | pass (after adding) |

## Scope

**In scope**:
- `src-tauri/src/preview_service.rs`
- `src-tauri/src/processing_pipeline.rs` (emit optional `transformed_path` on progress when useful)
- `src-tauri/src/types.rs` (`JobProgressEvent`)
- `src-tauri/src/tests/mod.rs`
- `src/types.ts`
- `src/state/AppStateContext.tsx`
- `src/hooks/useTauriEvents.ts`
- `src/components/RenameTab.tsx`
- `src/components/ActionFooter.tsx`
- `plans/README.md`

**Out of scope**:
- History per-row undo / Cancel UI (plan 003)
- Path re-validation / shell removal (plan 004)
- Persisting DB results before each rename (crash durability — deferred)
- UI polish (plan 001)
- Docs/TASKS (plan 005)

## Git workflow

- Branch: `advisor/002-apply-state-correctness`
- Commit message example: `fix: keep file paths and numbering preview correct after apply`
- Do NOT push unless instructed.

## Steps

### Step 1: Fix numbering prefix/suffix (Rust)

In `apply_numbering`:
- Treat missing **or empty** prefix as default `"file"`: e.g. `prefix.filter(|s| !s.is_empty()).unwrap_or("file")` after normalizing `Option`.
- Treat empty suffix as `""`.

In `apply_pattern`, **skip** the second prefix/suffix pass when `pattern.mode == RenameMode::Numbering` (numbering already applied them). Keep the outer prefix/suffix pass for Regex and Template modes.

Add tests in `src-tauri/src/tests/mod.rs`:
1. Numbering with `prefix: Some("img".into())` → `img1.txt` (not `imgimg1.txt`) for a `.txt` file with start 1 pad 0.
2. Numbering with `prefix: Some("".into())` → default `file1.txt` (or `file001` with pad).
3. Template mode with prefix still applies outer prefix once (regression guard).

Helper: extend test helpers to build a numbering `RenamePattern` (see existing `regex_pattern` / `template_pattern`).

**Verify**: `cd src-tauri && cargo test` → all pass including new numbering tests.

### Step 2: Include transformed path on completed progress events

Extend Rust `JobProgressEvent` with `transformed_path: Option<String>` (serde; omit or null when N/A).

In `processing_pipeline.rs` `emit_progress` / `finish_outcome`, when status is success/completed, pass the new path string.

Mirror field on `src/types.ts` `JobProgressEvent`.

**Verify**: `cd src-tauri && cargo test` → pass. `npm run typecheck` → pass.

### Step 3: Update FileInfo paths on completed status

Extend `UPDATE_FILE_STATUS` action to accept optional `originalPath` / `originalName` (or a single `pathUpdate`).

In reducer, when status is `done` and path updates provided:
- set `original_path`, `original_name`
- set `transformed_name` to null (or keep until preview refresh)
- clear `error`

In `useTauriEvents`, on `completed`, pass `transformed_path` from the event as the new `original_path`, and basename as `original_name` (from path or from existing `transformed_name`).

After a full job complete, clear previews or leave them empty so Apply requires a fresh preview (safer).

**Verify**: `npm run typecheck` → exit 0. Logic review: second apply would send updated paths.

### Step 4: Preview request generation id

In `RenameTab.tsx` effect:
- Use a `let requestId = 0` ref or incrementing counter closed over each effect run.
- On each debounce fire, capture `const id = ++latest`.
- Only `dispatch(SET_PREVIEWS)` / `SET_PREVIEW_ERROR` if `id === latest` when the promise settles.
- Cleanup: increment/cancel so in-flight responses are ignored (timer clear alone is insufficient).

**Verify**: `npm run typecheck` → exit 0. `rg "requestId|previewGeneration|latestRequest" src/components/RenameTab.tsx` → shows a generation guard.

### Step 5: Complete job from invoke result

In `ActionFooter.tsx` `handleApply`:
1. Before invoke: `dispatch({ type: 'START_JOB', jobId: 'pending' })` is **wrong** — better: keep `SET_PROCESSING` **or** wait until response. Preferred:
   - Call `applyRenameCmd` …
   - On success: `dispatch({ type: 'COMPLETE_JOB', jobId: result.job_id })` (and ensure file statuses were updated via events; if events already fired during the sync call, COMPLETE_JOB is still needed for `lastCompletedJobId` / `isProcessing`).
2. Because `apply_rename` is synchronous until the batch finishes, progress events typically arrive during the await. Still: **always** dispatch `COMPLETE_JOB` with `result.job_id` on successful invoke so Undo appears even if `job_complete` was missed.
3. On success after COMPLETE_JOB, optionally refresh history — not required.
4. Do not leave `isProcessing` true if COMPLETE_JOB already clears it.

If events already dispatched COMPLETE_JOB, a second COMPLETE_JOB with same id must be idempotent (current reducer just sets the same fields — OK).

Also dispatch `START_JOB` with a temporary id **only if** you change Rust to return job_id before work completes — **do not** change pipeline to async return early in this plan. For Cancel (plan 003), job_id exists only after completion today — plan 003 must note cancel mid-flight needs job_id registered at start. **For this plan**: at minimum ensure completion + `lastCompletedJobId` from invoke result.

**Important for 003**: Register cancel flag under `job_id` before Rayon starts (already does). Frontend Cancel needs `activeJobId` during the await. So in this plan also:
- Change `apply_rename` response flow OR have frontend generate nothing — better Rust fix deferred.
- Minimal approach for 002: on invoke **start**, you cannot know job_id yet. Plan 003 will need either (a) Rust emits `job_started` with id before Rayon, or (b) `apply_rename` returns immediately with job_id and runs async. **For 002**, document that Cancel requires a follow-up in 003: add `job_started` emit at the beginning of `execute_batch_rename` (after `job_id` created) and listen in `useTauriEvents` to `START_JOB`. **Implement that emit + listener here** so 003 can Cancel:

In `processing_pipeline.rs`, after `job_id` is created and cancel flag inserted, emit a small event e.g. `job_started` with `{ job_id }` (add type) **or** reuse progress with files_completed=0. Simplest: add `JobStartedEvent { job_id }` emit once.

Frontend: listen and `dispatch({ type: 'START_JOB', jobId })`.

On invoke success still `COMPLETE_JOB`.

**Verify**: `rg "COMPLETE_JOB" src/components/ActionFooter.tsx` → present on success path. `rg "job_started|START_JOB" src/` → START_JOB wired from event or equivalent. `npm run typecheck && cd src-tauri && cargo test` → pass.

### Step 6: Update index

Mark 002 DONE in `plans/README.md`.

## Test plan

- New Rust tests (numbering double-prefix / empty prefix / template+prefix) in `src-tauri/src/tests/mod.rs` — pattern after `preview_regex_replace`.
- Existing `rename_operation_creates_backup_and_undo_removes_output` must still pass.
- No Vitest — typecheck is the FE gate.

**Verify**: `cd src-tauri && cargo test` → all pass.

## Done criteria

- [ ] Numbering with prefix `img` produces single `img…` names (cargo test asserts)
- [ ] Empty numbering prefix defaults to `file…` (cargo test asserts)
- [ ] `UPDATE_FILE_STATUS` can update `original_path` / `original_name` on done
- [ ] Progress event carries `transformed_path` when completed
- [ ] RenameTab ignores stale preview responses (generation id)
- [ ] Successful `applyRename` dispatches `COMPLETE_JOB` with returned `job_id`
- [ ] `job_started` (or equivalent) dispatches `START_JOB` with real id before/during batch
- [ ] `npm run typecheck` and `cd src-tauri && cargo test` exit 0
- [ ] Scope respected; `plans/README.md` → DONE

## STOP conditions

- Drift in cited excerpts.
- Fix seems to require rewriting apply as fully async background job beyond emitting `job_started` — stop and report rather than inventing a job queue.
- Changing backup/undo byte semantics.
- Touching shell plugin or docs TASKS.

## Maintenance notes

- Crash mid-batch before DB update still loses undo records — deferred finding; do not pretend this plan fixed it.
- Reviewer: watch double COMPLETE_JOB; ensure numbering tests cover pad + extension.
- Plan 003 assumes `activeJobId` is set during apply via START_JOB.
