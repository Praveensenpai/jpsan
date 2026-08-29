use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "jpsan",
    author = "Praveensenpai",
    version = "0.1.0",
    about = "浄化 (jpsan) — Blazing-fast lossless anime video sanitizer for Japanese immersion"
)]
pub struct Cli {
    /// Path to input video file or directory
    #[arg(required = true)]
    pub path: PathBuf,

    /// Destination directory for cleaned video files
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Replace original files in-place after verifying clean output
    #[arg(short, long)]
    pub in_place: bool,

    /// Strip ALL subtitles completely (pure audio-visual listening immersion)
    #[arg(long)]
    pub strip_all_subs: bool,

    /// Keep all Japanese audio tracks if multiple exist (e.g. Stereo 2.0 + 5.1 Surround)
    #[arg(long)]
    pub keep_all_jp_audio: bool,

    /// Strip chapter markers (default: preserve chapters for OP/ED skipping)
    #[arg(long)]
    pub strip_chapters: bool,

    /// Disable automatic filename sanitization (keep original release names)
    #[arg(long)]
    pub no_sanitize: bool,

    /// Dry run mode (preview stream mappings and actions without writing files)
    #[arg(short, long)]
    pub dry_run: bool,

    /// Quiet mode (minimal output)
    #[arg(short, long)]
    pub quiet: bool,
}
