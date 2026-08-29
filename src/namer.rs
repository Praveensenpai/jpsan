use regex::Regex;
use std::path::{Path, PathBuf};

/// Sanitizes typical anime release filenames.
/// Example: `[SubsPlease] Saijo no Osewa - 01 (1080p) [79559860].mkv` -> `Saijo no Osewa - 01.mkv`
pub fn sanitize_filename(filename: &str) -> String {
    let path = Path::new(filename);
    let stem = match path.file_stem() {
        Some(s) => s.to_string_lossy().to_string(),
        None => return filename.to_string(),
    };
    let extension = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    // 1. Remove release group prefix: e.g. [SubsPlease], [Erai-raws], [Judas], etc.
    let re_group = Regex::new(r"^\[[^\]]+\]\s*").unwrap();
    let mut cleaned = re_group.replace(&stem, "").to_string();

    // 2. Remove brackets/parentheses that contain metadata (resolution, codec, subtitles, hashes)
    // Matches (1080p HEVC x265 10bit AAC), [1080p], [Multiple Subtitle], [Dual Audio], etc.
    let re_meta_bracket = Regex::new(r"(?i)\s*[\(\[]\s*[^)\]]*(?:1080p|720p|2160p|4k|hevc|x264|x265|avc|10bit|8bit|dual-audio|dual audio|multi-sub|multiple subtitle|subtitle|web-dl|bdrip|bluray|aac|flac|crc|[0-9a-f]{8})[^)\]]*[\)\]]").unwrap();
    cleaned = re_meta_bracket.replace_all(&cleaned, "").to_string();

    // 3. Remove any trailing CRC32 / hash brackets: e.g. [79559860], [F6EDB700]
    let re_hash = Regex::new(r"\s*\[[0-9A-Fa-f]{6,8}\]\s*$").unwrap();
    cleaned = re_hash.replace(&cleaned, "").to_string();

    // 4. Remove leftover empty brackets/parentheses and normalize spaces
    let re_empty = Regex::new(r"[\(\[]\s*[\)\]]").unwrap();
    cleaned = re_empty.replace_all(&cleaned, "").to_string();

    let re_spaces = Regex::new(r"\s{2,}").unwrap();
    cleaned = re_spaces.replace_all(&cleaned, " ").to_string();

    let final_stem = cleaned.trim().trim_matches('-').trim();

    if final_stem.is_empty() {
        filename.to_string()
    } else {
        format!("{}{}", final_stem, extension)
    }
}

/// Generates the target output path for a cleaned file.
pub fn get_output_path(
    input: &Path,
    output_dir: Option<&Path>,
    in_place: bool,
    sanitize_name: bool,
) -> PathBuf {
    let original_name = input
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "output.mkv".to_string());

    let target_name = if sanitize_name {
        sanitize_filename(&original_name)
    } else {
        original_name
    };

    if in_place {
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(target_name)
    } else if let Some(out_dir) = output_dir {
        out_dir.join(target_name)
    } else {
        // Default to a 'cleaned' subdirectory beside the input file
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join("cleaned").join(target_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_subsplease() {
        let input = "[SubsPlease] Saijo no Osewa - 01 (1080p) [79559860].mkv";
        assert_eq!(sanitize_filename(input), "Saijo no Osewa - 01.mkv");
    }

    #[test]
    fn test_sanitize_erai() {
        let input = "[Erai-raws] Jujutsu Kaisen - 05 [1080p][Multiple Subtitle].mkv";
        assert_eq!(sanitize_filename(input), "Jujutsu Kaisen - 05.mkv");
    }

    #[test]
    fn test_sanitize_already_clean() {
        let input = "Ore wo Suki nano wa Omae dake ka yo - 01.mkv";
        assert_eq!(sanitize_filename(input), "Ore wo Suki nano wa Omae dake ka yo - 01.mkv");
    }

    #[test]
    fn test_sanitize_judas_hevc() {
        let input = "[Judas] Kimetsu no Yaiba - S02E01 (1080p HEVC x265 10bit AAC) [A1B2C3D4].mkv";
        assert_eq!(sanitize_filename(input), "Kimetsu no Yaiba - S02E01.mkv");
    }

    #[test]
    fn test_sanitize_asw() {
        let input = "[ASW] Oshi no Ko - 01 [1080p HEVC AAC] [12345678].mkv";
        assert_eq!(sanitize_filename(input), "Oshi no Ko - 01.mkv");
    }
}
