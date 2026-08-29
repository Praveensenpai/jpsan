# ⛩️ 浄化 (jpsan) — Anime Immersion Sanitizer

> **Blazing-fast lossless anime video cleaner tailored for Japanese immersion learning.**

`jpsan` is a lightweight, high-performance Rust CLI tool that strips away all useless distractions from anime video files (foreign subtitles, non-Japanese audio dubs, embedded English typesetting fonts, release group tags, and metadata spam), producing clean, pure video + Japanese audio files with **zero quality loss**.

---

## 🌸 Features

- **⚡ Instant & Lossless**: Uses `ffmpeg` stream copying (`-c copy`) with **no re-encoding**. Processes a full 1080p episode in **under 1.5 seconds**.
- **🎧 Japanese Audio Preservation**: Automatically identifies Japanese audio tracks (`jpn` / `ja`), sets them as the default audio track, and purges foreign dubs and commentary tracks.
- **👁️ Clean Immersion Viewing**: Strips distracting English / non-Japanese subtitles so you stay focused on listening comprehension.
- **✨ Yomitan & Anki Mining Ready**: Preserves embedded Japanese subtitle tracks (`jpn` / `ja`) for dictionary lookups in `mpv` with Yomitan or sentence mining with `kotonoha` / `mpvacious`.
- **📦 Embedded Font Purge**: Drops 20–40 embedded English typesetting fonts (Arial, Comic Sans, Impact, Roboto), saving **15–30 MB per episode** and eliminating `mpv` font-cache startup stutter.
- **⏭️ Chapter Preservation**: Keeps chapter markers intact so you can automatically skip Openings (OP) and Endings (ED).
- **🧹 Clean Filename Sanitizer**: Automatically cleans release clutter (`[SubsPlease] Title - 01 (1080p) [HASH].mkv` ➔ `Title - 01.mkv`).
- **🛡️ Smart Skip**: Skips already-clean files instantly with zero redundant disk I/O.

---

## 🚀 Installation

### One-line Install (Prebuilt Binary)
```bash
curl -fsSL https://raw.githubusercontent.com/Praveensenpai/jpsan/main/install.sh | bash
```

### Build from Source
```bash
git clone https://github.com/Praveensenpai/jpsan.git
cd jpsan
./install.sh
```

Or via Cargo:
```bash
cargo install --path .
```

*Requirements: `ffmpeg` and `ffprobe` in your system `PATH`.*

---

## 📖 Usage

### Clean a Single Episode
```bash
jpsan "[SubsPlease] Saijo no Osewa - 01 (1080p) [79559860].mkv"
```
*(Creates a clean `Saijo no Osewa - 01.mkv` in a `cleaned/` subfolder).*

### Clean an Entire Directory / Series
```bash
jpsan /path/to/Anime/ --output /path/to/Cleaned/
```

### In-Place Replace (Overwrites Original Files & Renames)
```bash
jpsan /path/to/Anime/ --in-place
```

### Dry Run (Preview Actions Without Making Changes)
```bash
jpsan /path/to/Anime/ --dry-run
```

### Pure Raw Listening (Strip ALL Subtitles)
```bash
jpsan /path/to/Anime/ --strip-all-subs
```

---

## ⚙️ CLI Options

```
Usage: jpsan [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to input video file or directory

Options:
  -o, --output <OUTPUT>    Destination directory for cleaned video files
  -i, --in-place           Replace original files in-place after verifying clean output
      --strip-all-subs     Strip ALL subtitles completely (pure audio-visual listening immersion)
      --keep-all-jp-audio  Keep all Japanese audio tracks if multiple exist (e.g. Stereo 2.0 + 5.1 Surround)
      --strip-chapters     Strip chapter markers (default: preserve chapters for OP/ED skipping)
      --no-sanitize        Disable automatic filename sanitization (keep original release names)
  -d, --dry-run            Dry run mode (preview stream mappings and actions without writing files)
  -q, --quiet              Quiet mode (minimal output)
  -h, --help               Print help
  -V, --version            Print version
```

---

## 📄 License
MIT License © [Praveen Senpai](https://github.com/Praveensenpai)
