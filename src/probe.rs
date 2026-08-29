use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, Clone)]
pub struct ProbeOutput {
    pub streams: Vec<StreamInfo>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamInfo {
    pub index: usize,
    pub codec_type: Option<String>,
    pub codec_name: Option<String>,
    #[serde(default)]
    pub tags: Option<HashMap<String, String>>,
    #[serde(default)]
    pub disposition: Option<HashMap<String, i32>>,
}

#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub video_stream: Option<StreamInfo>,
    pub jp_audio_streams: Vec<StreamInfo>,
    pub foreign_audio_streams: Vec<StreamInfo>,
    pub jp_subtitle_streams: Vec<StreamInfo>,
    pub foreign_subtitle_streams: Vec<StreamInfo>,
    pub attachment_streams: Vec<StreamInfo>,
    pub original_file_size: u64,
}

impl StreamInfo {
    pub fn get_language(&self) -> Option<String> {
        self.tags
            .as_ref()?
            .get("language")
            .map(|l| l.to_lowercase())
    }

    pub fn get_title(&self) -> Option<String> {
        self.tags.as_ref()?.get("title").map(|t| t.to_string())
    }

    pub fn is_japanese(&self) -> bool {
        if let Some(lang) = self.get_language() {
            let l = lang.trim();
            if l == "jpn" || l == "ja" || l == "jp" || l == "japanese" {
                return true;
            }
        }
        if let Some(title) = self.get_title() {
            let t = title.to_lowercase();
            if t.contains("japanese") || t.contains("jpn") || t.contains("nihongo") {
                return true;
            }
        }
        false
    }

    pub fn is_english_or_foreign(&self) -> bool {
        if let Some(lang) = self.get_language() {
            let l = lang.trim();
            if l != "jpn" && l != "ja" && l != "jp" && l != "japanese" && l != "und" && !l.is_empty() {
                return true;
            }
        }
        if let Some(title) = self.get_title() {
            let t = title.to_lowercase();
            if t.contains("english") || t.contains("eng") || t.contains("dub") || t.contains("signs") {
                return true;
            }
        }
        false
    }
}

pub fn probe_file(path: &Path) -> Result<AnalysisResult> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            "-show_chapters",
        ])
        .arg(path)
        .output()
        .with_context(|| format!("Failed to execute ffprobe on {}", path.display()))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe failed on {}: {}", path.display(), err);
    }

    let probe: ProbeOutput = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("Failed to parse ffprobe JSON output for {}", path.display()))?;

    let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let mut video_stream = None;
    let mut all_audio = Vec::new();
    let mut all_subtitles = Vec::new();
    let mut attachment_streams = Vec::new();

    for stream in probe.streams {
        let codec_type = stream.codec_type.as_deref().unwrap_or("");
        match codec_type {
            "video" => {
                let is_attached_pic = stream
                    .disposition
                    .as_ref()
                    .and_then(|d| d.get("attached_pic"))
                    .copied()
                    .unwrap_or(0)
                    == 1;
                if is_attached_pic {
                    attachment_streams.push(stream);
                } else if video_stream.is_none() {
                    video_stream = Some(stream);
                }
            }
            "audio" => all_audio.push(stream),
            "subtitle" => all_subtitles.push(stream),
            "attachment" => attachment_streams.push(stream),
            _ => {}
        }
    }

    // Audio stream classification
    let mut jp_audio_streams = Vec::new();
    let mut foreign_audio_streams = Vec::new();

    if all_audio.len() == 1 {
        // Single audio track in the video: even if untagged ('und'), it's the main audio track
        jp_audio_streams.push(all_audio[0].clone());
    } else {
        for audio in all_audio {
            if audio.is_japanese() {
                jp_audio_streams.push(audio);
            } else if audio.is_english_or_foreign() {
                foreign_audio_streams.push(audio);
            } else {
                // If untagged and no JP audio found yet, consider as fallback candidate
                if jp_audio_streams.is_empty() {
                    jp_audio_streams.push(audio);
                } else {
                    foreign_audio_streams.push(audio);
                }
            }
        }
    }

    // Subtitle stream classification
    let mut jp_subtitle_streams = Vec::new();
    let mut foreign_subtitle_streams = Vec::new();

    for sub in all_subtitles {
        if sub.is_japanese() {
            jp_subtitle_streams.push(sub);
        } else {
            foreign_subtitle_streams.push(sub);
        }
    }

    Ok(AnalysisResult {
        video_stream,
        jp_audio_streams,
        foreign_audio_streams,
        jp_subtitle_streams,
        foreign_subtitle_streams,
        attachment_streams,
        original_file_size: file_size,
    })
}
