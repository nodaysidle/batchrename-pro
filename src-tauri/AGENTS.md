# src-tauri — Tauri native shell

## Purpose

Owns Tauri configuration, native Rust backend, capabilities, icons, and bundle settings.

## Ownership

- `Cargo.lock`
- `Cargo.toml`
- `build.rs`
- `capabilities`
- `entitlements.plist`
- `gen`
- `icons`
- `src`
- `tauri.conf.json`

## Local Contracts

- Do not add Rust dependencies without explicit approval.
- Do not change signing, bundle, entitlement, or release behavior unless requested.
- Keep native commands deterministic and error paths user-visible.

## Work Guidance

- Read this file after the root `AGENTS.md` before editing this subtree.
- Prefer extending existing modules/files over creating parallel duplicate systems.
- Update this `AGENTS.md` only when durable ownership, contracts, or verification guidance changes.

## Verification

- Rust/Tauri checks from root package/Cargo manifest when backend changes.

## Child DOX Index

- `src-tauri/src/AGENTS.md` — Rust backend implementation.
