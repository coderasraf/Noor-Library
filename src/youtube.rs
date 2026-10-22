//! YouTube channels and playlists as a source, through the YouTube Data
//! API v3. Each video becomes a "file" whose URL is its watch page; the app
//! plays it in an embedded YouTube player.
//!
//! Quota: listing costs one unit per 50 videos (playlistItems) plus one per
//! 50 for durations (videos), so even a 3,000-video channel uses ~120 of
//! the free 10,000 daily units.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::Client;
use serde_json::Value;
use url::Url;

use crate::config::{Config, Root};
use crate::crawler::{CrawlStats, Progress};
use crate::index::{unix_now, FileRow, Index, SourceMetaRow};

const API: &str = "https://www.googleapis.com/youtube/v3";

/// What a pasted YouTube link points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    ChannelId(String),
    Handle(String),
    Username(String),
    Playlist(String),
}

/// Understands channel links (/@handle, /channel/UC…, /user/…, /c/…),
/// playlist links (…list=PL…) and bare "@handle" / "UC…" / "PL…" ids.
pub fn parse_target(input: &str) -> Result<Target> {
    let s = input.trim();
    if let Some(h) = s.strip_prefix('@') {
        return Ok(Target::Handle(h.to_string()));
    }
    if s.starts_with("UC") && s.len() == 24 && !s.contains('/') {
        return Ok(Target::ChannelId(s.to_string()));
    }
    if (s.starts_with("PL") || s.starts_with("UU") || s.starts_with("OL")) && !s.contains('/') {
        return Ok(Target::Playlist(s.to_string()));
    }
    let with_scheme = if s.starts_with("http") { s.to_string() } else { format!("https://{s}") };
    let url = Url::parse(&with_scheme).context("not a YouTube link")?;
    let host = url.host_str().unwrap_or("");
    if !(host.ends_with("youtube.com") || host == "youtu.be") {
        bail!("not a YouTube link");
    }
    if let Some((_, list)) = url.query_pairs().find(|(k, _)| k == "list") {
        return Ok(Target::Playlist(list.into_owned()));
    }
    let segs: Vec<&str> = url.path_segments().map(|s| s.filter(|x| !x.is_empty()).collect()).unwrap_or_default();
    match segs.as_slice() {
        [h, ..] if h.starts_with('@') => Ok(Target::Handle(h[1..].to_string())),
        ["channel", id, ..] => Ok(Target::ChannelId(id.to_string())),
        ["user", name, ..] => Ok(Target::Username(name.to_string())),
        // Custom /c/ names usually match the handle.
        ["c", name, ..] => Ok(Target::Handle(name.to_string())),
        _ => bail!("paste a channel link (youtube.com/@name) or a playlist link (…list=…)"),
    }
}

/// A channel or playlist with its videos.
#[derive(Debug, Default)]
pub struct Listing {
    pub title: String,
    pub author: Option<String>,
    pub image: Option<String>,
    pub description: Option<String>,
    pub link: String,
    pub videos: Vec<Video>,
}

#[derive(Debug, Default, Clone)]
pub struct Video {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub published: Option<i64>,
    pub thumb: Option<String>,
    pub channel: Option<String>,
    pub position: i64,
    pub duration: Option<i64>,
    pub views: Option<i64>,
    pub likes: Option<i64>,
    pub comments: Option<i64>,
}

pub struct YouTube {
    client: Client,
    key: String,
}

impl YouTube {
    pub fn new(key: &str) -> Result<YouTube> {
        if key.trim().is_empty() {
            bail!("add a YouTube API key in Settings → YouTube first");
        }
        Ok(YouTube {
            client: Client::builder().timeout(Duration::from_secs(30)).build()?,
            key: key.trim().to_string(),
        })
    }

    async fn get(&self, path: &str, query: &[(&str, &str)]) -> Result<Value> {
        let resp = self
            .client
            .get(format!("{API}/{path}"))
            .query(query)
            .query(&[("key", self.key.as_str())])
            .send()
            .await?;
        let status = resp.status();
        let v: Value = resp.json().await.unwrap_or(Value::Null);
        if !status.is_success() {
            let reason = v["error"]["errors"][0]["reason"].as_str().unwrap_or("");
            let msg = v["error"]["message"].as_str().unwrap_or("request failed");
            bail!(
                "{}",
                match reason {
                    "quotaExceeded" => "YouTube's daily quota for this key is used up; try again tomorrow".to_string(),
                    "keyInvalid" => "YouTube rejected the API key".to_string(),
                    _ if msg.contains("API key not valid") => "YouTube rejected the API key".to_string(),
                    _ if msg.contains("has not been used") || msg.contains("is disabled") =>
                        "enable \"YouTube Data API v3\" for this key in Google Cloud Console".to_string(),
                    _ => format!("YouTube: {msg}"),
                }
            );
        }
        Ok(v)
    }

    /// A cheap call that validates the key.
    pub async fn check(&self) -> Result<()> {
        self.get("videos", &[("part", "id"), ("id", "dQw4w9WgXcQ")]).await.map(|_| ())
    }

    /// Name, picture and video count, without listing the videos. Used by
    /// "Check link".
    pub async fn describe(&self, target: &Target) -> Result<(String, Option<String>, i64, bool)> {
        match target {
            Target::Playlist(id) => {
                let v = self.get("playlists", &[("part", "snippet,contentDetails"), ("id", id)]).await?;
                let p = v["items"].get(0).ok_or_else(|| anyhow!("that playlist doesn't exist or is private"))?;
                Ok((
                    text(&p["snippet"], "title").unwrap_or_default(),
                    text(&p["snippet"], "channelTitle"),
                    p["contentDetails"]["itemCount"].as_i64().unwrap_or(0),
                    true,
                ))
            }
            _ => {
                let (k, v) = match target {
                    Target::ChannelId(id) => ("id", id.clone()),
                    Target::Handle(h) => ("forHandle", format!("@{h}")),
                    Target::Username(u) => ("forUsername", u.clone()),
                    Target::Playlist(_) => unreachable!(),
                };
                let v = self.get("channels", &[("part", "snippet,statistics"), (k, &v)]).await?;
                let c = v["items"].get(0).ok_or_else(|| anyhow!("couldn't find that channel"))?;
                let title = text(&c["snippet"], "title").unwrap_or_default();
                let count = c["statistics"]["videoCount"].as_str().and_then(|n| n.parse().ok()).unwrap_or(0);
                Ok((title.clone(), Some(title), count, false))
            }
        }
    }

    /// Resolves the target and lists every video, calling `on_page` with the
    /// running count.
    pub async fn list(&self, target: &Target, mut on_page: impl FnMut(usize)) -> Result<Listing> {
        let (playlist, mut listing) = match target {
            Target::Playlist(id) => {
                let v = self.get("playlists", &[("part", "snippet"), ("id", id)]).await?;
                let p = v["items"].get(0).ok_or_else(|| anyhow!("that playlist doesn't exist or is private"))?;
                let sn = &p["snippet"];
                (
                    id.clone(),
                    Listing {
                        title: text(sn, "title").unwrap_or_else(|| id.clone()),
                        author: text(sn, "channelTitle"),
                        image: best_thumb(&sn["thumbnails"]),
                        description: text(sn, "description"),
                        link: format!("https://www.youtube.com/playlist?list={id}"),
                        ..Listing::default()
                    },
                )
            }
            _ => {
                let (k, v) = match target {
                    Target::ChannelId(id) => ("id", id.clone()),
                    Target::Handle(h) => ("forHandle", format!("@{h}")),
                    Target::Username(u) => ("forUsername", u.clone()),
                    Target::Playlist(_) => unreachable!(),
                };
                let v = self.get("channels", &[("part", "snippet,contentDetails"), (k, &v)]).await?;
                let c = v["items"].get(0).ok_or_else(|| anyhow!("couldn't find that channel"))?;
                let sn = &c["snippet"];
                let uploads = c["contentDetails"]["relatedPlaylists"]["uploads"]
                    .as_str()
                    .ok_or_else(|| anyhow!("that channel has no public videos"))?
                    .to_string();
                let title = text(sn, "title").unwrap_or_default();
                (
                    uploads,
                    Listing {
                        author: Some(title.clone()),
                        title,
                        image: best_thumb(&sn["thumbnails"]),
                        description: text(sn, "description"),
                        link: format!("https://www.youtube.com/channel/{}", c["id"].as_str().unwrap_or("")),
                        ..Listing::default()
                    },
                )
            }
        };

        let mut page: Option<String> = None;
        loop {
            let mut q = vec![("part", "snippet,contentDetails"), ("playlistId", playlist.as_str()), ("maxResults", "50")];
            if let Some(p) = &page {
                q.push(("pageToken", p.as_str()));
            }
            let v = self.get("playlistItems", &q).await?;
            for item in v["items"].as_array().into_iter().flatten() {
                let sn = &item["snippet"];
                let title = text(sn, "title").unwrap_or_default();
                // Private and deleted videos stay in playlists as stubs.
                if title == "Private video" || title == "Deleted video" {
                    continue;
                }
                let Some(id) = sn["resourceId"]["videoId"].as_str() else { continue };
                listing.videos.push(Video {
                    id: id.to_string(),
                    title,
                    description: text(sn, "description"),
                    published: item["contentDetails"]["videoPublishedAt"]
                        .as_str()
                        .or(sn["publishedAt"].as_str())
                        .and_then(crate::feed::parse_date),
                    thumb: best_thumb(&sn["thumbnails"]),
                    channel: text(sn, "videoOwnerChannelTitle").or_else(|| listing.author.clone()),
                    position: sn["position"].as_i64().unwrap_or(listing.videos.len() as i64),
                    duration: None,
                    views: None,
                    likes: None,
                    comments: None,
                });
            }
            on_page(listing.videos.len());
            match v["nextPageToken"].as_str() {
                Some(p) => page = Some(p.to_string()),
                None => break,
            }
        }

        // Durations and view / like counts, 50 videos per call.
        for chunk in listing.videos.chunks_mut(50) {
            let ids: Vec<&str> = chunk.iter().map(|v| v.id.as_str()).collect();
            let joined = ids.join(",");
            let v = self.get("videos", &[("part", "contentDetails,statistics"), ("id", &joined)]).await?;
            for item in v["items"].as_array().into_iter().flatten() {
                let Some(video) = item["id"].as_str().and_then(|id| chunk.iter_mut().find(|v| v.id == id)) else { continue };
                video.duration = item["contentDetails"]["duration"].as_str().and_then(parse_iso_duration);
                // Counts come as strings; a hidden like count is missing.
                let count = |k: &str| item["statistics"][k].as_str().and_then(|n| n.parse().ok());
                video.views = count("viewCount");
                video.likes = count("likeCount");
                video.comments = count("commentCount");
            }
        }
        // A channel's uploads include Shorts, which bury the lectures. A
        // playlist was put together by hand, so it's kept whole. Videos with
        // no duration are upcoming or live right now.
        if !matches!(target, Target::Playlist(_)) {
            listing.videos.retain(|v| v.duration.is_some_and(|d| d > SHORTS_MAX_SECS));
        }
        Ok(listing)
    }
}

/// YouTube Shorts run up to three minutes.
const SHORTS_MAX_SECS: i64 = 180;

fn text(v: &Value, key: &str) -> Option<String> {
    v[key].as_str().map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

/// The largest thumbnail offered.
fn best_thumb(thumbs: &Value) -> Option<String> {
    ["maxres", "standard", "high", "medium", "default"]
        .iter()
        .find_map(|k| thumbs[k]["url"].as_str().map(String::from))
}

/// "PT1H2M3S" -> 3723 seconds.
pub fn parse_iso_duration(s: &str) -> Option<i64> {
    let s = s.strip_prefix('P')?;
    let (days, time) = match s.split_once('T') {
        Some((d, t)) => (d, t),
        None => (s, ""),
    };
    let mut total = 0;
    let mut num = String::new();
    for c in days.chars() {
        if c.is_ascii_digit() {
            num.push(c);
        } else {
            if c == 'D' {
                total += num.parse::<i64>().ok()? * 86_400;
            }
            num.clear();
        }
    }
    for c in time.chars() {
        if c.is_ascii_digit() || c == '.' {
            num.push(c);
        } else {
            let n = num.parse::<f64>().ok()? as i64;
            total += match c {
                'H' => n * 3600,
                'M' => n * 60,
                'S' => n,
                _ => 0,
            };
            num.clear();
        }
    }
    (total > 0).then_some(total)
}

pub fn watch_url(id: &str) -> String {
    format!("https://www.youtube.com/watch?v={id}")
}

/// Video id from a watch URL, for the embedded player.
pub fn video_id(url: &str) -> Option<String> {
    let u = Url::parse(url).ok()?;
    let host = u.host_str()?;
    if host == "youtu.be" {
        return u.path_segments()?.next().map(String::from);
    }
    if !host.ends_with("youtube.com") {
        return None;
    }
    u.query_pairs().find(|(k, _)| k == "v").map(|(_, v)| v.into_owned())
}

pub fn crawl(cfg: &Config, index: &mut Index, root: &Root, link: &str, progress: &dyn Progress) -> Result<CrawlStats> {
    let started = unix_now();
    let target = parse_target(link)?;
    let yt = YouTube::new(&cfg.youtube.api_key)?;
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    progress.log(&format!("[{}] listing YouTube {link}", root.name));
    let mut stats = CrawlStats::default();
    let listing = rt.block_on(yt.list(&target, |n| {
        stats.files = n;
        progress.update(&root.name, &stats);
    }))?;

    let speaker = root.speaker.clone().or(listing.author.clone());
    let rows: Vec<FileRow> = listing
        .videos
        .iter()
        .map(|v| FileRow {
            root: root.name.clone(),
            url: watch_url(&v.id),
            name: format!("{}.youtube", v.id),
            dir: listing.link.clone(),
            title: v.title.clone(),
            label: Some(v.title.clone()),
            duration: v.duration,
            published_at: v.published,
            thumb: v.thumb.clone(),
            description: v.description.clone(),
            speaker: root.speaker.clone().or(v.channel.clone()).or(speaker.clone()),
            series: Some(listing.title.clone()),
            position: Some(v.position),
            views: v.views,
            likes: v.likes,
            comments: v.comments,
            ..FileRow::default()
        })
        .collect();
    index.record_level(&root.name, &rows, &[], &[], &[])?;
    index.set_source_meta(&SourceMetaRow {
        root: root.name.clone(),
        title: Some(listing.title.clone()),
        image: listing.image.clone(),
        author: speaker,
        description: listing.description.clone(),
        link: Some(listing.link.clone()),
    })?;
    stats.pruned = index.prune(&root.name, started)?;
    stats.files = rows.len();
    stats.dirs = 1;
    progress.update(&root.name, &stats);
    progress.log(&format!("[{}] {} videos", root.name, rows.len()));
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets() {
        assert_eq!(parse_target("https://www.youtube.com/@AssimAlHakeem").unwrap(), Target::Handle("AssimAlHakeem".into()));
        assert_eq!(parse_target("@yaqeeninstitute").unwrap(), Target::Handle("yaqeeninstitute".into()));
        assert_eq!(
            parse_target("https://www.youtube.com/channel/UCxxxxxxxxxxxxxxxxxxxxxx").unwrap(),
            Target::ChannelId("UCxxxxxxxxxxxxxxxxxxxxxx".into())
        );
        assert_eq!(
            parse_target("https://www.youtube.com/playlist?list=PLabc123").unwrap(),
            Target::Playlist("PLabc123".into())
        );
        assert_eq!(
            parse_target("https://www.youtube.com/watch?v=abc&list=PLxyz").unwrap(),
            Target::Playlist("PLxyz".into())
        );
        assert_eq!(parse_target("youtube.com/user/someone").unwrap(), Target::Username("someone".into()));
        assert!(parse_target("https://vimeo.com/123").is_err());
    }

    #[test]
    fn durations_and_ids() {
        assert_eq!(parse_iso_duration("PT1H2M3S"), Some(3723));
        assert_eq!(parse_iso_duration("PT45M"), Some(2700));
        assert_eq!(parse_iso_duration("P1DT1S"), Some(86_401));
        assert_eq!(parse_iso_duration("P0D"), None);
        assert_eq!(video_id("https://www.youtube.com/watch?v=abc123"), Some("abc123".into()));
        assert_eq!(video_id("https://youtu.be/xyz"), Some("xyz".into()));
        assert_eq!(video_id("https://example.com/watch?v=abc"), None);
    }
}
