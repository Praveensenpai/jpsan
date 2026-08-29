use crate::probe::AnalysisResult;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct CleanOptions {
    pub keep_jp_subs: bool,
    pub strip_all_subs: bool,
    pub keep_all_jp_audio: bool,
    pub strip_chapters: bool,
    pub in_place: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
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

pub fn clean_video(
    input: &Path,
    target_output: &Path,
    analysis: &AnalysisResult,
    options: &CleanOptions,
) -> Result<CleanReport> {
    let video_stream = analysis
        .video_stream
        .as_ref()
        .context("No video stream found in input file")?;

    if analysis.jp_audio_streams.is_empty() {
        anyhow::bail!("No Japanese (or fallback) audio stream found in {}", input.display());
    }

    let jp_subs_to_keep = if options.strip_all_subs || !options.keep_jp_subs {
        0
    } else {
        analysis.jp_subtitle_streams.len()
    };

    let video_codec = video_stream.codec_name.clone().unwrap_or_else(|| "unknown".to_string());
    let audio_codec = analysis.jp_audio_streams[0]
        .codec_name
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    let has_foreign_audio = !analysis.foreign_audio_streams.is_empty();
    let has_foreign_subs = !analysis.foreign_subtitle_streams.is_empty();
    let has_attachments = !analysis.attachment_streams.is_empty();
    let has_subs_to_strip = options.strip_all_subs && !analysis.jp_subtitle_streams.is_empty();

    let needs_stream_cleaning =
        has_foreign_audio || has_foreign_subs || has_attachments || has_subs_to_strip;
    let needs_renaming = input != target_output;

    // 1. If streams are already clean AND filename is already clean -> SKIP
    if !needs_stream_cleaning && !needs_renaming {
        return Ok(CleanReport {
            input_path: input.to_path_buf(),
            output_path: target_output.to_path_buf(),
            original_size: analysis.original_file_size,
            new_size: analysis.original_file_size,
            duration_secs: 0.0,
            video_codec,
            audio_codec,
            foreign_audio_dropped: 0,
            foreign_subs_dropped: 0,
            jp_subs_kept: jp_subs_to_keep,
            attachments_dropped: 0,
            dry_run: options.dry_run,
            skipped: true,
            renamed_only: false,
        });
    }

    // 2. If streams are already clean BUT filename needs sanitizing
    if !needs_stream_cleaning && needs_renaming {
        let start_time = Instant::now();
        if !options.dry_run {
            if let Some(parent) = target_output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            if options.in_place {
                std::fs::rename(input, target_output)
                    .or_else(|_| {
                        std::fs::copy(input, target_output)?;
                        std::fs::remove_file(input)
                    })
                    .with_context(|| format!("Failed to rename {} to {}", input.display(), target_output.display()))?;
            } else {
                std::fs::copy(input, target_output)
                    .with_context(|| format!("Failed to copy {} to {}", input.display(), target_output.display()))?;
            }
        }
        let duration_secs = start_time.elapsed().as_secs_f64();
        return Ok(CleanReport {
            input_path: input.to_path_buf(),
            output_path: target_output.to_path_buf(),
            original_size: analysis.original_file_size,
            new_size: analysis.original_file_size,
            duration_secs,
            video_codec,
            audio_codec,
            foreign_audio_dropped: 0,
            foreign_subs_dropped: 0,
            jp_subs_kept: jp_subs_to_keep,
            attachments_dropped: 0,
            dry_run: options.dry_run,
            skipped: false,
            renamed_only: true,
        });
    }

    // 3. Dry run for files needing stream cleaning
    if options.dry_run {
        return Ok(CleanReport {
            input_path: input.to_path_buf(),
            output_path: target_output.to_path_buf(),
            original_size: analysis.original_file_size,
            new_size: analysis.original_file_size,
            duration_secs: 0.0,
            video_codec,
            audio_codec,
            foreign_audio_dropped: analysis.foreign_audio_streams.len(),
            foreign_subs_dropped: analysis.foreign_subtitle_streams.len(),
            jp_subs_kept: jp_subs_to_keep,
            attachments_dropped: analysis.attachment_streams.len(),
            dry_run: true,
            skipped: false,
            renamed_only: false,
        });
    }

    // 4. Lossless stream cleaning via ffmpeg
    let start_time = Instant::now();

    if let Some(parent) = target_output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create destination directory {}", parent.display()))?;
    }

    let tmp_output = match target_output.parent() {
        Some(parent) => parent.join(format!(
            ".tmp_{}_{}",
            std::process::id(),
            target_output.file_name().unwrap().to_string_lossy()
        )),
        None => PathBuf::from(format!(".tmp_{}.mkv", std::process::id())),
    };

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y")
        .arg("-v")
        .arg("error")
        .arg("-nostats")
        .arg("-i")
        .arg(input);

    // Video mapping
    cmd.arg("-map").arg(format!("0:{}", video_stream.index));

    // Audio mapping
    if options.keep_all_jp_audio {
        for audio in &analysis.jp_audio_streams {
            cmd.arg("-map").arg(format!("0:{}", audio.index));
        }
    } else {
        cmd.arg("-map").arg(format!("0:{}", analysis.jp_audio_streams[0].index));
    }
    cmd.arg("-metadata:s:a:0").arg("language=jpn");
    cmd.arg("-disposition:a:0").arg("default");

    // Subtitle mapping
    if jp_subs_to_keep > 0 {
        for sub in &analysis.jp_subtitle_streams {
            cmd.arg("-map").arg(format!("0:{}", sub.index));
        }
        cmd.arg("-metadata:s:s:0").arg("language=jpn");
        cmd.arg("-disposition:s:0").arg("default");
    } else {
        cmd.arg("-sn");
    }

    // Drop attachments & metadata
    cmd.arg("-dn");
    cmd.arg("-map_metadata").arg("-1");

    // Chapters mapping
    if options.strip_chapters {
        cmd.arg("-map_chapters").arg("-1");
    } else {
        cmd.arg("-map_chapters").arg("0");
    }

    cmd.arg("-c").arg("copy");
    cmd.arg(&tmp_output);

    let output = cmd
        .output()
        .with_context(|| format!("Failed to execute ffmpeg for {}", input.display()))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let _ = std::fs::remove_file(&tmp_output);
        anyhow::bail!("ffmpeg error while processing {}: {}", input.display(), err);
    }

    let new_size = std::fs::metadata(&tmp_output)
        .map(|m| m.len())
        .unwrap_or(0);

    if new_size == 0 {
        let _ = std::fs::remove_file(&tmp_output);
        anyhow::bail!("ffmpeg generated an empty output file for {}", input.display());
    }

    // Move to final target output
    std::fs::rename(&tmp_output, target_output)
        .or_else(|_| {
            std::fs::copy(&tmp_output, target_output)?;
            std::fs::remove_file(&tmp_output)
        })
        .with_context(|| format!("Failed to write final file {}", target_output.display()))?;

    // If in-place and the name changed, remove the original messy input file
    if options.in_place && input != target_output {
        let _ = std::fs::remove_file(input);
    }

    let duration_secs = start_time.elapsed().as_secs_f64();

    Ok(CleanReport {
        input_path: input.to_path_buf(),
        output_path: target_output.to_path_buf(),
        original_size: analysis.original_file_size,
        new_size,
        duration_secs,
        video_codec,
        audio_codec,
        foreign_audio_dropped: analysis.foreign_audio_streams.len(),
        foreign_subs_dropped: analysis.foreign_subtitle_streams.len(),
        jp_subs_kept: jp_subs_to_keep,
        attachments_dropped: analysis.attachment_streams.len(),
        dry_run: false,
        skipped: false,
        renamed_only: false,
    })
}
