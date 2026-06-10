# Agent Prompts — BatchRename Pro

## Global Rules

### Do
- Use Tauri 2 Rust commands for all backend logic — no server, no network required
- Use Vite 6 + React 19 + TypeScript strict + Tailwind CSS 4 for frontend
- Use rusqlite with WAL mode for SQLite storage in Tauri app data dir
- Use Rayon for parallel file processing in Rust
- Keep app bundle under 10MB with LTO and symbol stripping

### Don't
- Do NOT introduce any backend server, REST API, or network dependency
- Do NOT substitute any stack technology (no Next.js, no Electron, no Prisma)
- Do NOT use dynamic ffmpeg loading — statically link with minimal codecs only
- Do NOT add mobile responsive layouts — desktop-first only
- Do NOT use any ORM — raw rusqlite queries with migrations table

---

## Task Prompts
### Task 1: Project Scaffolding & Core Infrastructure

**Role:** Expert Tauri 2 + React 19 Full-Stack Desktop Engineer
**Goal:** Set up Tauri 2 project with typed IPC bridge, SQLite database, and themed UI shell

**Context**
Scaffold a Tauri 2 desktop app with Vite 6 bundler, React 19, TypeScript strict mode, and Tailwind CSS 4. Set up the Rust backend skeleton with SQLite database, shared IPC types, and the UI shell with dark/light theme. This is the foundation — every other task depends on it. Dark mode default: bg #0F172A, cards #1E293B, text #F1F5F9, accent #3B82F6. Navbar 48px sticky, center canvas, collapsible right sidebar, sticky action footer. Inter/system font stack.

**Files to Create**
- src-tauri/src/main.rs
- src-tauri/src/types.rs
- src-tauri/src/db.rs
- src-tauri/Cargo.toml
- src/types.ts
- src/lib/commands.ts
- src/lib/events.ts
- src/App.tsx

**Files to Modify**
- src-tauri/tauri.conf.json
- tailwind.config.ts
- tsconfig.json
- vite.config.ts
- package.json

**Steps**
1. Run `npm create tauri-app@latest batchrename-pro -- --template react-ts --manager npm` then add Tailwind CSS 4 via `npm install tailwindcss @tailwindcss/vite`. Configure tailwind.config.ts with dark mode class strategy and theme tokens: slate-950 bg, slate-800 cards, blue-500 accent, violet-500 alt-accent. Enable TypeScript strict mode with path aliases (@/ -> src/).
2. Create src-tauri/src/types.rs with serde Serialize/Deserialize structs: FileInfo (path, name, ext, file_type enum, size, thumbnail Option<String>, status enum), RenamePattern (mode enum, pattern, template, start_number, zero_pad, case_transform), ConvertOptions (target_format, quality, bitrate, resize), MetadataChanges, JobRecord, JobFileRecord. Create matching src/types.ts with identical TypeScript interfaces.
3. Create src-tauri/src/db.rs: initialize rusqlite with WAL mode in Tauri app_data_dir. Create tables: jobs (id TEXT PK, operation TEXT, status TEXT, created_at TEXT, file_count INT), job_files (id TEXT PK, job_id TEXT FK, original_path TEXT, new_path TEXT, status TEXT, error TEXT), settings (key TEXT PK, value TEXT), migrations (version INT PK, applied_at TEXT). Create FTS5 virtual table job_search over jobs. Implement migration runner, CRUD functions for history-service.
4. Create src/lib/commands.ts wrapping all Tauri invoke() calls as typed async functions: addFiles, previewRename, applyRename, convertFiles, readMetadata, writeMetadata, undoJob, getHistory, searchHistory, getSettings, updateSettings, openFilePicker, cancelJob. Create src/lib/events.ts with onJobProgress and onJobComplete typed event listeners with cleanup. Parse error strings into {code, message} objects.
5. Build UI shell in src/App.tsx: AppShell layout with sticky Navbar (48px, logo left, settings gear + help right), center DropZone canvas, collapsible right TransformationPanel sidebar (Rename|Convert|Metadata tabs), sticky ActionFooter (file counter, Apply button, history dropdown, undo). Implement ThemeContext with dark/light toggle persisted via updateSettings IPC. Use Inter font stack. Add 2px accent focus rings on all interactive elements.

**Validation**
`npm run typecheck && cd src-tauri && cargo check && cd .. && npm run build`

---

### Task 2: Drag-Drop, File List & State Management

**Role:** Expert React 19 + Tauri IPC Desktop UI Engineer
**Goal:** Build drag-drop input, Rust file validation, virtualized file list, and global state

**Context**
Implement the file input pipeline: glassmorphic drag-drop zone with animations, Rust file validation with type detection and thumbnail generation, virtualized file list with status cards, and global React state via useReducer. The drop zone should glow on drag-over with dashed border. File cards show thumbnail, original name, transformed name (accent color), format badge, and status (pending|processing|done|error). Hover shows lift effect and delete icon. Virtualize at 100+ files. Hard cap at 5000 files.

**Files to Create**
- src/components/DropZone.tsx
- src/components/FileList.tsx
- src/components/FileCard.tsx
- src/state/AppStateContext.tsx
- src/state/reducer.ts
- src/hooks/useTauriEvent.ts
- src-tauri/src/file_service.rs

**Files to Modify**
- src-tauri/src/main.rs
- src/App.tsx
- package.json

**Steps**
1. Create src/state/reducer.ts with AppState type (files: FileInfo[], previews: Map, activeJob: JobRecord|null, settings: Settings, history: JobRecord[]) and action types: ADD_FILES, REMOVE_FILE, SET_PREVIEWS, START_JOB, UPDATE_FILE_STATUS, COMPLETE_JOB, SET_SETTINGS, SET_HISTORY. Create AppStateContext.tsx with useReducer provider. Create useFileStats hook returning memoized counts by status.
2. Create src/hooks/useTauriEvent.ts: generic hook wrapping Tauri listen() with typed payload and automatic cleanup on unmount. Wire into AppStateContext to dispatch UPDATE_FILE_STATUS on job_progress events and COMPLETE_JOB on job_complete events.
3. Create src-tauri/src/file_service.rs: implement add_files command — canonicalize paths, reject nonexistent (FILE_NOT_FOUND error), detect file_type from extension mapping (audio: mp3/wav/flac/m4a, image: jpg/png/webp/avif, video: mp4/webm/mkv, document: others), generate 64x64 thumbnail data URLs for images using image crate, enforce 5000 file hard cap (TOO_MANY_FILES error). Register command in main.rs.
4. Create src/components/DropZone.tsx: glassmorphic card (backdrop-blur-md, bg-white/5, rounded-2xl, dashed border) with HTML5 onDragOver/onDrop handlers. Animated floating-files icon when empty, text 'Drag files here or click to browse'. On drag-over: border glows accent color, bg shifts lighter. On drop: call addFiles command, dispatch ADD_FILES. Click triggers openFilePicker fallback.
5. Create src/components/FileList.tsx with react-window FixedSizeList for virtualization at 100+ files. Create FileCard.tsx showing: thumbnail/type-icon, original name, transformed name in accent color, format badge pill, status indicator (pending gray, processing pulse, done green check, error red x). Hover: translateY(-2px) lift, opacity-revealed delete button. Install react-window: `npm install react-window @types/react-window`.

**Validation**
`npm run typecheck && cd src-tauri && cargo check && cd .. && npm run build`

---

### Task 3: Rename Preview, Apply Pipeline & Undo

**Role:** Expert Rust Systems + React UI Engineer
**Goal:** Build rename preview engine, apply pipeline with backups, and undo/rollback

**Context**
Implement the full rename workflow: Rust preview-service for regex/template pattern matching with conflict detection, Rename tab UI with live debounced preview, batch apply with Rayon parallel processing and backup creation, job history recording in SQLite, and undo/rollback from backups. Preview must complete under 100ms for 500 files. Templates support {date}, {number}, {original}, {ext}. Apply creates backup copies in app data dir before any rename. Undo restores originals byte-for-byte.

**Files to Create**
- src-tauri/src/preview_service.rs
- src-tauri/src/processing_pipeline.rs
- src/components/RenameTab.tsx
- src/components/ActionFooter.tsx
- src/components/HistoryDropdown.tsx
- src/hooks/useDebouncedPreview.ts

**Files to Modify**
- src-tauri/src/main.rs
- src-tauri/src/db.rs
- src/state/reducer.ts
- src/App.tsx

**Steps**
1. Create src-tauri/src/preview_service.rs: implement preview_rename command taking Vec<FileInfo> + RenamePattern. Support regex find/replace (compiled with 100ms timeout), template substitution ({date} -> YYYY-MM-DD, {number} -> zero-padded sequential, {original} -> stem, {ext} -> extension), case transforms (upper/lower/title). Detect conflicts (duplicate output names). Return Vec<PreviewResult> with original_name, new_name, has_conflict flag.
2. Create src-tauri/src/processing_pipeline.rs: implement apply_rename command. Create backup dir in app_data_dir/backups/{job_id}/. Copy originals to backup. Execute renames in parallel via Rayon thread pool. Emit job_progress event per file (file_index, total, status). Record job + job_files in SQLite via db.rs. Emit job_complete with summary. Handle partial failures per-file without crashing batch. Implement cancel_job via AtomicBool checked each iteration.
3. Implement undo_job command in processing_pipeline.rs: load job from SQLite, iterate job_files, copy backup -> original_path byte-for-byte, update job status to rolled_back. Handle missing backups gracefully (add to errors array). Reject already-rolled-back jobs with ALREADY_ROLLED_BACK error. Register all commands in main.rs.
4. Create src/components/RenameTab.tsx: regex input field with inline error display for invalid patterns, template builder row with quick-buttons ({date}, {number}, {original}) that insert into pattern field, preview toggle switch, live before/after name list. Create useDebouncedPreview hook (150ms debounce) calling previewRename command on pattern change, dispatching SET_PREVIEWS. Highlight conflicts in red.
5. Create src/components/ActionFooter.tsx: sticky bottom bar with 'X files ready' counter (left), large Apply button with accent bg + icon disabled when no files or no pattern (center), history icon opening HistoryDropdown (right), Undo button appearing after successful job. Apply button triggers applyRename, shows inline progress bar filling based on job_progress events. Create HistoryDropdown.tsx showing recent jobs with operation type icon, timestamp, file count, status badge.

**Validation**
`npm run typecheck && cd src-tauri && cargo check && cargo test && cd .. && npm run build`

---

### Task 4: Format Conversion & Metadata Editing

**Role:** Expert Rust Media Processing + React UI Engineer
**Goal:** Build audio/image/video conversion, metadata editing, and Convert/Metadata tabs

**Context**
Implement audio (MP3/WAV/FLAC/M4A via ffmpeg-next), image (JPG/PNG/WebP/AVIF via image crate), and video (MP4/WebM/MKV via ffmpeg-next) conversion services. Build Convert tab with format dropdown filtered by file type, quality slider, and advanced options. Implement metadata-service for ID3v2 tags (id3 crate) and EXIF data (kamadak-exif crate) with read, edit, and bulk strip. Build Metadata tab UI. Wire conversions into the existing processing pipeline with Rayon parallelism and backup creation.

**Files to Create**
- src-tauri/src/conversion_service.rs
- src-tauri/src/metadata_service.rs
- src/components/ConvertTab.tsx
- src/components/MetadataTab.tsx

**Files to Modify**
- src-tauri/src/main.rs
- src-tauri/src/processing_pipeline.rs
- src-tauri/Cargo.toml
- src/state/reducer.ts

**Steps**
1. Create src-tauri/src/conversion_service.rs with three modules. Audio: use ffmpeg-next for MP3↔WAV↔FLAC↔M4A with configurable bitrate (128-320kbps) and quality. Image: use image crate (pure Rust, no ffmpeg) for JPG↔PNG↔WebP↔AVIF with quality 0-100 and optional resize maintaining aspect ratio. Video: use ffmpeg-next for MP4↔WebM↔MKV with libx264/libvpx codecs, memory-mapped I/O for large files, optional VideoToolbox hardware accel on macOS. Validate format pairs, reject unsupported with UNSUPPORTED_CONVERSION.
2. Create src-tauri/src/metadata_service.rs: use id3 crate for reading/writing ID3v2 tags (title, artist, album, year, track, genre, comment) from MP3 files. Use kamadak-exif crate for reading EXIF from JPG/PNG. Implement read_metadata returning Vec<MetadataField> with name, value, editable flag. Implement write_metadata applying MetadataChanges. Implement strip operation (rewrite file without metadata). Add to Cargo.toml: id3, kamadak-exif.
3. Wire convert_files command into src-tauri/src/processing_pipeline.rs: create backups, process files in parallel via Rayon, emit job_progress per file with percentage from ffmpeg progress callback, record job in SQLite, support cancellation via AtomicBool. Register convert_files, read_metadata, write_metadata commands in main.rs.
4. Create src/components/ConvertTab.tsx: format dropdown showing only valid targets for selected file types (audio files -> audio formats, etc.), quality slider 0-100 with numeric display, collapsible advanced options section (chevron toggle with 150ms animation) for codec selection, bitrate override, and resize dimensions. Apply button triggers convertFiles IPC with assembled ConvertOptions.
5. Create src/components/MetadataTab.tsx: displays metadata fields on file selection via readMetadata IPC. Editable fields render as input controls, read-only fields as disabled. Bulk strip buttons: 'Strip EXIF' and 'Strip ID3' triggering writeMetadata with strip flags. Changes staged locally, applied via Apply button with backup and history recording through existing pipeline.

**Validation**
`npm run typecheck && cd src-tauri && cargo check && cd .. && npm run build`

---

### Task 5: Polish, Animations, Testing & CI/CD

**Role:** Expert Desktop App Polish, Testing & DevOps Engineer
**Goal:** Add animations, accessibility, settings, tests, and cross-platform CI/CD

**Context**
Finalize the app with all micro-interactions (200ms ease-out transitions, staggered file card animations, drag glow, hover scale+glow, progress bar fill, error shake, success confetti), accessibility (keyboard nav with Tab/Enter/Space, ARIA labels, focus rings, high contrast), settings modal (theme, accent color, output dir, max parallel jobs, backup retention), unit/integration tests, performance benchmarks, and GitHub Actions CI/CD for macOS/Windows/Linux with Tauri auto-updater.

**Files to Create**
- src/components/SettingsModal.tsx
- src/styles/animations.css
- .github/workflows/build.yml
- src-tauri/src/tests/mod.rs

**Files to Modify**
- src/components/FileCard.tsx
- src/components/ActionFooter.tsx
- src/components/DropZone.tsx
- src-tauri/tauri.conf.json
- src/App.tsx

**Steps**
1. Create src/styles/animations.css with utility classes: .transition-default (200ms ease-out, will-change transform/opacity), .hover-lift (translateY -2px + shadow on hover), .hover-glow (box-shadow accent/30 on hover), .shake-error (keyframe horizontal shake), .stagger-in (keyframe fadeInUp with calc(var(--i)*50ms) delay), .progress-fill (width transition 300ms), .confetti-burst (particle keyframes). Apply to DropZone (drag glow), FileCard (stagger + lift + shake), ActionFooter (progress fill + confetti).
2. Audit all components for accessibility: add aria-label to every button/input/interactive element, ensure Tab navigation order is logical, add role attributes to custom widgets, implement 2px accent focus-visible rings globally via Tailwind, add aria-live='polite' region for job status announcements, test with keyboard-only navigation. Add prefers-contrast media query for high contrast mode with boosted borders and text contrast.
3. Create src/components/SettingsModal.tsx opened from Navbar gear icon: theme toggle (dark/light), accent color picker (blue-500/violet-500/emerald-500/rose-500), default output directory selector via Tauri dialog, max parallel jobs slider (1-16 defaulting to CPU cores), auto-backup toggle, backup retention days (7/30/90/unlimited). Load via getSettings on mount, save each change via updateSettings IPC. Theme and accent changes apply immediately by updating CSS custom properties.
4. Write Rust tests in src-tauri/src/tests/: preview_service unit tests (regex, templates, numbering, conflicts, 500-file perf benchmark under 100ms), db.rs tests against in-memory SQLite (CRUD, FTS5 search, pagination, migration runner), file_service tests (type detection, hard cap enforcement), processing_pipeline integration test (rename 10 temp files with backup, verify undo restores byte-for-byte). Add frontend tests for reducer (all action types produce correct state) and commands.ts error parsing.
5. Create .github/workflows/build.yml: matrix strategy for macos-latest, windows-latest, ubuntu-latest. Steps: checkout, setup Node 20, setup Rust stable, install system deps (libwebkit2gtk-4.1-dev on Ubuntu), npm ci, npm run typecheck, cargo test, npm run build, tauri build. Upload artifacts: DMG (macOS), MSI (Windows), AppImage (Linux). Configure Tauri updater in tauri.conf.json pointing to GitHub Releases endpoint with JSON manifest.

**Validation**
`npm run typecheck && cd src-tauri && cargo test && cd .. && npm run build`