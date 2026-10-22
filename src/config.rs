//! TOML configuration: sources, what counts as media, player, YouTube,
//! assistant and interface settings.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

// `deny_unknown_fields` on every table, not just for tidiness. In TOML a bare
// key belongs to the most recent `[table]` / `[[array]]` header above it, so a
// `skip_dirs = [...]` written *below* `[[roots]]` silently becomes a field of
// that root. With serde's default (ignore unknown fields) the top-level
// `skip_dirs` then falls back to empty and nothing reports a problem. Denying
// unknown fields turns that into a parse error pointing at the line.
//
// Field order matters for saving: TOML needs plain values before tables, so
// the tables and `roots` come last.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Relative paths are resolved against the config file's directory.
    #[serde(default = "default_db_path")]
    pub db_path: String,
    /// Directory pages fetched in parallel.
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
    /// Pause between depth levels, to be polite to the server.
    #[serde(default = "default_delay_ms")]
    pub delay_ms: u64,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// Guards against symlink loops, which produce ever-longer *distinct*
    /// URLs and so are never caught by the visited set.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
    #[serde(default = "default_media_extensions")]
    pub media_extensions: Vec<String>,
    /// Case-insensitive substring match against directory names.
    #[serde(default)]
    pub skip_dirs: Vec<String>,
    #[serde(default)]
    pub ui: Ui,
    #[serde(default)]
    pub player: Player,
    #[serde(default)]
    pub youtube: YouTube,
    #[serde(default)]
    pub assistant: Assistant,
    #[serde(default)]
    pub roots: Vec<Root>,

    /// Directory the config was loaded from; not part of the file.
    #[serde(skip)]
    pub base_dir: PathBuf,
}

/// A place to index. Exactly one of the location fields is set:
/// `url` (a server's folder listing), `path` (a folder on this PC),
/// `youtube` (a channel or playlist link) or `feed` (a podcast / RSS feed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub youtube: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feed: Option<String>,
    /// What the source holds; decides how files are grouped into titles.
    #[serde(default)]
    pub kind: SourceKind,
    /// Speaker for everything in this source ("Mizanur Rahman Azhari").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<String>,
    /// The first folder level under the source is the speaker's name
    /// ("Waz/Speaker Name/lecture.mp3").
    #[serde(default, skip_serializing_if = "is_false")]
    pub speaker_folders: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Lectures: single talks (a folder per talk, or loose files).
/// Series: folders of numbered parts or episodes (tafsir, seerah...).
/// Quran: reciter folders holding surah files (001.mp3 ... 114.mp3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    #[default]
    Lectures,
    Series,
    Quran,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Lectures => "lectures",
            SourceKind::Series => "series",
            SourceKind::Quran => "quran",
        }
    }
}

pub enum RootKind {
    Http(Url),
    Local(PathBuf),
    YouTube(String),
    Feed(Url),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ui {
    /// "bn" (Bangla) or "en" (English).
    #[serde(default = "default_language")]
    pub language: String,
}

impl Default for Ui {
    fn default() -> Self {
        Ui {
            language: default_language(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub command: String,
    /// `{url}` in any argument is replaced with the media URL or file path.
    #[serde(default = "default_player_args")]
    pub args: Vec<String>,
    /// Send everything to the external player instead of playing audio,
    /// MP4/WebM and YouTube inside the app.
    #[serde(default)]
    pub external_only: bool,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            command: default_player_command(),
            args: default_player_args(),
            external_only: false,
        }
    }
}

/// YouTube Data API v3, for indexing channels and playlists.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YouTube {
    #[serde(default)]
    pub api_key: String,
}

/// The watch assistant (Google Gemini).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assistant {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_assistant_model")]
    pub model: String,
}

impl Default for Assistant {
    fn default() -> Self {
        Assistant {
            api_key: String::new(),
            model: default_assistant_model(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            db_path: default_db_path(),
            concurrency: default_concurrency(),
            delay_ms: default_delay_ms(),
            timeout_secs: default_timeout_secs(),
            max_depth: default_max_depth(),
            media_extensions: default_media_extensions(),
            skip_dirs: ["sample"].map(String::from).to_vec(),
            ui: Ui::default(),
            player: Player::default(),
            youtube: YouTube::default(),
            assistant: Assistant::default(),
            roots: Vec::new(),
            base_dir: PathBuf::from("."),
        }
    }
}

fn default_db_path() -> String {
    "noor.db".into()
}
fn default_concurrency() -> usize {
    6
}
fn default_delay_ms() -> u64 {
    250
}
fn default_timeout_secs() -> u64 {
    20
}
fn default_max_depth() -> usize {
    12
}
fn default_language() -> String {
    "bn".into()
}
fn default_player_args() -> Vec<String> {
    vec!["{url}".into()]
}
fn default_assistant_model() -> String {
    // An alias Google keeps pointing at its current Flash model; pinned
    // versions get retired ("no longer available to new users").
    "gemini-flash-latest".into()
}
fn default_media_extensions() -> Vec<String> {
    [
        // audio: most lectures, recitations and nasheeds
        "mp3", "m4a", "aac", "ogg", "opus", "wav", "flac", "wma", "amr",
        // video
        "mp4", "m4v", "mkv", "webm", "avi", "mov", "wmv", "flv", "3gp", "ts", "mpg", "mpeg",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// VLC's installer doesn't put `vlc` on PATH on Windows, so prefer the
/// standard install location when it exists.
fn default_player_command() -> String {
    if cfg!(windows) {
        for base in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Ok(dir) = std::env::var(base) {
                let exe = Path::new(&dir).join("VideoLAN").join("VLC").join("vlc.exe");
                if exe.is_file() {
                    return exe.to_string_lossy().into_owned();
                }
            }
        }
    }
    "vlc".into()
}

const FILE_HEADER: &str = "\
# Noor Library configuration (saved by the app; hand edits are fine too).
#
# Every bare key must come BEFORE the first [table] or [[roots]] header:
# in TOML a bare key belongs to the most recent header above it, so a
# `skip_dirs` written below [[roots]] would become a field of that root.

";

impl Config {
    pub fn load(path: &Path) -> Result<Config> {
        let text = std::fs::read_to_string(path).with_context(|| {
            format!(
                "reading config {} (copy config.example.toml to get started)",
                path.display()
            )
        })?;
        let mut cfg =
            Self::from_toml_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        cfg.base_dir = base_dir_of(path);
        Ok(cfg)
    }

    /// Loads the config, or writes and returns defaults if there is none.
    pub fn load_or_create(path: &Path) -> Result<Config> {
        if path.exists() {
            return Self::load(path);
        }
        let mut cfg = Config::default();
        cfg.base_dir = base_dir_of(path);
        cfg.save(path)?;
        Ok(cfg)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let body = toml::to_string_pretty(self).context("serialising config")?;
        // Write-then-rename so a crash mid-write can't leave a truncated file.
        let tmp = path.with_extension("toml.tmp");
        std::fs::write(&tmp, format!("{FILE_HEADER}{body}"))
            .with_context(|| format!("writing {}", tmp.display()))?;
        std::fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
        Ok(())
    }

    pub fn from_toml_str(text: &str) -> Result<Config> {
        let mut cfg: Config = toml::from_str(text).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("unknown field") {
                anyhow!(
                    "{msg}\nhint: bare keys (skip_dirs, media_extensions, ...) must come before \
                     the first [table] or [[roots]] header; below it they belong to that table"
                )
            } else {
                anyhow!(msg)
            }
        })?;
        cfg.normalise()?;
        Ok(cfg)
    }

    /// Validates and canonicalises the config in place. Also used by the app
    /// after every edit, so the file on disk is always loadable.
    pub fn normalise(&mut self) -> Result<()> {
        self.concurrency = self.concurrency.clamp(1, 64);
        self.max_depth = self.max_depth.max(1);
        self.timeout_secs = self.timeout_secs.max(1);
        self.media_extensions = dedup(
            self.media_extensions
                .iter()
                .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase()),
        );
        self.skip_dirs = dedup(self.skip_dirs.iter().map(|s| s.trim().to_lowercase()));
        if !matches!(self.ui.language.as_str(), "bn" | "en") {
            self.ui.language = default_language();
        }
        self.youtube.api_key = self.youtube.api_key.trim().to_string();
        self.assistant.api_key = self.assistant.api_key.trim().to_string();
        self.assistant.model = self.assistant.model.trim().trim_start_matches("models/").to_string();
        if self.assistant.model.is_empty() {
            self.assistant.model = default_assistant_model();
        }

        let mut names = HashSet::new();
        for root in &mut self.roots {
            root.name = root.name.trim().to_string();
            if root.name.is_empty() {
                bail!("every source needs a name");
            }
            if !names.insert(root.name.to_lowercase()) {
                bail!("duplicate source name {:?}", root.name);
            }
            root.speaker = root.speaker.take().map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
            let set = [&root.url, &root.path, &root.youtube, &root.feed]
                .iter()
                .filter(|v| v.is_some())
                .count();
            if set != 1 {
                bail!("source {:?}: set exactly one of url, path, youtube or feed", root.name);
            }
            if let Some(url) = &mut root.url {
                // The trailing slash matters twice: `Url::join` resolves
                // relative hrefs against the last `/`, and the crawler's
                // "stay inside the root" check is a prefix match.
                *url = url.trim().to_string();
                if !url.ends_with('/') {
                    url.push('/');
                }
                let parsed = Url::parse(url)
                    .with_context(|| format!("source {:?}: bad url {url:?}", root.name))?;
                if !matches!(parsed.scheme(), "http" | "https") {
                    bail!("source {:?}: only http(s) links are supported", root.name);
                }
            }
            if let Some(feed) = &mut root.feed {
                *feed = feed.trim().to_string();
                let parsed = Url::parse(feed)
                    .with_context(|| format!("source {:?}: bad feed url {feed:?}", root.name))?;
                if !matches!(parsed.scheme(), "http" | "https") {
                    bail!("source {:?}: feeds must be http(s) links", root.name);
                }
            }
            if let Some(yt) = &mut root.youtube {
                *yt = yt.trim().to_string();
            }
        }
        Ok(())
    }

    /// The SQLite file, with relative paths taken from the config's folder.
    pub fn db_file(&self) -> PathBuf {
        let p = Path::new(&self.db_path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.base_dir.join(p)
        }
    }

    pub fn is_media(&self, name: &str) -> bool {
        name.rsplit_once('.').is_some_and(|(_, ext)| {
            let ext = ext.to_ascii_lowercase();
            self.media_extensions.iter().any(|m| *m == ext)
        })
    }

    pub fn should_skip_dir(&self, name: &str) -> bool {
        let name = name.trim_end_matches(['/', '\\']).to_lowercase();
        self.skip_dirs.iter().any(|s| name.contains(s.as_str()))
    }

    pub fn root(&self, name: &str) -> Option<&Root> {
        self.roots.iter().find(|r| r.name == name)
    }
}

impl Root {
    pub fn kind(&self) -> Result<RootKind> {
        if let Some(url) = &self.url {
            return Ok(RootKind::Http(Url::parse(url).with_context(|| {
                format!("source {:?}: bad url {url:?}", self.name)
            })?));
        }
        if let Some(path) = &self.path {
            return Ok(RootKind::Local(PathBuf::from(path)));
        }
        if let Some(yt) = &self.youtube {
            return Ok(RootKind::YouTube(yt.clone()));
        }
        if let Some(feed) = &self.feed {
            return Ok(RootKind::Feed(Url::parse(feed).with_context(|| {
                format!("source {:?}: bad feed url {feed:?}", self.name)
            })?));
        }
        bail!("source {:?} has no location", self.name)
    }

    /// Human-readable location, for listings.
    pub fn location(&self) -> &str {
        self.url
            .as_deref()
            .or(self.path.as_deref())
            .or(self.youtube.as_deref())
            .or(self.feed.as_deref())
            .unwrap_or("")
    }

    /// "web", "folder", "youtube" or "feed".
    pub fn origin(&self) -> &'static str {
        if self.url.is_some() {
            "web"
        } else if self.path.is_some() {
            "folder"
        } else if self.youtube.is_some() {
            "youtube"
        } else {
            "feed"
        }
    }
}

fn base_dir_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

fn dedup(items: impl Iterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
skip_dirs = ["Sample"]
media_extensions = [".MP3", "mp4"]

[[roots]]
name = "Waz"
url = "http://media.local/waz"
speaker_folders = true

[[roots]]
name = "Tafsir"
path = 'D:\Tafsir'
kind = "series"
speaker = "Some Speaker"

[[roots]]
name = "Channel"
youtube = "https://www.youtube.com/@example"

[[roots]]
name = "Podcast"
feed = "https://example.com/feed.xml"
kind = "series"
"#;

    #[test]
    fn parses_and_normalises() {
        let cfg = Config::from_toml_str(GOOD).unwrap();
        assert_eq!(cfg.roots[0].url.as_deref(), Some("http://media.local/waz/"));
        assert!(cfg.roots[0].speaker_folders);
        assert!(matches!(cfg.roots[1].kind().unwrap(), RootKind::Local(_)));
        assert_eq!(cfg.roots[1].kind, SourceKind::Series);
        assert_eq!(cfg.roots[2].origin(), "youtube");
        assert_eq!(cfg.roots[3].origin(), "feed");
        assert!(cfg.is_media("Lecture.mp3"));
        assert!(!cfg.is_media("readme.txt"));
        assert_eq!(cfg.ui.language, "bn");
    }

    #[test]
    fn key_below_roots_is_an_error_not_silently_ignored() {
        let text = "[[roots]]\nname = \"x\"\nurl = \"http://a/\"\nskip_dirs = [\"sample\"]\n";
        let err = format!("{:#}", Config::from_toml_str(text).unwrap_err());
        assert!(err.contains("skip_dirs"), "{err}");
    }

    #[test]
    fn source_needs_exactly_one_location() {
        let both = "[[roots]]\nname = \"x\"\nurl = \"http://a/\"\nfeed = \"http://b/f.xml\"\n";
        assert!(Config::from_toml_str(both).is_err());
        let neither = "[[roots]]\nname = \"x\"\n";
        assert!(Config::from_toml_str(neither).is_err());
    }

    #[test]
    fn save_round_trips() {
        let cfg = Config::from_toml_str(GOOD).unwrap();
        let text = format!("{FILE_HEADER}{}", toml::to_string_pretty(&cfg).unwrap());
        let back = Config::from_toml_str(&text).unwrap();
        assert_eq!(back.roots.len(), 4);
        assert_eq!(back.roots[1].speaker.as_deref(), Some("Some Speaker"));
    }
}
