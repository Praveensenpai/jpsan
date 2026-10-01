# CODEBASE.md: jpsan Semantic Digest

> **Notice**: This file is an AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow
```text
main::main() ──> cli::Cli (Clap parser)
      │
      ├──> probe::probe_file() ──> ffprobe (JSON stream analysis)
      ├──> namer::get_output_path() ──> namer::sanitize_filename() (Regex cleanup)
      └──> cleaner::clean_video() ──> std::fs::rename | ffmpeg (-c copy lossless remux)
```

## 2. Global Constraints & Architecture Patterns
- **Primary Language & Edition**: Rust 2021 edition.
- **Architectural Paradigm**: Modular CLI utility (`cli/`, `probe/`, `namer/`, `cleaner/`).
- **Hard Constraints**: <400 lines/file, <60 lines/fn, max 4 params, zero production unwrap() outside test/static regexes, zero compiler/clippy warnings.
- **Target Distribution**: Linux x86_64 standalone binary via GitHub Releases (`x86_64-unknown-linux-gnu`).

## 3. Module & Interface Skeleton

### `src/cli.rs` (Role: cli, Lines: 47)
- **Responsibility**: Clap CLI argument parser and parameter definitions.
- **Imports**: `clap::Parser`, `std::path::PathBuf`
- **Types & Enums**:
  ```rust
  pub struct Cli {
      pub path: PathBuf,
      pub output: Option<PathBuf>,
      pub in_place: bool,
      pub strip_all_subs: bool,
      pub keep_all_jp_audio: bool,
      pub strip_chapters: bool,
      pub no_sanitize: bool,
      pub dry_run: bool,
      pub quiet: bool,
  }
  ```
- **Consumers**: `main.rs`
- **Side Effects / I/O**: None.

### `src/probe.rs` (Role: infra, Lines: 177)
- **Responsibility**: Subprocess wrapper for `ffprobe` to inspect streams, codecs, tags, dispositions, and chapters.
- **Imports**: `anyhow::{Context, Result}`, `serde::Deserialize`, `std::collections::HashMap`, `std::path::Path`, `std::process::Command`
- **Types & Enums**:
  ```rust
  pub struct ProbeOutput { pub streams: Vec<StreamInfo> }
  pub struct StreamInfo {
      pub index: usize,
      pub codec_type: Option<String>,
      pub codec_name: Option<String>,
      pub tags: Option<HashMap<String, String>>,
      pub disposition: Option<HashMap<String, i32>>,
  }
  pub struct AnalysisResult {
      pub video_stream: Option<StreamInfo>,
      pub jp_audio_streams: Vec<StreamInfo>,
      pub foreign_audio_streams: Vec<StreamInfo>,
      pub jp_subtitle_streams: Vec<StreamInfo>,
      pub foreign_subtitle_streams: Vec<StreamInfo>,
      pub attachment_streams: Vec<StreamInfo>,
      pub original_file_size: u64,
  }
  ```
- **Public Functions & Signatures**:
  ```rust
  pub fn probe_file(path: &Path) -> Result<AnalysisResult>
  impl StreamInfo {
      pub fn get_language(&self) -> Option<String>
      pub fn get_title(&self) -> Option<String>
      pub fn is_japanese(&self) -> bool
      pub fn is_english_or_foreign(&self) -> bool
  }
  ```
- **Consumers**: `main.rs`
- **Side Effects / I/O**: Executes `ffprobe` subprocess.

### `src/namer.rs` (Role: domain, Lines: 152)
- **Responsibility**: Sanitizes anime filenames by stripping release groups, metadata tags, languages, codecs, resolutions, and CRC32 hashes.
- **Imports**: `regex::Regex`, `std::path::{Path, PathBuf}`
- **Public Functions & Signatures**:
  ```rust
  pub fn sanitize_filename(filename: &str) -> String
  pub fn get_output_path(
      input: &Path,
      output_dir: Option<&Path>,
      in_place: bool,
      sanitize_name: bool,
  ) -> PathBuf
  ```
- **Consumers**: `main.rs`
- **Side Effects / I/O**: None.

### `src/cleaner.rs` (Role: domain/infra, Lines: 263)
- **Responsibility**: Lossless stream cleaning, audio/subtitle extraction, font stripping, and file atomic renaming/remuxing via `ffmpeg`.
- **Imports**: `crate::probe::AnalysisResult`, `anyhow::{Context, Result}`, `std::path::{Path, PathBuf}`, `std::process::Command`, `std::time::Instant`
- **Types & Enums**:
  ```rust
  pub struct CleanOptions {
      pub keep_jp_subs: bool,
      pub strip_all_subs: bool,
      pub keep_all_jp_audio: bool,
      pub strip_chapters: bool,
      pub in_place: bool,
      pub dry_run: bool,
  }
  pub struct CleanReport {
      pub input_path: PathBuf,
      pub output_path: PathBuf,
      pub original_size: u64,
      pub new_size: u64,
      pub duration_secs: f64,
      pub video_codec: String,
      pub audio_codec: String,
      pub foreign_audio_dropped: usize,
      pub foreign_subs_dropped: usize,
      pub jp_subs_kept: usize,
      pub attachments_dropped: usize,
      pub dry_run: bool,
      pub skipped: bool,
      pub renamed_only: bool,
  }
  ```
- **Public Functions & Signatures**:
  ```rust
  pub fn clean_video(
      input: &Path,
      target_output: &Path,
      analysis: &AnalysisResult,
      options: &CleanOptions,
  ) -> Result<CleanReport>
  ```
- **Consumers**: `main.rs`
- **Side Effects / I/O**: Filesystem rename/copy/remove, spawns `ffmpeg` subprocess for remuxing.

### `src/main.rs` (Role: api/entrypoint, Lines: 308)
- **Responsibility**: CLI lifecycle management, file discovery, dependency checks, progress reporting, and summary metrics.
- **Imports**: `clap::Parser`, `cli::Cli`, `cleaner::*`, `namer::*`, `probe::*`, `console::{style, Emoji}`, `std::io::Write`
- **Consumers**: OS execution entrypoint.
- **Side Effects / I/O**: Terminal logging to stdout/stderr, filesystem discovery via `walkdir`.

## 4. Execution Lifecycle Trace
1. **Startup**: Entrypoint (`main.rs`) parses CLI arguments via `cli::Cli::parse()`, checks `ffmpeg`/`ffprobe` availability in `PATH`.
2. **File Discovery**: Recursively finds matching video formats (`.mkv`, `.mp4`, `.webm`, `.avi`, `.m4v`, `.mov`) using `walkdir`.
3. **Probe & Analysis**: For each file, calls `probe::probe_file()` to classify video, Japanese audio, foreign audio, subtitle, and font attachment streams.
4. **Target Resolution**: Computes sanitized destination path via `namer::get_output_path()`.
5. **Stream Sanitization / Renaming**:
   - If streams are already clean and path is clean: skips.
   - If streams are clean but filename needs sanitizing: performs atomic rename.
   - If foreign streams or attachments exist: remuxes losslessly via `ffmpeg -c copy`.
6. **Report & Metrics**: Prints formatted item result with explicit stdout flushing, displays total time and bytes saved.

## 5. Verification Commands
```bash
# Test
cargo test --all-targets

# Lint & Format
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Build Release
cargo build --release --target x86_64-unknown-linux-gnu
```

## 6. Recent Iteration Changes
- **2026-10-01**: Enhanced `namer::sanitize_filename` regex to eliminate language and audio tags (`[Japanese]`, `[English]`, `[Eng Sub]`, `[Dual Audio]`, `[RAW]`), added stdout/stderr flushing after reports, created AI-first `CODEBASE.md`.
