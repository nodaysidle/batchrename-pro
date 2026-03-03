# BatchRename Pro

## 🎯 Product Vision
A cross-platform desktop app that unifies batch file renaming and format conversion into a single, privacy-first tool with a premium drag-and-drop interface, eliminating the need to juggle separate utilities for renaming, converting, and editing metadata.

## ❓ Problem Statement
Creators, podcasters, photographers, and video editors regularly need to rename large batches of files and convert between formats. Today this requires multiple disconnected tools, command-line scripts, or cloud services that compromise privacy and speed. No existing app handles both renaming and conversion elegantly in one local-first interface.

## 🎯 Goals
- Deliver a single app that handles batch rename, format conversion, and metadata editing for audio, image, and video files
- Provide a drag-and-drop interface with live rename previews so users see results before committing
- Process all files locally on-device for maximum privacy and speed using Rust parallel processing
- Ship cross-platform (macOS, Windows, Linux) from a single Tauri 2 codebase with an app bundle under 10MB
- Maintain full job history with undo/rollback so no file operation is irreversible
- Achieve MVP in one week: drag-drop, rename preview, MP3↔WAV conversion, apply with history log

## 🚫 Non-Goals
- Mobile support — this is a desktop-first application; mobile layouts are explicitly out of scope
- Cloud sync, remote storage, or any network-dependent features — the app is entirely local-first
- Streaming or real-time media playback — the app transforms files, it does not play them
- Extensibility via third-party plugins or a plugin marketplace
- Batch processing of documents (PDF, DOCX) beyond simple renaming — no document format conversion
- Server-side processing or SaaS backend infrastructure
- AI-powered rename suggestions or auto-tagging

## 👥 Target Users
- Podcasters who need to rename episode files with consistent naming patterns and convert between audio formats
- Photographers managing large shoots who need batch renaming with date/sequence templates and EXIF metadata editing
- Video editors working across formats (MP4, WebM, MKV) who need quick local conversion without quality loss
- Content creators managing mixed-media assets (audio, image, video) who want a single tool for renaming and converting
- Power users and developers who currently rely on command-line tools but want a visual interface with undo capability

## 🧩 Core Features
- Drag-and-Drop File Input: Glassmorphic drop zone accepting audio, image, video, and document files simultaneously with animated empty state and multi-file support
- Batch Rename with Live Preview: Regex patterns, template builder with quick-insert tokens ({date}, {number}, {original}), numbering sequences, and a real-time before/after preview list with toggle
- Audio Format Conversion: Local conversion between MP3, WAV, FLAC, and M4A using Rust ffmpeg bindings with quality slider and parallel processing
- Image Format Conversion: Local conversion between JPG, PNG, WebP, and AVIF with quality controls and batch processing
- Video Format Conversion: Local conversion between MP4, WebM, and MKV using ffmpeg with configurable output settings
- Metadata Editor: ID3 tag editing for audio files and EXIF data viewing/stripping for images, including a one-click bulk strip option
- Job History with Undo/Rollback: SQLite-backed history log of all operations with the ability to undo or rollback any completed job to restore original files
- Real-Time Progress Indicators: Per-file status cards (pending, processing, done, error) with smooth progress bar animations and staggered list updates
- Parallel Processing Engine: Rust-powered concurrent file processing leveraging multiple CPU cores for fast batch operations
- Transformation Panel: Collapsible right sidebar with tabbed interface (Rename, Convert, Metadata) for configuring operations before applying
- Action Footer: Sticky bottom bar with file-ready counter, prominent Apply button, history dropdown, and contextual Undo button

## ⚙️ Non-Functional Requirements
- App bundle size must be under 10MB
- All file processing must happen locally — no network calls for core functionality
- Cross-platform support for macOS, Windows, and Linux from a single codebase via Tauri 2
- Dark mode by default with optional light mode; dark palette uses #0F172A background, #1E293B cards, #3B82F6 or #A78BFA accents
- All UI transitions must complete within 200ms using ease-out timing
- Keyboard accessibility: all interactive elements navigable via Tab, Enter, and Space with visible 2px accent focus rings
- Screen reader support with ARIA labels on all interactive elements
- High contrast mode support for accessibility compliance
- Typography: Inter or system font for UI text, JetBrains Mono for file paths and code, with defined size scale (12–24px)
- SQLite database with FTS5 for fast search across job history
- Auto-updater built into the app bundle for seamless version updates
- Responsive layout for desktop and tablet; sidebar collapses to bottom panel on narrower screens

## 📊 Success Metrics
- MVP delivered within one week: drag-drop input, rename preview, MP3↔WAV conversion, apply button, and history log all functional
- Batch rename preview renders within 100ms for up to 500 files
- Audio conversion throughput at least 2x faster than sequential processing due to parallel Rust pipeline
- App cold start time under 2 seconds on mid-range hardware
- Zero data leaves the device — all processing verified as local-only
- Job history undo successfully restores original files with 100% fidelity
- App bundle size remains under 10MB across all three platforms

## 📌 Assumptions
- Users have ffmpeg-compatible codecs available or the app bundles necessary codec libraries within the size budget
- Tauri 2 provides stable cross-platform IPC between the SvelteKit frontend and Rust backend for production use
- SQLite with rusqlite is sufficient for job history storage without requiring a more complex database
- Users will primarily work with files numbering in the hundreds, not tens of thousands, for the initial release
- The target audience is comfortable with regex patterns or will rely on the template quick-buttons for renaming
- Video conversion can leverage ffmpeg Rust bindings without requiring a separate ffmpeg installation on the user machine
- Desktop screen sizes of 1280px width and above are the primary target; tablet is secondary

## ❓ Open Questions
- Should ffmpeg be bundled within the app or require users to install it separately — and how does bundling affect the 10MB size target?
- What is the maximum file count and total file size the app should handle before showing a warning or degrading gracefully?
- Should the undo/rollback feature keep backup copies of original files, or store transformation metadata to reverse operations?
- How should the app handle partial failures in a batch — stop all processing, skip failures, or queue retries?
- Is the auto-updater using Tauri's built-in updater, and what is the update channel strategy (stable, beta)?
- Should the app support custom user-defined rename templates that persist across sessions?
- What is the licensing model — free, freemium with conversion limits, or one-time purchase?