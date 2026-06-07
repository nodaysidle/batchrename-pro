# Tasks Plan — BatchRename Pro

## 📌 Global Assumptions
- ffmpeg is statically linked into the Tauri binary for zero user setup (accept larger bundle)
- macOS is the primary development platform; Windows and Linux tested in CI
- Node 20+ and Rust stable 1.75+ are available in the development environment
- SQLite WAL mode is supported on all target platforms
- Tauri 2 stable API is used without beta features
- React 19 stable with concurrent features is available via Vite 6
- Backup files are stored in the Tauri app data directory, not alongside source files

## ⚠️ Risks
- Static ffmpeg linking may push bundle size over the 10MB target, especially with video codecs. Mitigation: include only required codecs (libmp3lame, libvorbis, libvpx, libx264). — high
- ffmpeg-next Rust bindings may have cross-compilation issues on Windows/Linux. Mitigation: test CI matrix early in Phase 1 and have fallback to CLI ffmpeg subprocess. — high
- Video conversion with hardware acceleration is platform-specific and difficult to test uniformly. Mitigation: make hardware accel opt-in, default to software encoding. — medium
- Large file batches (1000+ files) may cause frontend state performance issues despite virtualization. Mitigation: benchmark with 5000 files early, optimize reducer with immer if needed. — medium
- macOS notarization and Windows code signing require paid certificates and add CI complexity. Mitigation: set up signing early in dev cycle, use GitHub Secrets for credentials. — low

## 🧩 Epics
## Project Scaffolding & Core Infrastructure
**Goal:** Set up Tauri 2 project with Vite 6 + React 19 frontend, Rust backend skeleton, SQLite database, IPC bridge types, and UI shell with dark theme.

### User Stories
_None_

### Acceptance Criteria
_None_

### ✅ Initialize Tauri 2 + Vite 6 + React 19 project (4h)

Scaffold Tauri 2 app with Vite 6 bundler, React 19 frontend, TypeScript strict mode, Tailwind CSS 4 with theme tokens for dark/light mode and accent colors.

**Acceptance Criteria**
- tauri dev launches a window with React rendering
- Tailwind CSS 4 dark theme tokens applied (slate-950 bg, slate-800 cards, blue-500 accent)
- TypeScript strict mode enabled with path aliases configured
- Bundle builds for macOS without errors

**Dependencies**
_None_

### ✅ Define shared TypeScript types and Rust serde structs (3h)

Create types.ts with all IPC payload interfaces (FileInfo, RenamePattern, ConvertOptions, MetadataChanges, job types) and matching Rust structs with serde Serialize/Deserialize.

**Acceptance Criteria**
- types.ts exports all interfaces matching TRD API contracts
- Rust structs in src-tauri/src/types.rs compile and round-trip through serde_json
- No type duplication between commands.ts and components

**Dependencies**
- Initialize Tauri 2 + Vite 6 + React 19 project

### ✅ Implement tauri-ipc-layer (commands.ts + events.ts) (3h)

Wrap all Tauri invoke() calls in typed async functions with structured error parsing. Wrap listen() subscriptions with typed payloads and cleanup functions.

**Acceptance Criteria**
- commands.ts exports typed functions for all 13 IPC commands
- events.ts exports onJobProgress and onJobComplete with typed callbacks
- Error strings parsed into {code, message} objects
- All functions have JSDoc comments matching TRD

**Dependencies**
- Define shared TypeScript types and Rust serde structs

### ✅ Build SQLite database with migrations and history-service (5h)

Initialize rusqlite with WAL mode, create jobs, job_files, settings, migrations tables, FTS5 virtual table, and indexes. Implement history-service CRUD and migration runner.

**Acceptance Criteria**
- Database created in Tauri app data dir on first launch
- All tables, indexes, and FTS5 virtual table created per TRD schema
- history-service functions pass unit tests against in-memory SQLite
- Forward-only migrations tracked in migrations table

**Dependencies**
- Initialize Tauri 2 + Vite 6 + React 19 project

### ✅ Build UI shell with navbar, layout, and theme switching (5h)

Implement AppShell, Navbar, ThemeContext, and global layout structure with sticky navbar (48px), center canvas, collapsible right sidebar, sticky action footer. Inter/system font stack.

**Acceptance Criteria**
- Navbar renders with logo, settings gear, and help icon
- Dark/light theme toggle persists via update_settings IPC
- Layout matches spec: center canvas, right sidebar, sticky footer
- Focus rings (2px accent) visible on all interactive elements via keyboard nav

**Dependencies**
- Implement tauri-ipc-layer (commands.ts + events.ts)

## Drag-Drop & File Management
**Goal:** Enable users to add files via drag-drop or file picker, display file list with thumbnails and status, and manage file state.

### User Stories
_None_

### Acceptance Criteria
_None_

### ✅ Implement drag-drop zone component (4h)

Build glassmorphic DropZone with HTML5 drag-and-drop handlers, animated file icon when empty, border glow on drag-over, click-to-browse fallback via open_file_picker.

**Acceptance Criteria**
- Drop zone renders with dashed border, glassmorphism card style, and animated icon
- Dragging files over zone triggers glow effect and bg shift
- Dropping files invokes add_files IPC and populates file list
- Click triggers native file picker dialog

**Dependencies**
- Build UI shell with navbar, layout, and theme switching

### ✅ Implement Rust add_files command and file-service validation (5h)

Build add_files command handler that validates paths, detects file types from extension/magic bytes, generates thumbnails for images (64x64), returns FileInfo array. Enforce 5000 file hard cap.

**Acceptance Criteria**
- Paths validated with canonicalize, nonexistent paths return FILE_NOT_FOUND
- File type detected correctly for audio, image, video, document extensions
- Image thumbnails generated as data URLs at 64x64
- Files exceeding hard cap rejected with TOO_MANY_FILES error

**Dependencies**
- Build SQLite database with migrations and history-service

### ✅ Build file list with virtualized rendering (5h)

Implement horizontally scrollable FileList with FileCard components showing thumbnail, original name, transformed name, format badge, and status indicator. Use react-window for 100+ files.

**Acceptance Criteria**
- File cards display all required fields per TRD spec
- Hover state shows lift effect and delete icon
- Virtualized rendering activates at 100+ files without DOM bloat
- File removal updates state and removes card with animation

**Dependencies**
- Implement Rust add_files command and file-service validation
- Implement drag-drop zone component

### ✅ Implement global state management with useReducer (4h)

Build AppStateContext with useReducer for files, previews, activeJob, settings, history. Define all action types. Wire useTauriEvent for job_progress/job_complete. Memoize derived stats.

**Acceptance Criteria**
- AppStateContext provides state and dispatch to all children
- All action types (ADD_FILES, REMOVE_FILE, SET_PREVIEWS, START_JOB, UPDATE_FILE_STATUS, COMPLETE_JOB) produce correct state
- useTauriEvent subscribes and cleans up Tauri events on unmount
- useFileStats returns memoized counts by status

**Dependencies**
- Implement tauri-ipc-layer (commands.ts + events.ts)

## Rename Preview & Apply with Undo
**Goal:** Implement full rename workflow: regex/template pattern input, live preview, batch apply with backup, job history recording, and undo/rollback.

### User Stories
_None_

### Acceptance Criteria
_None_

### ✅ Build preview-service in Rust (5h)

Implement regex application, template substitution ({date}, {number}, {original}, {ext}), sequential numbering with zero-pad, case transforms, and conflict detection. Pure in-memory, no disk I/O.

**Acceptance Criteria**
- Regex find/replace works correctly with compiled patterns and 100ms timeout
- Template variables substitute correctly including {date} and {number} with zero-padding
- Conflict detection identifies duplicate output names within batch
- Preview for 500 files completes under 100ms in release build

**Dependencies**
- Define shared TypeScript types and Rust serde structs

### ✅ Build Rename tab in transformation panel (5h)

Implement RenameTab with regex input field, template builder, quick-buttons ({date}, {number}, {original}), preview toggle, and live before/after list. Debounce preview_rename calls at 150ms.

**Acceptance Criteria**
- Pattern input triggers debounced preview_rename IPC on every keystroke
- Live preview list shows before/after names with conflicts highlighted in red
- Template quick-buttons insert variables into pattern field
- Invalid regex shows inline error below input field

**Dependencies**
- Build preview-service in Rust
- Implement global state management with useReducer

### ✅ Implement apply_rename with backup and progress events (6h)

Build processing pipeline for batch rename: create backups, execute renames via Rayon thread pool, emit job_progress events per file, record job in SQLite, emit job_complete.

**Acceptance Criteria**
- Backup copies created in app data dir before any rename
- Renames execute in parallel via Rayon with per-file progress events
- Job and file records inserted into SQLite with correct statuses
- Partial failures tracked per-file without crashing the batch

**Dependencies**
- Build preview-service in Rust
- Build SQLite database with migrations and history-service

### ✅ Build action footer with Apply, progress bar, and history dropdown (5h)

Implement sticky ActionFooter with file counter, Apply button (disabled when invalid), inline progress bar during jobs, history dropdown showing recent jobs, and Undo button post-apply.

**Acceptance Criteria**
- Apply button disabled when no files or no pattern configured
- Progress bar fills smoothly during active job based on job_progress events
- History dropdown lists recent jobs with operation type, timestamp, and file count
- Undo button appears after successful job and triggers undo_job IPC

**Dependencies**
- Implement apply_rename with backup and progress events
- Build Rename tab in transformation panel

### ✅ Implement undo_job with backup restoration (4h)

Build undo_job command that restores all original files from backups, marks job as rolled_back in SQLite, handles partial restores when some backups are missing.

**Acceptance Criteria**
- Undo restores original files byte-for-byte from backup copies
- Job status updated to rolled_back in SQLite after undo
- Missing backups reported in errors array without crashing
- Already-rolled-back jobs return ALREADY_ROLLED_BACK error

**Dependencies**
- Implement apply_rename with backup and progress events

## Format Conversion
**Goal:** Enable audio, image, and video format conversion with quality controls, parallel processing, and progress tracking.

### User Stories
_None_

### Acceptance Criteria
_None_

### ✅ Implement audio conversion service (MP3, WAV, FLAC, M4A) (6h)

Build conversion-service audio module using ffmpeg-next crate bindings. Support MP3↔WAV↔FLAC↔M4A with configurable bitrate/quality. Validate format pairs.

**Acceptance Criteria**
- MP3↔WAV conversion produces valid output files
- FLAC and M4A conversions work with correct codec parameters
- Quality/bitrate setting applied correctly to output
- Invalid source→target pairs rejected with UNSUPPORTED_CONVERSION

**Dependencies**
- Define shared TypeScript types and Rust serde structs

### ✅ Implement image conversion service (JPG, PNG, WebP, AVIF) (5h)

Build conversion-service image module using pure-Rust image crate. Support JPG↔PNG↔WebP↔AVIF with compression quality and optional resize while maintaining aspect ratio.

**Acceptance Criteria**
- All four format conversions produce valid output with correct encoding
- Quality parameter controls compression level appropriately
- Resize option maintains aspect ratio when maintain_aspect is true
- No ffmpeg dependency for image conversion — pure Rust only

**Dependencies**
- Define shared TypeScript types and Rust serde structs

### ✅ Implement video conversion service (MP4, WebM, MKV) (6h)

Build conversion-service video module using ffmpeg-next with codec selection (libx264, libvpx). Support MP4↔WebM↔MKV with memory-mapped I/O for large files.

**Acceptance Criteria**
- MP4↔WebM↔MKV conversions produce playable output files
- Memory-mapped I/O used for files over configurable threshold
- Per-file progress percentage reported during conversion
- Hardware acceleration auto-detected on macOS (VideoToolbox)

**Dependencies**
- Implement audio conversion service (MP3, WAV, FLAC, M4A)

### ✅ Build Convert tab UI with format selector and quality controls (4h)

Implement ConvertTab with format dropdown, quality slider (0-100), collapsible advanced options (codec, bitrate, resize), and output directory selector.

**Acceptance Criteria**
- Format dropdown shows only valid target formats for selected files
- Quality slider updates ConvertOptions and shows current value
- Advanced options collapse/expand with chevron animation
- Apply triggers convert_files IPC with correct options

**Dependencies**
- Implement global state management with useReducer
- Implement audio conversion service (MP3, WAV, FLAC, M4A)

### ✅ Wire conversion into processing pipeline with parallel execution (5h)

Integrate conversion-service into processing-pipeline with Rayon parallelism, backup creation, per-file progress events, job cancellation via AtomicBool, and history recording.

**Acceptance Criteria**
- Batch conversion processes files in parallel across Rayon thread pool
- Job cancellation stops remaining files within one iteration
- Progress events emitted per-file with correct percentage
- Job recorded in SQLite with per-file success/failure status

**Dependencies**
- Implement video conversion service (MP4, WebM, MKV)
- Implement image conversion service (JPG, PNG, WebP, AVIF)
- Implement apply_rename with backup and progress events

## Metadata Editing & Job History
**Goal:** Enable ID3 tag and EXIF metadata reading, editing, and bulk stripping. Implement searchable job history with detail views.

### User Stories
_None_

### Acceptance Criteria
_None_

### ✅ Implement metadata-service for ID3 and EXIF (5h)

Build metadata-service using id3 crate for audio ID3v2 tags and kamadak-exif for image EXIF data. Support read, write, and bulk strip operations with editable field identification.

**Acceptance Criteria**
- ID3v2 tags read correctly from MP3 files with all standard fields
- EXIF data read from JPG/PNG with field names and values
- strip_exif rewrites image without metadata, verifiable by re-read
- get_editable_fields returns correct field descriptors per file type

**Dependencies**
- Define shared TypeScript types and Rust serde structs

### ✅ Build Metadata tab UI with tag editor and bulk strip (5h)

Implement MetadataTab with per-field tag editor inputs, read-only field indicators, and one-click bulk strip buttons for EXIF and ID3. Display metadata on file selection.

**Acceptance Criteria**
- Selecting files triggers read_metadata IPC and displays tags
- Editable fields show input controls, read-only fields show disabled state
- Bulk strip button triggers write_metadata with strip flags
- Changes applied via Apply button with backup and history recording

**Dependencies**
- Implement metadata-service for ID3 and EXIF
- Implement global state management with useReducer

### ✅ Build job history view with FTS5 search (5h)

Implement history panel accessible from footer dropdown showing paginated job list with search. Clicking a job shows detail view with per-file results. Search uses FTS5.

**Acceptance Criteria**
- History dropdown shows recent jobs with type icon, timestamp, file count, status
- Search input filters jobs via FTS5 matching descriptions and file names
- Job detail view lists all files with original/transformed names and status
- Pagination loads more jobs on scroll with correct offset

**Dependencies**
- Build action footer with Apply, progress bar, and history dropdown

### ✅ Implement settings persistence and settings modal (4h)

Build settings modal with theme toggle, accent color picker, default output dir, max parallel jobs, auto-backup toggle, backup retention days. Persist via get_settings/update_settings IPC.

**Acceptance Criteria**
- Settings modal opens from navbar gear icon
- All settings load from SQLite on app start and save on change
- Theme and accent color changes apply immediately without reload
- Backup retention days setting used by cleanup_backups

**Dependencies**
- Build UI shell with navbar, layout, and theme switching

## Polish, Testing & Cross-Platform Packaging
**Goal:** Finalize animations, accessibility, performance benchmarks, test suite, and cross-platform CI/CD builds with auto-updater.

### User Stories
_None_

### Acceptance Criteria
_None_

### ✅ Implement micro-interactions and animations (4h)

Add all TRD-specified animations: 200ms ease-out transitions, staggered file card processing animations (50ms offset), button hover scale+glow, Apply progress fill, error shake, success confetti.

**Acceptance Criteria**
- All transitions use 200ms ease-out with will-change GPU hints
- File cards animate in stagger during batch processing
- Error state triggers red accent and shake animation on file cards
- Success confetti particle effect fires on job completion

**Dependencies**
- Build file list with virtualized rendering
- Build action footer with Apply, progress bar, and history dropdown

### ✅ Accessibility audit and keyboard navigation (4h)

Ensure all interactive elements are keyboard-navigable (Tab, Enter, Space), add ARIA labels, implement high contrast mode support, add screen reader labels, verify focus ring visibility.

**Acceptance Criteria**
- Full app navigable via Tab with visible 2px accent focus rings
- All buttons and inputs have descriptive ARIA labels
- High contrast mode renders correctly with sufficient contrast ratios
- Screen reader announces file status changes and job progress

**Dependencies**
- Implement micro-interactions and animations

### ✅ Write unit and integration test suite (6h)

Unit tests for preview-service, file-service, history-service against in-memory SQLite, frontend reducer, and IPC error parsing. Integration tests for full rename and convert flows with temp directories.

**Acceptance Criteria**
- Preview-service tests cover regex, templates, numbering, conflicts, and performance benchmark
- History-service tests cover CRUD, FTS5 search, and pagination against in-memory SQLite
- Integration test: rename 10 files with backup, undo restores originals byte-for-byte
- Frontend reducer tests cover all action types with correct state transitions

**Dependencies**
- Implement undo_job with backup restoration
- Wire conversion into processing pipeline with parallel execution

### ✅ Performance benchmarking and optimization (4h)

Benchmark preview_rename under 100ms for 500 files, app startup under 2s, IPC round-trip for 500 file payloads under 100ms. Optimize hot paths. Enable LTO in release builds.

**Acceptance Criteria**
- preview_rename for 500 files completes under 100ms in release build
- App launches to interactive webview in under 2 seconds
- 500-file IPC payload round-trip under 100ms measured end-to-end
- Release build with LTO and symbol stripping under 10MB on macOS

**Dependencies**
- Write unit and integration test suite

### ✅ Cross-platform CI/CD and auto-updater setup (5h)

Configure GitHub Actions matrix builds for macOS, Windows, Linux. Set up Tauri auto-updater with GitHub Releases manifest. Code signing for macOS (notarization) and Windows.

**Acceptance Criteria**
- GitHub Actions builds and tests on macOS, Windows, and Ubuntu
- Tauri bundler produces DMG (macOS), MSI (Windows), AppImage (Linux)
- Auto-updater checks GitHub Releases and prompts user on new version
- macOS build passes notarization with valid Developer ID certificate

**Dependencies**
- Performance benchmarking and optimization

## ❓ Open Questions
- Should ffmpeg be statically linked (larger bundle, zero setup) or dynamically loaded with runtime detection? Static is assumed but may exceed 10MB target.
- What is the default backup retention period — 7 days, 30 days, or unlimited? This affects disk usage for users processing large video files.
- Should recursive directory processing be supported in MVP or deferred to a later phase?
- Should the 5000 file hard cap be a hard rejection or a soft warning that power users can override?
- Is a portable/no-install version needed for corporate environments where users cannot install apps?