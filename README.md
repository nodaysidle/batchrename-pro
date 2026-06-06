<p align="center">
  <strong>BatchRename Pro</strong>
</p>

<p align="center">
  <strong>Batch rename files safely in one local-first desktop app. No cloud. No scripts.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-macOS%20%7C%20Windows%20%7C%20Linux-black?style=flat-square&logo=apple&logoColor=white" alt="Platform">
  <img src="https://img.shields.io/badge/Tauri-2.0-24C8DB?style=flat-square&logo=tauri&logoColor=white" alt="Tauri">
  <img src="https://img.shields.io/badge/Rust-2021-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust">
  <img src="https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black" alt="React">
  <img src="https://img.shields.io/badge/DMG-6.9MB-brightgreen?style=flat-square" alt="DMG Size">
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License">
</p>

---

BatchRename Pro handles the file renaming operations you do too often to keep doing manually. Rename hundreds of files by regex or template in a dark-mode desktop app that ships as a 6.9MB macOS DMG. Rename operations are backed up and undoable.

---

## Download

Download the latest macOS DMG from the GitHub release page:

- [BatchRename Pro 0.1.0 release](https://github.com/nodaysidle/batchrename-pro/releases/tag/v0.1.0)
- Download file: `BatchRename-Pro-0.1.0-aarch64.dmg`
- SHA256: `ef6e33a03881430c329fd9fd888cf4010142598010a89b535cf0eb2c3948309b`

Install: open the DMG and drag `BatchRename Pro.app` to `/Applications`.

This release is ad-hoc signed, not Apple-notarized. If macOS blocks first launch, right-click the app and choose **Open**.

---

## Release Notes

### v0.1.0

- First public macOS release.
- Rename workflow complete: picker, drag-drop input, live preview, conflict blocking, apply with backup, undo, and job history.
- Verified with TypeScript typecheck, production build, Rust test suite, strict codesign check, DMG verification, and launch smoke test.
- Format conversion and metadata editing tabs are present but disabled until fully implemented.

---

## What It Does

**Batch Rename** — Regex patterns, template tokens (`{date}` as `YYYY-MM-DD`, `{number}`, `{original}`, `{ext}`), sequential numbering with zero-padding, case transforms. Live preview before anything touches disk.

**Format Conversion** — Coming soon. Disabled in the current release build.

**Metadata Editing** — Coming soon. Disabled in the current release build.

**Undo Everything** — Every operation creates a backup before execution. Full rollback from SQLite-backed job history.

---

## Stack

- **Framework:** Tauri 2
- **Frontend:** Vite 6 + React 19 + TypeScript strict + Tailwind CSS 4
- **Backend:** Rust 2021 + Rayon (parallel processing)
- **Database:** SQLite via rusqlite (WAL mode) + FTS5
- **Media:** image crate for local thumbnails; conversion features remain disabled in v0.1.0

---

## Building from Source

```bash
git clone https://github.com/nodaysidle/batchrename-pro.git
cd batchrename-pro
```

**Prerequisites:** Node 20+, Rust stable 1.75+, Xcode CLI Tools (macOS)

```bash
npm install

# Development
npx tauri dev

# Production build
npm run build && npx tauri build
```

Release binary: `src-tauri/target/release/bundle/macos/BatchRename Pro.app`

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│  WebView (React 19 + TypeScript + Tailwind CSS) │
│  DropZone │ FileList (virtualized) │ TransformPanel │
│                   ActionFooter                   │
└──────────────────┬──────────────────────────────┘
                   │ Tauri IPC
┌──────────────────┴──────────────────────────────┐
│  Rust Backend                                   │
│  Preview Service │ File Service │ Processing Pipeline (Rayon) │
│  SQLite (WAL)    │ Convert Service │ Metadata Service │
└─────────────────────────────────────────────────┘
```

---

## UI

Dark mode default. Glassmorphic design. Two accent themes — blue and violet.

- Drag-drop zone with animated states
- Virtualized file list (react-window) for 100+ files
- Collapsible right sidebar: Rename / Convert / Metadata tabs
- Sticky action footer with Apply, Undo, History

---

## Performance

- Rename preview (500 files): in-memory, no disk I/O — < 100ms
- App bundle: ~16MB; compressed DMG: 6.9MB
- Cold start: < 2s
- File hard cap: 5,000 files when adding items

---

## Status

MVP: Rename workflow complete. Picker and drag-drop input, live preview, conflict blocking, apply with backup, undo, job history, accent themes.

Disabled until fully implemented: Format conversion (audio/image/video), metadata editing (ID3/EXIF).

---

<p align="center">
  Built by <a href="https://github.com/nodaysidle">NODAYSIDLE</a>
</p>
