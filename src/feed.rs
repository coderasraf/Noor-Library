//! Podcast / RSS feeds as a source: each episode with an audio or video
//! enclosure becomes a file; the feed itself gives the series name, artwork
//! and speaker.

use anyhow::{bail, Context, Result};
use roxmltree::{Document, Node};

use crate::config::{Config, Root};
use crate::crawler::{CrawlStats, Progress};
use crate::index::{unix_now, FileRow, Index, SourceMetaRow};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Feed {
    pub title: Option<String>,
    pub author: Option<String>,
    pub image: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub items: Vec<Episode>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Episode {
    pub title: String,
    pub url: String,
    pub size: Option<i64>,
    pub duration: Option<i64>,
    pub published: Option<i64>,
    pub image: Option<String>,
    pub description: Option<String>,
}

pub fn crawl(cfg: &Config, index: &mut Index, root: &Root, url: &url::Url, progress: &dyn Progress) -> Result<CrawlStats> {
    let started = unix_now();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
    let client = crate::crawler::client(cfg)?;
    progress.log(&format!("[{}] reading feed {url}", root.name));
    let xml = rt
        .block_on(async {
            let resp = client.get(url.clone()).send().await?.error_for_status()?;
            resp.text().await
        })
        .with_context(|| format!("downloading {url}"))?;
    let feed = parse(&xml)?;

    let author = root.speaker.clone().or(feed.author.clone());
    let rows: Vec<FileRow> = feed
        .items
        .iter()
        .map(|e| FileRow {
            root: root.name.clone(),
            url: e.url.clone(),
            name: file_name(&e.url).unwrap_or_else(|| e.title.clone()),
            dir: url.to_string(),
            size: e.size,
            title: e.title.clone(),
            label: Some(e.title.clone()),
            duration: e.duration,
            published_at: e.published,
            thumb: e.image.clone().or(feed.image.clone()),
            description: e.description.clone(),
            speaker: author.clone(),
            series: feed.title.clone(),
            ..FileRow::default()
        })
        .collect();
    index.record_level(&root.name, &rows, &[], &[], &[])?;
    index.set_source_meta(&SourceMetaRow {
        root: root.name.clone(),
        title: feed.title.clone(),
        image: feed.image.clone(),
        author,
        description: feed.description.clone(),
        link: Some(url.to_string()),
    })?;
    let pruned = index.prune(&root.name, started)?;
    let stats = CrawlStats {
        dirs: 1,
        files: rows.len(),
        pruned,
        ..CrawlStats::default()
    };
    progress.update(&root.name, &stats);
    progress.log(&format!("[{}] {} episodes", root.name, rows.len()));
    Ok(stats)
}

/// Last path segment of a URL, decoded ("ep12.mp3").
fn file_name(url: &str) -> Option<String> {
    let u = url::Url::parse(url).ok()?;
    let last = u.path_segments()?.rfind(|s| !s.is_empty())?;
    Some(crate::crawler::percent_decode(last))
}

/// Parses RSS 2.0 (with iTunes podcast tags) or Atom.
pub fn parse(xml: &str) -> Result<Feed> {
    let doc = Document::parse(xml).context("the feed isn't valid XML")?;
    let root = doc.root_element();
    match root.tag_name().name() {
        "rss" => {
            let channel = child(root, "channel").context("RSS feed without a <channel>")?;
            Ok(parse_rss(channel))
        }
        "feed" => Ok(parse_atom(root)),
        other => bail!("not a podcast feed (root element <{other}>)"),
    }
}

fn parse_rss(channel: Node) -> Feed {
    let channel_image = attr_of(channel, "image", "href")
        .or_else(|| child(channel, "image").and_then(|i| text_of(i, "url")));
    let items = channel
        .children()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|item| {
            let enclosure = item
                .children()
                .find(|n| n.tag_name().name() == "enclosure" && is_media(n.attribute("type"), n.attribute("url")))
                .or_else(|| item.children().find(|n| n.tag_name().name() == "content" && is_media(n.attribute("type"), n.attribute("url"))))?;
            let url = enclosure.attribute("url")?.trim().to_string();
            Some(Episode {
                title: text_of(item, "title").unwrap_or_else(|| url.clone()),
                size: enclosure.attribute("length").or(enclosure.attribute("fileSize")).and_then(|l| l.parse().ok()).filter(|n: &i64| *n > 0),
                duration: text_of(item, "duration").and_then(|d| parse_duration(&d)),
                published: text_of(item, "pubDate").and_then(|d| parse_date(&d)),
                image: attr_of(item, "image", "href"),
                description: text_of(item, "summary")
                    .or_else(|| text_of(item, "description"))
                    .map(|d| strip_html(&d))
                    .filter(|d| !d.is_empty()),
                url,
            })
        })
        .collect();
    Feed {
        title: text_of(channel, "title"),
        author: text_of(channel, "author").or_else(|| text_of(channel, "creator")),
        image: channel_image,
        description: text_of(channel, "summary")
            .or_else(|| text_of(channel, "description"))
            .map(|d| strip_html(&d)),
        link: text_of(channel, "link"),
        items,
    }
}

fn parse_atom(feed: Node) -> Feed {
    let items = feed
        .children()
        .filter(|n| n.has_tag_name("entry"))
        .filter_map(|entry| {
            let link = entry.children().find(|n| {
                n.tag_name().name() == "link"
                    && n.attribute("rel") == Some("enclosure")
                    && is_media(n.attribute("type"), n.attribute("href"))
            })?;
            let url = link.attribute("href")?.trim().to_string();
            Some(Episode {
                title: text_of(entry, "title").unwrap_or_else(|| url.clone()),
                size: link.attribute("length").and_then(|l| l.parse().ok()),
                duration: None,
                published: text_of(entry, "published").or_else(|| text_of(entry, "updated")).and_then(|d| parse_date(&d)),
                image: None,
                description: text_of(entry, "summary").map(|d| strip_html(&d)),
                url,
            })
        })
        .collect();
    Feed {
        title: text_of(feed, "title"),
        author: child(feed, "author").and_then(|a| text_of(a, "name")),
        image: text_of(feed, "logo").or_else(|| text_of(feed, "icon")),
        description: text_of(feed, "subtitle"),
        link: None,
        items,
    }
}

fn is_media(mime: Option<&str>, url: Option<&str>) -> bool {
    if let Some(m) = mime {
        if m.starts_with("audio/") || m.starts_with("video/") {
            return true;
        }
    }
    url.and_then(|u| u.split(['?', '#']).next())
        .and_then(|u| u.rsplit_once('.'))
        .is_some_and(|(_, ext)| {
            matches!(ext.to_ascii_lowercase().as_str(), "mp3" | "m4a" | "aac" | "ogg" | "opus" | "wav" | "mp4" | "m4v" | "webm")
        })
}

/// First child element with this local name (any namespace).
fn child<'a, 'i>(node: Node<'a, 'i>, name: &str) -> Option<Node<'a, 'i>> {
    node.children().find(|n| n.is_element() && n.tag_name().name() == name)
}

fn text_of(node: Node, name: &str) -> Option<String> {
    let c = child(node, name)?;
    let text: String = c.descendants().filter(|n| n.is_text()).filter_map(|n| n.text()).collect();
    let t = text.trim();
    (!t.is_empty()).then(|| t.to_string())
}

fn attr_of(node: Node, name: &str, attr: &str) -> Option<String> {
    node.children()
        .filter(|n| n.is_element() && n.tag_name().name() == name)
        .find_map(|n| n.attribute(attr).map(|a| a.trim().to_string()))
        .filter(|a| !a.is_empty())
}

/// "3600", "62:03" or "1:02:03" -> seconds.
pub fn parse_duration(s: &str) -> Option<i64> {
    let parts: Vec<i64> = s.trim().split(':').map(|p| p.trim().parse::<f64>().ok().map(|f| f as i64)).collect::<Option<_>>()?;
    let secs = match parts.as_slice() {
        [s] => *s,
        [m, s] => m * 60 + s,
        [h, m, s] => h * 3600 + m * 60 + s,
        _ => return None,
    };
    (secs > 0).then_some(secs)
}

/// RFC 2822 ("Tue, 10 Jun 2003 04:00:00 +0600") or RFC 3339
/// ("2003-06-10T04:00:00Z") -> Unix seconds. Feeds are sloppy about the
/// rest, so only the date and time are read; the zone is applied when it
/// is a plain offset.
pub fn parse_date(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() >= 10 && s.as_bytes()[4] == b'-' {
        let y: i64 = s.get(0..4)?.parse().ok()?;
        let mo: i64 = s.get(5..7)?.parse().ok()?;
        let d: i64 = s.get(8..10)?.parse().ok()?;
        let (h, mi, se) = if s.len() >= 19 {
            (s.get(11..13)?.parse().ok()?, s.get(14..16)?.parse().ok()?, s.get(17..19)?.parse().ok()?)
        } else {
            (0, 0, 0)
        };
        let offset = s.get(19..).and_then(offset_seconds).unwrap_or(0);
        return Some(days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + se - offset);
    }
    let rest = s.split_once(',').map_or(s, |(_, r)| r).trim();
    let mut it = rest.split_whitespace();
    let d: i64 = it.next()?.parse().ok()?;
    let mo = month(it.next()?)?;
    let mut y: i64 = it.next()?.parse().ok()?;
    if y < 100 {
        y += 2000;
    }
    let (mut h, mut mi, mut se) = (0, 0, 0);
    if let Some(t) = it.next() {
        let p: Vec<i64> = t.split(':').filter_map(|x| x.parse().ok()).collect();
        h = *p.first().unwrap_or(&0);
        mi = *p.get(1).unwrap_or(&0);
        se = *p.get(2).unwrap_or(&0);
    }
    let offset = it.next().and_then(offset_seconds).unwrap_or(0);
    Some(days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + se - offset)
}

fn offset_seconds(z: &str) -> Option<i64> {
    let z = z.trim().trim_start_matches(|c: char| c == '.' || c.is_ascii_digit());
    if z.is_empty() || z == "Z" || z == "GMT" || z == "UT" || z == "UTC" {
        return Some(0);
    }
    let sign = match z.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let digits: String = z[1..].chars().filter(char::is_ascii_digit).collect();
    if digits.len() < 4 {
        return None;
    }
    let h: i64 = digits[0..2].parse().ok()?;
    let m: i64 = digits[2..4].parse().ok()?;
    Some(sign * (h * 3600 + m * 60))
}

fn month(m: &str) -> Option<i64> {
    let m = m.get(0..3)?.to_ascii_lowercase();
    ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"]
        .iter()
        .position(|x| *x == m)
        .map(|i| i as i64 + 1)
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Plain text from a feed's HTML description.
pub fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:itunes="http://www.itunes.com/dtds/podcast-1.0.dtd">
<channel>
  <title>Daily Reminders</title>
  <link>https://example.org</link>
  <itunes:author>Some Speaker</itunes:author>
  <itunes:image href="https://example.org/art.jpg"/>
  <description><![CDATA[<p>Short <b>reminders</b> &amp; talks</p>]]></description>
  <item>
    <title>Episode 2: Patience</title>
    <enclosure url="https://cdn.example.org/ep2.mp3" length="1234" type="audio/mpeg"/>
    <pubDate>Tue, 10 Jun 2003 04:00:00 +0600</pubDate>
    <itunes:duration>1:02:03</itunes:duration>
  </item>
  <item>
    <title>Blog post without audio</title>
  </item>
  <item>
    <title>Episode 1: Gratitude</title>
    <enclosure url="https://cdn.example.org/ep1.m4a" type="audio/x-m4a"/>
    <pubDate>Mon, 09 Jun 2003 04:00:00 GMT</pubDate>
    <itunes:duration>62:03</itunes:duration>
    <itunes:image href="https://example.org/ep1.jpg"/>
  </item>
</channel>
</rss>"#;

    #[test]
    fn parses_podcast_rss() {
        let f = parse(RSS).unwrap();
        assert_eq!(f.title.as_deref(), Some("Daily Reminders"));
        assert_eq!(f.author.as_deref(), Some("Some Speaker"));
        assert_eq!(f.image.as_deref(), Some("https://example.org/art.jpg"));
        assert_eq!(f.description.as_deref(), Some("Short reminders & talks"));
        assert_eq!(f.items.len(), 2, "items without audio are skipped");
        assert_eq!(f.items[0].duration, Some(3723));
        assert_eq!(f.items[1].duration, Some(3723));
        assert_eq!(f.items[0].size, Some(1234));
        assert_eq!(f.items[1].image.as_deref(), Some("https://example.org/ep1.jpg"));
        // 04:00 at +06:00 is 22:00 UTC the day before.
        assert_eq!(f.items[0].published, Some(days_from_civil(2003, 6, 9) * 86_400 + 22 * 3600));
    }

    #[test]
    fn parses_atom_and_dates() {
        let atom = r#"<feed xmlns="http://www.w3.org/2005/Atom"><title>T</title><author><name>A</name></author>
            <entry><title>E1</title><link rel="enclosure" href="https://x/e1.mp3" type="audio/mpeg"/><published>2024-03-01T10:00:00Z</published></entry></feed>"#;
        let f = parse(atom).unwrap();
        assert_eq!(f.author.as_deref(), Some("A"));
        assert_eq!(f.items[0].published, Some(days_from_civil(2024, 3, 1) * 86_400 + 36_000));
        assert_eq!(parse_date("2024-03-01T10:00:00+06:00"), Some(days_from_civil(2024, 3, 1) * 86_400 + 36_000 - 21_600));
        assert_eq!(parse_duration("45"), Some(45));
        assert_eq!(parse_duration("abc"), None);
        assert!(parse("<html></html>").is_err());
    }
}
