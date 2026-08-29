mod cli;
mod cleaner;
mod namer;
mod probe;

use anyhow::Result;
use clap::Parser;
use cli::Cli;
use cleaner::{clean_video, CleanOptions, CleanReport};
use console::{style, Emoji};
use namer::get_output_path;
use probe::probe_file;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use walkdir::WalkDir;

static CHERRY_BLOSSOM: Emoji<'_, '_> = Emoji("🌸 ", "");
static CLEAN_ICON: Emoji<'_, '_> = Emoji("✨ ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("⛩️  ", "");

fn check_dependencies() -> Result<()> {
    if Command::new("ffprobe").arg("-version").output().is_err() {
        anyhow::bail!("'ffprobe' was not found in PATH. Please install ffmpeg.");
    }
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        anyhow::bail!("'ffmpeg' was not found in PATH. Please install ffmpeg.");
    }
    Ok(())
}

fn is_video_file(path: &Path) -> bool {
    // Ignore hidden files and temporary files
    if let Some(file_name) = path.file_name() {
        let name = file_name.to_string_lossy();
        if name.starts_with('.') {
            return false;
        }
    }

    if let Some(ext) = path.extension() {
        let e = ext.to_string_lossy().to_lowercase();
        matches!(e.as_str(), "mkv" | "mp4" | "webm" | "avi" | "m4v" | "mov")
    } else {
        false
    }
}

fn collect_video_files(input_path: &Path) -> Vec<PathBuf> {
    if input_path.is_file() {
        if is_video_file(input_path) {
            vec![input_path.to_path_buf()]
        } else {
            vec![]
        }
    } else if input_path.is_dir() {
        let mut files: Vec<PathBuf> = WalkDir::new(input_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file() && is_video_file(e.path()))
            .map(|e| e.into_path())
            .collect();
        files.sort();
        files
    } else {
        vec![]
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

fn print_report(idx: usize, total: usize, report: &CleanReport, quiet: bool) {
    if quiet {
        return;
    }

    let original_name = report
        .input_path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    let target_name = report
        .output_path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();

    let prefix = format!("[{}/{}]", idx, total);
    println!(
        "\n{} {}",
        style(prefix).magenta().bold(),
        style(format!("▶ {}", original_name)).bold().cyan()
    );

    if report.skipped {
        println!("       {} Already clean & immersion-ready (Skipped)", style("✔").green().bold());
        return;
    }

    if original_name != target_name {
        println!(
            "       {} {}",
            style("↳ Clean name:").dim(),
            style(&target_name).green().bold()
        );
    }

    if report.renamed_only {
        println!("       {} Streams already clean — renamed to clean filename", style("✔").green());
        return;
    }

    println!(
        "       {} Video: [{}] | Audio: [JP {}]",
        style("•").magenta(),
        style(&report.video_codec).yellow(),
        style(&report.audio_codec).yellow()
    );

    let mut actions = Vec::new();
    if report.foreign_audio_dropped > 0 {
        actions.push(format!("Dropped {} foreign audio dub(s)", report.foreign_audio_dropped));
    }
    if report.foreign_subs_dropped > 0 {
        actions.push(format!("Stripped {} foreign subtitle(s)", report.foreign_subs_dropped));
    }
    if report.jp_subs_kept > 0 {
        actions.push(format!("Preserved {} Japanese sub(s)", report.jp_subs_kept));
    } else {
        actions.push("0 sub tracks (Raw video)".to_string());
    }
    if report.attachments_dropped > 0 {
        actions.push(format!("Purged {} font attachment(s)", report.attachments_dropped));
    }

    for action in actions {
        println!("       {} {}", style("✔").green(), style(action).dim());
    }

    if !report.dry_run {
        let size_diff = if report.original_size > report.new_size {
            format!(
                " (Saved {})",
                style(format_bytes(report.original_size - report.new_size)).green().bold()
            )
        } else {
            "".to_string()
        };

        println!(
            "       {} Size: {} ➔ {}{} | Elapsed: {:.2}s",
            style("•").magenta(),
            format_bytes(report.original_size),
            style(format_bytes(report.new_size)).bold(),
            size_diff,
            report.duration_secs
        );
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    check_dependencies()?;

    println!(
        "{}{}",
        SPARKLE,
        style("浄化 (jpsan) — Anime Immersion Sanitizer").bold().magenta()
    );
    if cli.dry_run {
        println!(
            "{}",
            style("⚠ Running in DRY-RUN mode. No files will be modified.").yellow().bold()
        );
    }

    let files = collect_video_files(&cli.path);
    if files.is_empty() {
        println!(
            "{} No video files found at {}",
            style("!").red().bold(),
            cli.path.display()
        );
        return Ok(());
    }

    println!(
        "{} Found {} video file(s) to process",
        CHERRY_BLOSSOM,
        style(files.len()).cyan().bold()
    );

    let clean_options = CleanOptions {
        keep_jp_subs: !cli.strip_all_subs,
        strip_all_subs: cli.strip_all_subs,
        keep_all_jp_audio: cli.keep_all_jp_audio,
        strip_chapters: cli.strip_chapters,
        in_place: cli.in_place,
        dry_run: cli.dry_run,
    };

    let overall_start = Instant::now();
    let mut total_original_bytes = 0u64;
    let mut total_new_bytes = 0u64;
    let mut cleaned_files = 0;
    let mut skipped_files = 0;
    let mut failed_files = 0;
    let total_count = files.len();

    for (idx, file) in files.iter().enumerate() {
        match probe_file(file) {
            Ok(analysis) => {
                let target_out = get_output_path(
                    file,
                    cli.output.as_deref(),
                    cli.in_place,
                    !cli.no_sanitize,
                );

                match clean_video(file, &target_out, &analysis, &clean_options) {
                    Ok(report) => {
                        if report.skipped {
                            skipped_files += 1;
                        } else {
                            total_original_bytes += report.original_size;
                            total_new_bytes += report.new_size;
                            cleaned_files += 1;
                        }
                        print_report(idx + 1, total_count, &report, cli.quiet);
                    }
                    Err(e) => {
                        failed_files += 1;
                        eprintln!(
                            "\n[{}/{}] {} Failed to clean {}: {:#}",
                            idx + 1,
                            total_count,
                            style("✖").red().bold(),
                            file.display(),
                            e
                        );
                    }
                }
            }
            Err(e) => {
                failed_files += 1;
                eprintln!(
                    "\n[{}/{}] {} Failed to probe {}: {:#}",
                    idx + 1,
                    total_count,
                    style("✖").red().bold(),
                    file.display(),
                    e
                );
            }
        }
    }

    let overall_time = overall_start.elapsed().as_secs_f64();

    println!("\n{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());
    println!(
        "{} {}",
        CLEAN_ICON,
        style("Immersion Sanitization Complete!").bold().green()
    );
    println!(
        "  Cleaned/Renamed: {} | Skipped (Already clean): {} | Failed: {} | Time: {:.2}s",
        style(cleaned_files).green().bold(),
        style(skipped_files).cyan().bold(),
        if failed_files > 0 {
            style(failed_files).red().bold()
        } else {
            style(failed_files).dim()
        },
        overall_time
    );

    if !cli.dry_run && total_original_bytes > total_new_bytes {
        let saved = total_original_bytes - total_new_bytes;
        println!(
            "  Total Space Saved: {} ({} ➔ {})",
            style(format_bytes(saved)).green().bold(),
            format_bytes(total_original_bytes),
            format_bytes(total_new_bytes)
        );
    }
    println!("{}", style("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━").dim());

    Ok(())
}
