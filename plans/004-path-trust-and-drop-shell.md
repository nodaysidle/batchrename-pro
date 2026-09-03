# Plan 004: Re-validate paths, drop shell plugin, allowlist settings

> **Executor instructions**: Follow this plan step by step. Run every
> verification command and confirm the expected result before moving to the
> next step. If anything in the "STOP conditions" section occurs, stop and
> report — do not improvise. When done, update the status row for this plan
> in `plans/README.md` — unless a reviewer dispatched you and told you they
> maintain the index.
>
> **Drift check (run first)**: `git diff --stat 06c3362..HEAD -- src-tauri/src/main.rs src-tauri/src/file_service.rs src-tauri/src/processing_pipeline.rs src-tauri/src/preview_service.rs src-tauri/src/db.rs src-tauri/Cargo.toml src-tauri/capabilities/default.json src/lib/commands.ts package.json package-lock.json`
> If any in-scope file changed since this plan was written, compare the
> "Current state" excerpts against the live code before proceeding; on a
> mismatch, treat it as a STOP condition.

## Status

- **Priority**: P1
- **Effort**: M
- **Risk**: MED
- **Depends on**: none (complete before any convert/metadata backends)
- **Category**: security
- **Planned at**: commit `06c3362`, 2026-07-25

## Why this matters

`apply_rename` / `preview_rename` trust webview-supplied `FileInfo.original_path` without re-canonicalizing (only `add_files` does). A compromised webview or buggy client can point rename/backup at arbitrary paths the OS user can write. The app also enables unused `tauri-plugin-shell` + `shell:default`, widening the capability surface. `update_settings` writes arbitrary keys including unclamped `file_hard_cap`.

## Current state

```77:93:src-tauri/src/main.rs
async fn apply_rename(..., files: Vec<FileInfo>, ...) {
    let filtered: Vec<FileInfo> = files
        .into_iter()
        .filter(|f| file_ids.contains(&f.id))
        .collect();
    let job_id = processing_pipeline::execute_batch_rename(&app, conn, filtered, pattern)?;
```

```37:40:src-tauri/src/file_service.rs
    let canonical = path
        .canonicalize()
        .map_err(|_| format!("FILE_NOT_FOUND: {}", path_str))?;
```

```161:161:src-tauri/src/main.rs
        .plugin(tauri_plugin_shell::init())
```

```25:28:src-tauri/capabilities/default.json
    "dialog:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "shell:default"
```

```12:12:src-tauri/Cargo.toml
tauri-plugin-shell = "2"
```

```138:148:src-tauri/src/main.rs
async fn update_settings(settings: HashMap<String, String>, ...) {
    for (key, value) in &settings {
        db::set_setting(conn, key, value)?;
    }
```

Frontend does **not** import `@tauri-apps/plugin-shell` (verified at plan time). Dialog picking uses `@tauri-apps/plugin-dialog` in `DropZone.tsx`.

**Conventions**: Keep `CODE: message` errors. Prefer smallest correct change — session map in `AppState` is OK; do not add network services. Do not add dependencies; **removing** shell is required.

## Commands you will need

| Purpose | Command | Expected on success |
|---------|---------|---------------------|
| Rust tests | `cd src-tauri && cargo test` | all pass |
| Typecheck | `npm run typecheck` | exit 0 |
| Grep shell FE | `rg "plugin-shell|shell:" src/ src-tauri/capabilities src-tauri/src/main.rs src-tauri/Cargo.toml` | no shell plugin usage after change |
| Build FE | `npm run build` | exit 0 |

## Scope

**In scope**:
- `src-tauri/src/main.rs`
- `src-tauri/src/file_service.rs` (helpers for revalidate if needed)
- `src-tauri/src/processing_pipeline.rs` / `preview_service.rs` only if signatures change
- `src-tauri/src/types.rs` if new structs needed
- `src-tauri/src/tests/mod.rs`
- `src-tauri/Cargo.toml` / `Cargo.lock` (via cargo remove)
- `src-tauri/capabilities/default.json`
- `src/lib/commands.ts` only if IPC args change (prefer keep FE API stable by resolving paths server-side)
- `package.json` / lockfile **only if** `@tauri-apps/plugin-shell` is listed (check; may be absent on FE)
- `plans/README.md`

**Out of scope**:
- Thumbnail bomb limits (related security; not selected A–E — do not expand unless trivial one-liner while touching `file_service`)
- Regex timeout
- UI polish / History undo
- Deleting `open_file_picker` noop (optional cleanup — allowed only as a tiny extra if you already touch `main.rs` invoke list; not required)

## Git workflow

- Branch: `advisor/004-path-trust-and-drop-shell`
- Commit example: `fix: revalidate rename paths and remove unused shell plugin`
- Do NOT push unless instructed.

## Steps

### Step 1: Backend session registry for trusted files

In `AppState` (`main.rs`), add a `Mutex<HashMap<String, FileInfo>>` (or path-only map) populated by successful `add_files` entries (keyed by `file.id`).

On `preview_rename` and `apply_rename`:
1. For each requested `file_id`, look up the **trusted** `FileInfo` from the map.
2. If missing → error `UNKNOWN_FILE: ...`.
3. Optionally re-`canonicalize` the stored path and verify it still exists before preview/apply; if the client sent a `FileInfo`, **ignore** client `original_path` and use the server copy.
4. On successful rename completion, update the map entry’s `original_path` / `original_name` to the new path (so a second apply in the same session works even if FE is buggy — complements plan 002).
5. On remove — FE does not notify Rust today. Accept that map can grow until app restart **or** add a tiny `forget_files` / clear on empty — prefer: update paths on apply; leave stale ids until restart unless you add `remove_files` command (out of scope). Document in Maintenance.

**Verify**: Add a unit/integration-style test if feasible (registry helper test), or a processing test that apply uses canonical paths. `cargo test` passes.

### Step 2: Remove shell plugin

1. Remove `.plugin(tauri_plugin_shell::init())` from `main.rs`.
2. Remove `"shell:default"` from `capabilities/default.json`.
3. Remove `tauri-plugin-shell` from `Cargo.toml` and refresh lockfile with `cargo remove tauri-plugin-shell` inside `src-tauri` (network may be needed for cargo; if blocked, edit Cargo.toml and run `cargo generate-lockfile` / `cargo check`).
4. Confirm FE has no shell imports.

**Verify**: `rg "tauri_plugin_shell|shell:default|plugin-shell" src-tauri src` → no matches (except maybe lockfile transitive — dependency line must be gone from Cargo.toml). `cd src-tauri && cargo test` → pass.

### Step 3: Allowlist + clamp `update_settings`

Allow only known keys matching `Settings` / defaults used in setup:
`theme`, `accent_color`, `default_output_dir`, `max_parallel_jobs`, `auto_backup`, `backup_retention_days`, `last_rename_pattern`, `last_convert_format`, `file_hard_cap`.

For each value:
- `theme`: `dark`|`light`
- `accent_color`: `blue`|`violet` (expand later only if UI expands)
- `max_parallel_jobs`: parse u32, clamp 1..=16
- `backup_retention_days`: clamp to a sane set or 0..=3650
- `file_hard_cap`: clamp e.g. 1..=5000 (or 1..=10000 max)
- booleans: `true`/`false` only
- Reject unknown keys with `INVALID_SETTING: ...`

**Verify**: Add a small Rust test that unknown key errors and hard_cap clamps — if testing commands is hard, test a pure `fn validate_setting(key, value) -> Result<(), String>` extracted to `db.rs` or `main.rs` module. `cargo test` passes.

### Step 4: Update index

Mark 004 DONE in `plans/README.md`.

## Test plan

- New tests: settings validate/clamp; if possible, apply rejects unknown file id not in registry (may need refactor to testable fn).
- Existing rename/undo tests must pass.
- Pattern: `src-tauri/src/tests/mod.rs`.

## Done criteria

- [ ] `preview_rename` / `apply_rename` use server-trusted paths (not client-supplied path strings)
- [ ] Unknown file ids rejected
- [ ] `tauri-plugin-shell` removed from Cargo.toml; `shell:default` removed; plugin init removed
- [ ] `update_settings` allowlists keys and clamps `file_hard_cap` / parallel jobs
- [ ] `cd src-tauri && cargo test` and `npm run typecheck` exit 0
- [ ] Scope respected; README DONE

## STOP conditions

- FE somehow requires shell open for Help links — not present today; if discovered, stop and report.
- Path policy for symlinks unclear (canonicalize already used in add_files) — keep same policy; do not invent sandboxing.
- Drift / double verification failure.

## Maintenance notes

- Convert backends must register outputs into the same trusted map.
- Reviewer: confirm no client path used in `fs::rename` / `create_backup`.
- Map growth across long sessions is acceptable for v0.1; add forget API later if needed.
