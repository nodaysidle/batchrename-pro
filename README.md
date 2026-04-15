# BatchRename Pro

Batch file renaming, format conversion, and metadata editing — one local-first desktop app. No cloud, no scripts, no juggling tools.

Built with Tauri 2. Rust backend. React frontend. Ships under 11MB.

![Platform](https://img.shields.io/badge/macOS-ARM64-blue) ![Platform](https://img.shields.io/badge/Windows-x64-blue) ![Platform](https://img.shields.io/badge/Linux-x64-blue)

## What it does

**Batch Rename** — Regex patterns, template builder with quick-insert tokens (`{date}`, `{number}`, `{original}`, `{ext}`), sequential numbering with zero-padding, case transforms. Live preview before committing.

**Format Conversion** — Audio (MP3, WAV, FLAC, M4A), Image (JPG, PNG, WebP, AVIF), Video (MP4, WebM, MKV). Quality controls, parallel processing via Rayon.

**Metadata Editing** — ID3 tag read/write for audio. EXIF read/strip for images. One-click bulk strip.

**Undo Everything** — Every operation creates backups before touching your files. Full undo/rollback from SQLite-backed job history.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  WebView (React 19 + TypeScript + Tailwind CSS) │
│  ┌──────────┬──────────────┬──────────────────┐ │
│  │ DropZone │  FileList    │ TransformPanel   │ │
│  │          │  (virtualized│  Rename│Convert  │ │
│  │          │   100+ files)│  │Metadata       │ │
│  └──────────┴──────────────┴──────────────────┘ │
│                  ActionFooter                    │
└──────────────────┬──────────────────────────────┘
                   │ Tauri IPC
┌──────────────────┴──────────────────────────────┐
│  Rust Backend                                   │
│  ┌──────────┬──────────┬────────────────────┐   │
│  │ Preview  │ File     │ Processing         │   │
│  │ Service  │ Service  │ Pipeline (Rayon)   │   │
│  │          │          │                    │   │
│  │ Regex    │ Backup   │ Parallel rename    │   │
│  │ Template │ Restore  │ Progress events    │   │
│  │ Numbering│ Validate │ Job cancellation   │   │
│  └──────────┴──────────┴────────────────────┘   │
│  ┌──────────┬──────────┬────────────────────┐   │
│  │ SQLite   │ Convert  │ Metadata           │   │
│  │ (WAL)    │ Service  │ Service            │   │
│  │          │          │                    │   │
│  │ History  │ ffmpeg   │ ID3 / EXIF         │   │
│  │ FTS5     │ image    │ Read / Write       │   │
│  │ Settings │ crate    │ Strip              │   │
│  └──────────┴──────────┴────────────────────┘   │
└─────────────────────────────────────────────────┘
```

## Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri 2 |
| Frontend | Vite 6 + React 19 + TypeScript strict |
| Styling | Tailwind CSS 4 |
| Backend | Rust 2021 |
| Database | SQLite via rusqlite (WAL mode) |
| Parallelism | Rayon thread pool |
| Image processing | image crate (pure Rust) |
| Media conversion | ffmpeg-next bindings |
| Search | SQLite FTS5 |
| Icons | Lucide React |

## Build

```bash
# Prerequisites
# - Node 20+
# - Rust stable 1.75+
# - Xcode Command Line Tools (macOS)

# Install frontend dependencies
npm install

# Development
npx tauri dev

# Production build
npm run build && npx tauri build
```

The release binary is at `src-tauri/target/release/bundle/macos/BatchRename Pro.app` on macOS.

## Performance

| Metric | Target | Actual |
|--------|--------|--------|
| Rename preview (500 files) | < 100ms | ✅ in-memory, no disk I/O |
| App bundle size | < 10MB | ~11MB |
| Cold start | < 2s | ✅ |
| File hard cap | 5,000 | enforced on add |

## UI

Dark mode default. Glassmorphic design. Two accent themes — blue and violet.

- 48px sticky navbar with accent color toggle
- Drag-drop zone with animated states
- Virtualized file list (react-window) for 100+ files
- Collapsible right sidebar with Rename / Convert / Metadata tabs
- Sticky action footer with Apply, Undo, and History

## Project structure

```
├── src/                          # React frontend
│   ├── components/               # UI components
│   │   ├── DropZone.tsx          # File input (drag-drop + native picker)
│   │   ├── FileList.tsx          # Virtualized file list
│   │   ├── FileCard.tsx          # Individual file display
│   │   ├── RenameTab.tsx         # Rename pattern builder
│   │   ├── ConvertTab.tsx        # Format conversion UI
│   │   ├── MetadataTab.tsx       # Metadata editor UI
│   │   ├── TransformationPanel.tsx # Right sidebar tabs
│   │   ├── ActionFooter.tsx      # Bottom bar + history
│   │   └── Navbar.tsx            # Top bar + settings
│   ├── contexts/                 # React context providers
│   ├── hooks/                    # Custom hooks
│   ├── lib/                      # IPC command wrappers
│   ├── state/                    # useReducer state management
│   └── types.ts                  # TypeScript interfaces
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri commands + setup
│   │   ├── types.rs              # Shared Rust types (serde)
│   │   ├── db.rs                 # SQLite migrations + CRUD
│   │   ├── file_service.rs       # File validation + backup
│   │   ├── preview_service.rs    # Rename pattern engine
│   │   └── processing_pipeline.rs # Rayon parallel executor
│   ├── Cargo.toml
│   └── tauri.conf.json
├── PRD.md                        # Product requirements
├── TRD.md                        # Technical requirements
├── ARD.md                        # Architecture decisions
├── TASKS.md                      # Task breakdown
└── AGENTS.md                     # Agent instructions
```

## Status

MVP: Rename engine fully functional. Drag-drop, live preview, apply with backup, undo, job history, accent themes.

In progress: Format conversion (audio/image/video), metadata editing (ID3/EXIF).

## License

NODAYSIDLE. No days idle.

---

Built by [Punto](https://gitlab.com/NODAYSIDLE) — NODAYSIDLE's AI agent, and the mysterious anonymous 3rd partner nobody knows about yet.
