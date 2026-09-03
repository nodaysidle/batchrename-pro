# src-tauri/src — Rust backend implementation

## Purpose

Owns Tauri command handlers, native state, persistence, filesystem/system integrations, and backend tests.

## Ownership

- `db.rs`
- `file_service.rs`
- `main.rs`
- `preview_service.rs`
- `processing_pipeline.rs`
- `tests`
- `types.rs`

## Local Contracts

- Do not add Rust dependencies without explicit approval.
- Do not change signing, bundle, entitlement, or release behavior unless requested.
- Keep native commands deterministic and error paths user-visible.
- `file_registry` is 1:1 with the UI list. `add_files` inserts; `forget_files` / `clear_files` / successful undo drop ids. Session hard cap uses `registry.len()` (live files only).
- Occupancy (A→B while B→C, swaps) is a preview rule: plan a safe order or temp hop, or block with a visible reason. `apply_rename` is sequential (no Rayon `par_iter` on `fs::rename`).
- `apply_rename` starts a job under a short DB lock, returns the job id, and streams per-file progress. Cancel skips remaining work; it does not undo an in-flight `fs::rename`.

## Work Guidance

- Read this file after the root `AGENTS.md` before editing this subtree.
- Prefer extending existing modules/files over creating parallel duplicate systems.
- Update this `AGENTS.md` only when durable ownership, contracts, or verification guidance changes.

## Verification

- Rust/Tauri checks from root package/Cargo manifest when backend changes.

## Child DOX Index

None.
