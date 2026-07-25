# Plan 005: Docs honesty + session-level file hard cap

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**: `git diff --stat 06c3362..HEAD -- AGENTS.md docs/internal/TASKS.md README.md src-tauri/src/main.rs src-tauri/src/file_service.rs src-tauri/src/tests/mod.rs src/lib/commands.ts src/components/DropZone.tsx src/state/AppStateContext.tsx`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P2
- **Effort**: S
- **Risk**: LOW
- **Depends on**: none (can run anytime)
- **Category**: docs / bug
- **Planned at**: commit `06c3362`, 2026-07-25

## Why this matters

`docs/internal/TASKS.md` marks Convert/Metadata/SettingsModal/animations/Windows CI/updater as ✅ while the live tree has stubs and missing modules — agents treat unfinished work as shipped. Separately, `add_files` resets `current_count` to 0 each invoke, so repeated drops can exceed the advertised 5,000 file hard cap in one session.

## Current state

```32:39:src-tauri/src/main.rs
    let mut files = Vec::new();
    let mut current_count = 0u32;

    for path in &paths {
        match file_service::validate_and_build_file_info(path, hard_cap, current_count) {
            Ok(file_info) => {
                current_count += 1;
```

```33:34:src-tauri/src/file_service.rs
    if current_count >= hard_cap {
        return Err(format!("TOO_MANY_FILES: Exceeds hard cap of {}", hard_cap));
```

```47:48:src/lib/commands.ts
export async function addFiles(paths: string[]): Promise<AddFilesResponse> {
  return safeInvoke('add_files', { paths });
```

`docs/internal/TASKS.md` Phase “Format Conversion” / “Polish” items use `### ✅` for conversion_service, metadata, SettingsModal, animations, Windows CI (e.g. lines ~241–434 region).

Root `AGENTS.md` Task 4–5 still instruct creating missing files as if scaffolding from scratch (acceptable as historical prompts) but should gain a short **Current ship status** note so agents do not assume they exist.

README already says Convert/Metadata coming soon — keep aligned; fix architecture bullets that claim FTS5 search / Convert services if still overstated (`README.md` stack/architecture sections).

**Conventions**: DOX — update closest `AGENTS.md` only for durable contract changes; do not write diary notes. Prefer marking tasks `⏸️ Deferred (v0.1.0 rename-only)` rather than deleting history. Commit style: `docs: ...` / `fix: ...`.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Docs AGENTS list | `find . -name AGENTS.md -not -path './.git/*' -not -path './node_modules/*' -not -path './src-tauri/target/*' \| sort` | lists expected DOX files |
| Rust tests | `cd src-tauri && cargo test` | pass including new hard-cap test |
| Typecheck | `npm run typecheck` | exit 0 |
| Status | `git status --short` | only docs + hard-cap related files + plans |

## Scope

**In scope**:
- `docs/internal/TASKS.md` — flip false ✅ for unshipped Task 4–5 items to deferred/incomplete
- Root `AGENTS.md` — add a concise “v0.1.0 ship status” callout near Task 4–5 (do not rewrite entire task prompts unless necessary)
- `README.md` — only if architecture/stack claims still contradict rename-only (FTS actively used, Convert services listed as implemented)
- `src-tauri/src/main.rs` — accept `existing_count: u32` (or similar) on `add_files`
- `src/lib/commands.ts` — pass existing count
- `src/components/DropZone.tsx` — pass `state.files.length`
- Optionally `src-tauri/src/file_service.rs` if signature docs need clarity
- `src-tauri/src/tests/mod.rs` — hard-cap session test
- `plans/README.md`

**Out of scope**:
- Implementing convert/metadata/SettingsModal/animations
- Wiring FTS MATCH search
- UI polish, path trust, History undo
- Changing default hard_cap value in settings (clamp is plan 004)

## Git workflow

- Branch: `advisor/005-docs-honesty-and-session-hardcap`
- Commits: `docs: mark deferred Task 4-5 items for v0.1.0` and/or `fix: enforce session file hard cap across add_files`
- Do NOT push unless instructed.

## Steps

### Step 1: Correct TASKS.md false checkmarks

In `docs/internal/TASKS.md`, for items that are **not** in the live tree (audio/video/image conversion services as shipped, Convert tab full UI, metadata service, SettingsModal, animations.css polish as complete, Windows CI, auto-updater, frontend test suite, etc.):

- Replace `### ✅` with `### ⏸️ Deferred` (or `### ❌ Not shipped in v0.1.0`) and one-line note: “Rename-only release; see README.”

Do **not** mark shipped rename/preview/undo/CI-mac-linux items as deferred.

**Verify**: `rg "✅ Implement audio conversion|✅ Implement video conversion|✅ Implement settings persistence|✅ Implement micro-interactions" docs/internal/TASKS.md` → no matches (or only if truly shipped). Prefer `rg "Deferred|Not shipped" docs/internal/TASKS.md` → shows updates.

### Step 2: AGENTS.md ship-status callout

Near Task 4 / Task 5 in root `AGENTS.md`, add a short blockquote or subsection:

> **v0.1.0 reality check**: Convert/Metadata backends, `SettingsModal.tsx`, `src/styles/animations.css`, frontend Vitest suite, Windows CI, and updater are **not** shipped. Treat Task 4–5 prompts as backlog, not acceptance. Live product is rename + backup + undo.

Do not delete the historical task prompts (they remain useful backlog).

**Verify**: `rg "v0.1.0 reality check|not shipped" AGENTS.md` → match.

### Step 3: README alignment (only if needed)

If README architecture still claims working Convert/Metadata **services** or active FTS search UX, edit those bullets to “schema-ready / coming soon” to match `README` release notes. Do not invent new marketing.

**Verify**: Skim `README.md` “Stack” / architecture — no false “services exist” claims.

### Step 4: Session hard cap on `add_files`

Change Rust command signature to accept existing session count, e.g.:

```rust
async fn add_files(paths: Vec<String>, existing_count: u32, state: ...) 
```

Initialize `current_count = existing_count` before the loop (still enforce `validate_and_build_file_info(..., hard_cap, current_count)`).

Update `commands.ts`:

```ts
export async function addFiles(paths: string[], existingCount = 0): Promise<AddFilesResponse> {
  return safeInvoke('add_files', { paths, existingCount }); // serde: existing_count
}
```

Tauri/serde: use `existing_count` in Rust with `#[serde(default)]` if needed; ensure JS passes `existingCount` mapped correctly — Tauri 2 typically expects camelCase in JS → snake_case in Rust via rename, **or** pass `existing_count` explicitly in the invoke args object as `{ paths, existing_count: existingCount }`.

Update `DropZone.tsx` `handleFiles` to pass `state.files.length` (get from `useAppState()`).

Add Rust test: with `hard_cap` 2, `existing_count` 2, adding another path returns `TOO_MANY_FILES` (may test `validate_and_build_file_info` directly with current_count=2, hard_cap=2 — already enough if command wiring is thin). Also test command-level if easy.

**Verify**: `cd src-tauri && cargo test` → pass. `npm run typecheck` → pass. `rg "existing_count|existingCount" src-tauri/src/main.rs src/lib/commands.ts src/components/DropZone.tsx` → matches.

### Step 5: Update index

Mark 005 DONE in `plans/README.md`.

**Verify**: `git status --short` → only in-scope files.

## Test plan

- New test: `file_service` or command path — `current_count >= hard_cap` rejects (extend `tests/mod.rs`).
- Docs verification commands from AGENTS docs ladder: `find … AGENTS.md` + `git status --short`.

## Done criteria

- [ ] Unshipped Task 4–5 items in `TASKS.md` no longer show false ✅ as complete
- [ ] `AGENTS.md` contains explicit v0.1.0 “not shipped” callout for Task 4–5 artifacts
- [ ] `add_files` accounts for `existing_count` / session size
- [ ] DropZone passes current file count
- [ ] `cargo test` + `npm run typecheck` exit 0
- [ ] No product features beyond hard-cap + docs
- [ ] README status DONE

## STOP conditions

- Editing TASKS would require rewriting the entire PRD — keep changes minimal; stop if asked to implement convert to “make ✅ true.”
- Serde rename for `existing_count` unclear after one attempt — stop and report invoke arg mismatch rather than guessing multiple API shapes.
- Drift / double verification failure.

## Maintenance notes

- When Convert ships, flip deferred TASKS items forward carefully and remove the AGENTS reality-check or update it.
- Reviewer: ensure hard_cap still defaults to 5000; session count cannot be spoofed downward without also spoofing — **note**: client can lie about `existing_count` low. For stronger enforcement, combine with plan 004 server-side registry length as the source of truth (`existing_count = map.len()` and **ignore** client count). **Preferred hardening in this plan if 004 already landed**: use registry size instead of client-provided count. If 004 not landed yet, client-provided count is an improvement over per-batch zero; document the spoof residual in Maintenance.

**Executor preference**: If `plans/004` is DONE and a trusted file map exists, set `current_count` from `map.len()` and ignore client `existing_count` (keep param for compatibility or remove). If 004 not done, use client `existing_count` from DropZone.
