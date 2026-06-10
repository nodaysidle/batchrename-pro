# Architecture Requirements Document

## 🧱 System Overview
BatchRename Pro is a cross-platform desktop application built on Tauri 2 that unifies batch file renaming, format conversion, and metadata editing into a single local-first tool. The frontend is a Vite 6 + TypeScript + React SPA styled with Tailwind CSS 4, communicating with a Rust backend via Tauri IPC commands. All file processing (rename, convert, metadata edit) runs locally in Rust with parallel execution. Job history and undo/rollback are persisted in an embedded SQLite database with FTS5 search. The app ships as a single bundle under 10MB for macOS, Windows, and Linux with a built-in auto-updater.

## 🏗 Architecture Style
Monolithic desktop application with a two-process model: a webview renderer process (React UI) and a native host process (Rust backend) connected via Tauri IPC command bridge. No server, no network dependencies, no microservices.

## 🎨 Frontend Architecture
- **Framework:** Vite 6 with TypeScript and React 19, bundled as the Tauri webview frontend. Single-page application with no server-side rendering. Tailwind CSS 4 for all styling including dark/light theme tokens, glassmorphism utilities, and responsive layout.
- **State Management:** React built-in state: useState for local component state, useReducer for complex transformation panel state, useContext for global theme and app-level state (file list, job queue, history). No external state library. Tauri event listeners sync backend progress updates into React state via custom hooks (useTauriEvent).
- **Routing:** No client-side router. Single-view application with conditional rendering based on app state (empty drop zone vs file list vs processing view). Panel tabs (Rename, Convert, Metadata) managed via local component state, not URL routes.
- **Build Tooling:** Vite 6 as dev server and production bundler. TypeScript strict mode. Tailwind CSS 4 via PostCSS plugin. Tauri CLI for app packaging, code signing, and platform-specific builds. ESLint and Prettier for code quality.

## 🧠 Backend Architecture
- **Approach:** Rust backend exposed as Tauri 2 invoke commands. Each command is a standalone function annotated with #[tauri::command] that the frontend calls via invoke(). Commands handle file operations, format conversion, metadata editing, and history management. Long-running operations (batch convert) use Tauri event emitter to stream progress back to the frontend.
- **API Style:** Tauri IPC command-response pattern. Frontend calls invoke('command_name', { args }) which maps to a Rust #[tauri::command] function. Async commands return Result<T, String> serialized as JSON. Long-running operations use Tauri app_handle.emit() to push progress events that the frontend subscribes to via listen().
- **Services:**
- FileService: File system operations including batch rename, copy, move, and delete with atomic rollback support via backup-before-write strategy
- ConversionService: Audio (MP3, WAV, FLAC, M4A), image (JPG, PNG, WebP, AVIF), and video (MP4, WebM, MKV) format conversion using ffmpeg Rust bindings (ffmpeg-next crate) with configurable quality settings
- MetadataService: ID3 tag reading and writing for audio files (id3 crate), EXIF reading and stripping for images (kamadak-exif or rexif crate), bulk metadata operations
- HistoryService: SQLite-backed job history with full operation logging, undo/rollback by restoring backup files, FTS5 search across job descriptions and file names
- ProcessingPipeline: Parallel job executor using Rayon for CPU-bound tasks, manages a work queue with per-file status tracking and progress emission via Tauri events
- PreviewService: Rename preview generator that applies regex patterns and templates to file names in-memory without touching disk, returns before/after pairs for UI display

## 🗄 Data Layer
- **Primary Store:** SQLite via rusqlite crate, single database file stored in the Tauri app data directory (platform-specific). FTS5 virtual table for full-text search across job history entries.
- **Relationships:** Three core tables: jobs (id, timestamp, operation_type, status, file_count, description), job_files (id, job_id FK, original_path, original_name, transformed_name, transformed_path, backup_path, format_from, format_to, status, error_message), and settings (key-value store for user preferences like default output format, theme, last used patterns). job_files references jobs via foreign key. No ORM, direct SQL via rusqlite with prepared statements.
- **Migrations:** Schema versioning via a migrations table with version numbers. Migrations are embedded Rust strings applied sequentially on app startup. rusqlite::Connection::execute runs each migration in a transaction. Forward-only migrations for simplicity; no down migrations needed for a local desktop app.

## ☁️ Infrastructure
- **Hosting:** No hosting. Fully local desktop application distributed as platform-native installers: .dmg for macOS, .msi/.exe for Windows, .AppImage/.deb for Linux. Built and packaged via Tauri CLI (tauri build). Tauri built-in auto-updater checks a static JSON manifest hosted on GitHub Releases or a simple static file server for version updates.
- **Scaling Strategy:** Vertical scaling only via Rayon thread pool for parallel file processing. Thread pool size defaults to available CPU cores. No horizontal scaling, no server infrastructure. File count limits enforced in UI (warning at 1000+ files, hard cap configurable). Memory-mapped file I/O for large video files to avoid excessive RAM usage.
- **CI/CD:** GitHub Actions workflow: lint (clippy + eslint), test (cargo test + vitest), build (tauri build for all three platforms via matrix strategy), release (upload artifacts to GitHub Releases with auto-updater manifest). Single branch trunk-based development with release tags.

## ⚖️ Key Trade-offs
- Bundling ffmpeg libraries increases app size but eliminates user installation dependency; mitigated by including only required codecs and stripping debug symbols to stay under 10MB
- SQLite over a more capable database limits concurrent write throughput but is perfectly adequate for single-user desktop job history and eliminates external dependencies
- No client-side router simplifies the architecture for a single-view app but means deep-linking or multi-window workflows would require refactoring
- Backup-before-write undo strategy uses more disk space temporarily but guarantees 100% fidelity rollback without complex reverse-transform logic
- React over Svelte (per stack preset) adds slightly more JS bundle weight but provides a larger ecosystem and TypeScript integration maturity
- Rayon for parallelism is simple and effective for CPU-bound file processing but does not support async I/O; heavy I/O operations use tokio via Tauri async runtime instead
- No external state management library keeps dependencies minimal but means complex state flows must be carefully structured with useReducer and context to avoid prop drilling

## 📐 Non-Functional Requirements
- App cold start time under 2 seconds on mid-range hardware (Intel i5 / Apple M1 equivalent)
- All UI transitions complete within 200ms using ease-out timing curves
- Batch rename preview renders within 100ms for up to 500 files
- App bundle size under 10MB across all three platforms after compression
- Zero network calls for core functionality; all processing is local-only
- Full keyboard accessibility: Tab, Enter, Space navigation with visible 2px accent focus rings on all interactive elements
- Screen reader support with ARIA labels on all interactive elements
- Dark mode default with light mode toggle; theme tokens defined in Tailwind CSS 4 config
- SQLite database operations are non-blocking from the UI thread; all DB access runs in Rust async commands
- Undo/rollback restores original files with 100% byte-level fidelity via backup file strategy
- Cross-platform consistent behavior on macOS 12+, Windows 10+, and Ubuntu 22.04+