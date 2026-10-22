//! The library view of the index: files grouped into lectures, series and
//! Quran recitations ("titles"), plus the queries behind the app's pages.
//!
//! Grouping runs after every sync of a source. It is a pure function of the
//! files, images and source details on record, so it can always be re-run,
//! and titles are keyed by (source, key) so a re-sync updates them in place.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::Result;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde::Serialize;
use url::Url;

use crate::config::{Root, RootKind, SourceKind};
use crate::crawler::percent_decode;
use crate::index::{fts_query, unix_now, Index};
use crate::naming;
use crate::quran::{self, Surah};
use crate::topics;

// ------------------------------------------------------------- grouping ---

struct FileInfo {
    id: i64,
    url: String,
    name: String,
    dir_url: String,
    size: Option<i64>,
    added_at: i64,
    duration: Option<i64>,
    label: Option<String>,
    published_at: Option<i64>,
    thumb: Option<String>,
    description: Option<String>,
    speaker: Option<String>,
    position: Option<i64>,
}

/// What one title looks like before it is written.
#[derive(Default)]
struct Group {
    key: String,
    kind: &'static str,
    name: String,
    year: Option<u32>,
    speaker: Option<String>,
    description: Option<String>,
    poster: Option<String>,
    dir_url: String,
    folder: Option<String>,
    /// (file index, season, episode), in play order.
    items: Vec<(usize, Option<u32>, Option<u32>)>,
}

/// Source-level details from YouTube or a feed.
#[derive(Default, Clone)]
struct SourceMeta {
    title: Option<String>,
    image: Option<String>,
    author: Option<String>,
    description: Option<String>,
    link: Option<String>,
}

/// Rebuilds the titles of one source from its files. Returns the number of
/// titles it now has.
pub fn rebuild_titles(index: &mut Index, root: &Root) -> Result<usize> {
    let files = drop_alternate_formats(load_files(index.conn(), &root.name)?);
    let images = load_images(index.conn(), &root.name)?;
    let meta = load_meta(index.conn(), &root.name)?;

    let mut groups = match root.kind()? {
        RootKind::Http(_) | RootKind::Local(_) => {
            let base = root_prefix(root)?;
            match root.kind {
                SourceKind::Lectures => group_lectures(&files, &base, root.speaker_folders),
                SourceKind::Series => group_series(&files, &base, root.speaker_folders),
                SourceKind::Quran => group_quran(&files, &base, root),
            }
        }
        RootKind::YouTube(_) | RootKind::Feed(_) => group_external(&files, root, &meta),
    };
    groups.retain(|g| !g.items.is_empty());

    let now = unix_now();
    let tx = index.conn_mut().transaction()?;
    let mut keep = Vec::with_capacity(groups.len());
    let mut fts_rows: Vec<(i64, String, String, String)> = Vec::with_capacity(groups.len());
    {
        let mut upsert = tx.prepare_cached(
            "INSERT INTO titles (root, key, kind, name, year, speaker, topics, description, language,
                 media, folder, dir_url, poster, file_count, total_duration, total_size,
                 added_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
             ON CONFLICT(root, key) DO UPDATE SET
                 kind = excluded.kind, name = excluded.name, year = excluded.year,
                 speaker = excluded.speaker, topics = excluded.topics,
                 description = excluded.description, language = excluded.language,
                 media = excluded.media, folder = excluded.folder, dir_url = excluded.dir_url,
                 poster = excluded.poster, file_count = excluded.file_count,
                 total_duration = excluded.total_duration, total_size = excluded.total_size,
                 updated_at = excluded.updated_at
             RETURNING id",
        )?;
        let mut assign = tx.prepare_cached(
            "UPDATE files SET title_id = ?1, season = ?2, episode = ?3 WHERE id = ?4",
        )?;
        let youtube = root.youtube.is_some();

        for g in &groups {
            // Speaker: set on the source, else found in folders or the feed.
            let speaker = root
                .speaker
                .clone()
                .or_else(|| g.speaker.clone())
                .or_else(|| meta.author.clone());
            let labels: Vec<String> = g
                .items
                .iter()
                .take(80)
                .map(|&(i, _, _)| files[i].label.clone().unwrap_or_else(|| naming::clean_label(&files[i].name)))
                .collect();
            let folder_name = folder_name_of(&g.dir_url);
            let text = format!(
                "{} {} {} {}",
                g.name,
                g.description.as_deref().unwrap_or(""),
                folder_name,
                labels.join(" ")
            );
            let topic_list = if g.kind == "quran" {
                vec!["Quran"]
            } else {
                topics::detect(&text)
            };
            // From the talk's own words only: speaker and folder names are
            // often in Latin letters even for Bangla lectures.
            let language = naming::language(&format!("{} {}", g.name, labels.join(" ")));
            let media = if youtube {
                "youtube"
            } else if g.items.iter().any(|&(i, _, _)| is_video(&files[i].name)) {
                "video"
            } else {
                "audio"
            };
            let poster = g
                .poster
                .clone()
                .or_else(|| pick_poster(&images, &g.dir_url, g.items.first().map(|&(i, _, _)| &files[i].dir_url)))
                .or_else(|| meta.image.clone());
            let durations: Vec<i64> = g.items.iter().filter_map(|&(i, _, _)| files[i].duration).collect();
            let total_duration = (!durations.is_empty()).then(|| durations.iter().sum::<i64>());
            let sizes: Vec<i64> = g.items.iter().filter_map(|&(i, _, _)| files[i].size).collect();
            let total_size = (!sizes.is_empty()).then(|| sizes.iter().sum::<i64>());
            let added = g.items.iter().map(|&(i, _, _)| files[i].published_at.unwrap_or(files[i].added_at)).min().unwrap_or(now);
            let updated = g.items.iter().map(|&(i, _, _)| files[i].published_at.unwrap_or(files[i].added_at)).max().unwrap_or(now);
            let description = g.description.clone().filter(|d| !d.trim().is_empty());

            let id: i64 = upsert.query_row(
                params![
                    root.name,
                    g.key,
                    g.kind,
                    g.name,
                    g.year,
                    speaker,
                    topics::to_column(&topic_list),
                    description,
                    language,
                    media,
                    g.folder,
                    g.dir_url,
                    poster,
                    g.items.len() as i64,
                    total_duration,
                    total_size,
                    added,
                    updated
                ],
                |r| r.get(0),
            )?;
            keep.push(id);
            for &(i, season, episode) in &g.items {
                assign.execute(params![id, season, episode, files[i].id])?;
            }
            // Searchable text: the description and episode names, so a
            // search for a subject finds the series that covers it.
            let extra = format!("{} {}", description.as_deref().unwrap_or(""), labels.join(" "));
            fts_rows.push((id, g.name.clone(), speaker.unwrap_or_default(), extra.chars().take(4000).collect::<String>()));
        }
    }

    // Drop this source's search entries first (stale titles included), then
    // re-add them for the titles that remain.
    tx.execute(
        "DELETE FROM titles_fts WHERE rowid IN (SELECT id FROM titles WHERE root = ?1)",
        [&root.name],
    )?;

    // Titles whose files are all gone, and everything hanging off them.
    let ids_json = serde_json::to_string(&keep)?;
    let stale = "SELECT id FROM titles WHERE root = ?1 AND id NOT IN (SELECT value FROM json_each(?2))";
    for table in ["watchlist", "plays"] {
        tx.execute(
            &format!("DELETE FROM {table} WHERE title_id IN ({stale})"),
            params![root.name, ids_json],
        )?;
    }
    tx.execute(
        "DELETE FROM titles WHERE root = ?1 AND id NOT IN (SELECT value FROM json_each(?2))",
        params![root.name, ids_json],
    )?;

    // Search index for this source's titles, rebuilt wholesale. The
    // speaker and episode names are searchable too.
    {
        let mut insert = tx.prepare_cached("INSERT INTO titles_fts (rowid, name, speaker, extra) VALUES (?1, ?2, ?3, ?4)")?;
        for (id, name, speaker, extra) in &fts_rows {
            insert.execute(params![id, name, speaker, extra])?;
        }
    }

    // Speaker pictures from YouTube channels and feeds.
    if let (Some(author), Some(image)) = (&meta.author, &meta.image) {
        tx.execute(
            "INSERT INTO speakers (name, image) VALUES (?1, ?2)
             ON CONFLICT(name) DO UPDATE SET image = excluded.image",
            params![author, image],
        )?;
    }
    tx.commit()?;
    Ok(keep.len())
}

/// The same talk is often published in several formats side by side
/// ("Day1.mp3" and "Day1.ogg" on the Internet Archive). Keep one per
/// folder and name, preferring the most widely playable format.
fn drop_alternate_formats(files: Vec<FileInfo>) -> Vec<FileInfo> {
    fn rank(name: &str) -> u8 {
        match name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).as_deref() {
            Some("mp3") => 0,
            Some("m4a") | Some("mp4") => 1,
            Some("aac") | Some("webm") => 2,
            Some("ogg") | Some("opus") => 3,
            _ => 4,
        }
    }
    let stem = |f: &FileInfo| -> String {
        let n = f.name.rsplit_once('.').map_or(f.name.as_str(), |(s, _)| s);
        format!("{}#{}", f.dir_url, n.to_lowercase())
    };
    let mut best: HashMap<String, (u8, usize)> = HashMap::new();
    for (i, f) in files.iter().enumerate() {
        let r = rank(&f.name);
        let e = best.entry(stem(f)).or_insert((r, i));
        if r < e.0 {
            *e = (r, i);
        }
    }
    let keep: std::collections::HashSet<usize> = best.values().map(|&(_, i)| i).collect();
    files.into_iter().enumerate().filter(|(i, _)| keep.contains(i)).map(|(_, f)| f).collect()
}

fn is_video(name: &str) -> bool {
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp4" | "m4v" | "mkv" | "webm" | "avi" | "mov" | "wmv" | "flv" | "3gp" | "ts" | "mpg" | "mpeg"
        )
    })
}

/// The decoded path prefix that folder URLs of this root start with.
fn root_prefix(root: &Root) -> Result<String> {
    Ok(match root.kind()? {
        RootKind::Http(url) => percent_decode(url.path()),
        RootKind::Local(path) => {
            let mut s = path.to_string_lossy().into_owned();
            if !s.ends_with(['\\', '/']) {
                s.push(std::path::MAIN_SEPARATOR);
            }
            s
        }
        RootKind::YouTube(_) | RootKind::Feed(_) => String::new(),
    })
}

fn load_files(conn: &Connection, root: &str) -> Result<Vec<FileInfo>> {
    let mut stmt = conn.prepare(
        "SELECT id, url, name, size, added_at, duration, label, published_at, thumb,
                description, speaker, position
         FROM files WHERE root = ?1 ORDER BY url",
    )?;
    let rows = stmt.query_map([root], |r| {
        let url: String = r.get(1)?;
        Ok(FileInfo {
            id: r.get(0)?,
            dir_url: parent_url(&url),
            url,
            name: r.get(2)?,
            size: r.get(3)?,
            added_at: r.get(4)?,
            duration: r.get(5)?,
            label: r.get(6)?,
            published_at: r.get(7)?,
            thumb: r.get(8)?,
            description: r.get(9)?,
            speaker: r.get(10)?,
            position: r.get(11)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

fn load_images(conn: &Connection, root: &str) -> Result<HashMap<String, Vec<(String, String)>>> {
    let mut stmt = conn.prepare("SELECT dir, url, name FROM images WHERE root = ?1")?;
    let mut map: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let rows = stmt.query_map([root], |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?)))?;
    for row in rows {
        let (dir, url, name) = row?;
        map.entry(dir).or_default().push((url, name));
    }
    Ok(map)
}

fn load_meta(conn: &Connection, root: &str) -> Result<SourceMeta> {
    Ok(conn
        .query_row(
            "SELECT title, image, author, description, link FROM source_meta WHERE root = ?1",
            [root],
            |r| {
                Ok(SourceMeta {
                    title: r.get(0)?,
                    image: r.get(1)?,
                    author: r.get(2)?,
                    description: r.get(3)?,
                    link: r.get(4)?,
                })
            },
        )
        .optional()?
        .unwrap_or_default())
}

/// "http://h/a/b/x.mp3" -> "http://h/a/b/".
fn parent_url(url: &str) -> String {
    match url.rfind('/') {
        Some(i) => url[..=i].to_string(),
        None => url.to_string(),
    }
}

/// Decoded last segment of a folder URL.
fn folder_name_of(dir_url: &str) -> String {
    let trimmed = dir_url.trim_end_matches('/');
    percent_decode(trimmed.rsplit('/').next().unwrap_or(""))
}

/// Folder segments between the source root and this folder, decoded.
fn rel_segments(dir_url: &str, base: &str) -> Vec<String> {
    let path = match Url::parse(dir_url) {
        Ok(u) if u.scheme() == "file" => u
            .to_file_path()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        Ok(u) => percent_decode(u.path()),
        Err(_) => return Vec::new(),
    };
    let rest = strip_prefix_ci(&path, base).unwrap_or("");
    rest.split(['/', '\\'])
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Windows paths compare case-insensitively.
fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len()
        && s.is_char_boundary(prefix.len())
        && s[..prefix.len()].eq_ignore_ascii_case(prefix)
    {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

/// Parent folder URL of a folder URL.
fn up_one(dir_url: &str) -> String {
    let trimmed = dir_url.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(i) => trimmed[..=i].to_string(),
        None => dir_url.to_string(),
    }
}

/// With "folders are speaker names", the first folder under the source is
/// the speaker; the rest of the path is what the grouping looks at.
fn split_speaker(rel: Vec<String>, speaker_folders: bool) -> (Option<String>, Vec<String>) {
    if speaker_folders && !rel.is_empty() {
        let mut rel = rel;
        let speaker = rel.remove(0);
        (Some(speaker), rel)
    } else {
        (None, rel)
    }
}

/// "Patience Part 2" -> "Patience": the shared name of a multi-part talk.
fn base_label(label: &str) -> String {
    let lowered = naming::ascii_digits(label);
    let re_free = lowered
        .split(|c: char| c == '-' || c == '–' || c == '(' || c == ')')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let words: Vec<&str> = re_free.split_whitespace().collect();
    // Drop a trailing "part N" / "পর্ব N" / bare number.
    let mut end = words.len();
    if end > 0 && words[end - 1].chars().all(|c| c.is_ascii_digit()) {
        end -= 1;
        if end > 0 {
            let w = words[end - 1].to_lowercase();
            if matches!(w.as_str(), "part" | "pt" | "episode" | "ep" | "পর্ব" | "খণ্ড" | "পার্ট" | "lecture" | "day") {
                end -= 1;
            }
        }
    }
    words[..end].join(" ")
}

/// Lectures from folders. A folder holding one talk (maybe in parts, plus a
/// picture) is that talk; a folder of many talks gives one title per talk.
/// Files named "Topic Part 1", "Topic Part 2" become one talk in parts.
fn group_lectures(files: &[FileInfo], base: &str, speaker_folders: bool) -> Vec<Group> {
    let mut by_dir: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, f) in files.iter().enumerate() {
        by_dir.entry(f.dir_url.as_str()).or_default().push(i);
    }
    let mut groups: Vec<Group> = Vec::new();
    let mut index_of: HashMap<String, usize> = HashMap::new();

    for (dir, members) in by_dir {
        let (speaker, rel) = split_speaker(rel_segments(dir, base), speaker_folders);
        let folder = rel.first().cloned();
        let labels: Vec<String> = members.iter().map(|&i| naming::clean_label(&files[i].name)).collect();
        let bases: BTreeSet<String> = labels.iter().map(|l| naming::title_key(&base_label(l))).collect();
        let folder_title = rel.last().map(|f| naming::clean_label(f));

        // One talk in this folder (possibly in parts): the folder names it,
        // unless the folder is just a year or a speaker's name.
        let single = !rel.is_empty() && bases.len() == 1 && members.len() <= 12;
        if single {
            let name = folder_title
                .filter(|t| t.chars().any(char::is_alphabetic))
                .unwrap_or_else(|| base_label(&labels[0]));
            let key = format!("d:{dir}");
            let at = *index_of.entry(key.clone()).or_insert_with(|| {
                groups.push(Group {
                    key,
                    kind: "lecture",
                    name,
                    year: naming::parse(&folder_name_of(dir)).year,
                    speaker: speaker.clone(),
                    dir_url: dir.to_string(),
                    folder: if rel.len() > 1 { folder.clone() } else { None },
                    ..Group::default()
                });
                groups.len() - 1
            });
            let mut parts: Vec<(usize, Option<u32>)> = members.iter().map(|&i| (i, naming::episode_number(&files[i].name))).collect();
            parts.sort_by_key(|(i, n)| (n.unwrap_or(u32::MAX), files[*i].name.clone()));
            for (i, n) in parts {
                groups[at].items.push((i, None, n));
            }
            continue;
        }
        for (&i, label) in members.iter().zip(&labels) {
            let base_name = base_label(label);
            let key = format!("f:{dir}#{}", naming::title_key(&base_name));
            let at = *index_of.entry(key.clone()).or_insert_with(|| {
                groups.push(Group {
                    key,
                    kind: "lecture",
                    name: if base_name.is_empty() { label.clone() } else { base_name.clone() },
                    year: naming::parse(&files[i].name).year,
                    speaker: speaker.clone(),
                    dir_url: dir.to_string(),
                    folder: folder.clone(),
                    ..Group::default()
                });
                groups.len() - 1
            });
            groups[at].items.push((i, None, naming::episode_number(&files[i].name)));
        }
    }
    for g in &mut groups {
        g.items.sort_by_key(|&(i, _, n)| (n.unwrap_or(u32::MAX), files[i].name.clone()));
    }
    groups
}

/// A "Part 2" / "Volume 3" / "খণ্ড ২" folder inside a series.
fn volume_folder(name: &str) -> Option<u32> {
    naming::season_folder(name).or_else(|| {
        let n = naming::ascii_digits(name).to_lowercase();
        let n = n.trim();
        for prefix in ["volume", "vol", "part", "disc", "cd", "খণ্ড", "পর্ব"] {
            if let Some(rest) = n.strip_prefix(prefix) {
                let digits: String = rest.trim_start_matches([' ', '.', '-', '_']).chars().take_while(char::is_ascii_digit).collect();
                if !digits.is_empty() && rest.trim_start_matches([' ', '.', '-', '_']).len() == digits.len() {
                    return digits.parse().ok();
                }
            }
        }
        None
    })
}

/// Series from folders: each series folder (walking up past "Part 2" /
/// "Volume 3" folders) is one title, its files the episodes.
fn group_series(files: &[FileInfo], base: &str, speaker_folders: bool) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut index_of: HashMap<String, usize> = HashMap::new();
    for (i, f) in files.iter().enumerate() {
        let mut series_dir = f.dir_url.clone();
        let (speaker, mut rel) = split_speaker(rel_segments(&series_dir, base), speaker_folders);
        let mut volume = None;
        while rel.len() > 1 {
            match rel.last().and_then(|n| volume_folder(n)) {
                Some(v) => {
                    volume.get_or_insert(v);
                    series_dir = up_one(&series_dir);
                    rel.pop();
                }
                None => break,
            }
        }
        let label = naming::clean_label(&f.name);
        // Episodes loose in the source (or speaker) folder: group by name.
        let (key, name) = if rel.is_empty() {
            let b = base_label(&label);
            (format!("t:{}#{}", speaker.as_deref().unwrap_or(""), naming::title_key(&b)), b)
        } else {
            (format!("d:{series_dir}"), naming::clean_label(rel.last().unwrap()))
        };
        let at = *index_of.entry(key.clone()).or_insert_with(|| {
            groups.push(Group {
                key,
                kind: "series",
                name,
                year: naming::parse(&folder_name_of(&series_dir)).year,
                speaker: speaker.clone(),
                dir_url: series_dir.clone(),
                folder: if rel.len() > 1 { rel.first().cloned() } else { None },
                ..Group::default()
            });
            groups.len() - 1
        });
        groups[at].items.push((i, volume, naming::episode_number(&f.name)));
    }
    for g in &mut groups {
        g.items.sort_by_key(|&(i, s, e)| (s.unwrap_or(0), e.unwrap_or(u32::MAX), files[i].name.clone()));
        // Number the unnumbered ones by their order, so every episode has one.
        let all_unnumbered = g.items.iter().all(|it| it.2.is_none());
        if all_unnumbered {
            for (n, item) in g.items.iter_mut().enumerate() {
                item.2 = Some(n as u32 + 1);
            }
        }
    }
    groups
}

/// Quran recitations: each folder of surah files is one reciter's
/// recitation, named by the folder (the reciter) and ordered by surah.
fn group_quran(files: &[FileInfo], base: &str, root: &Root) -> Vec<Group> {
    let mut by_dir: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
    for (i, f) in files.iter().enumerate() {
        by_dir.entry(f.dir_url.as_str()).or_default().push(i);
    }
    let mut groups = Vec::new();
    for (dir, members) in by_dir {
        let rel = rel_segments(dir, base);
        // Quality folders ("128kbps") name nothing; use the one above.
        let named = rel
            .iter()
            .rev()
            .find(|s| !s.to_lowercase().contains("kbps") && s.chars().any(char::is_alphabetic))
            .cloned();
        let reciter = named.clone().unwrap_or_else(|| root.speaker.clone().unwrap_or_else(|| root.name.clone()));
        let mut items: Vec<(usize, Option<u32>, Option<u32>)> =
            members.iter().map(|&i| (i, None, naming::surah_number(&files[i].name))).collect();
        items.sort_by_key(|&(i, _, n)| (n.unwrap_or(u32::MAX), files[i].name.clone()));
        groups.push(Group {
            key: format!("q:{dir}"),
            kind: "quran",
            name: naming::clean_label(&reciter),
            speaker: Some(naming::clean_label(&reciter)),
            dir_url: dir.to_string(),
            folder: if rel.len() > 1 { rel.first().cloned() } else { None },
            items,
            ..Group::default()
        });
    }
    groups
}

/// YouTube and podcast sources. As *lectures*, every video or episode is
/// its own title; as a *series* (or Quran), the whole source is one title
/// with its videos / episodes in order.
fn group_external(files: &[FileInfo], root: &Root, meta: &SourceMeta) -> Vec<Group> {
    if root.kind == SourceKind::Lectures {
        return files
            .iter()
            .enumerate()
            .map(|(i, f)| Group {
                key: format!("v:{}", f.url),
                kind: "lecture",
                name: f.label.clone().unwrap_or_else(|| naming::clean_label(&f.name)),
                year: f.published_at.map(year_of),
                speaker: f.speaker.clone(),
                description: f.description.clone(),
                poster: f.thumb.clone(),
                dir_url: f.url.clone(),
                items: vec![(i, None, None)],
                ..Group::default()
            })
            .collect();
    }
    // Playlists play in playlist order; feeds oldest first, so episode 1 is
    // the first one published.
    let mut order: Vec<usize> = (0..files.len()).collect();
    order.sort_by_key(|&i| (files[i].position.unwrap_or(i64::MAX), files[i].published_at.unwrap_or(0), i));
    let items = order
        .into_iter()
        .enumerate()
        .map(|(n, i)| (i, None, Some(n as u32 + 1)))
        .collect();
    vec![Group {
        key: "src".into(),
        kind: if root.kind == SourceKind::Quran { "quran" } else { "series" },
        name: meta.title.clone().unwrap_or_else(|| root.name.clone()),
        year: files.iter().filter_map(|f| f.published_at).min().map(year_of),
        speaker: meta.author.clone().or_else(|| files.iter().find_map(|f| f.speaker.clone())),
        description: meta.description.clone(),
        poster: meta.image.clone().or_else(|| files.iter().find_map(|f| f.thumb.clone())),
        dir_url: meta.link.clone().unwrap_or_else(|| root.location().to_string()),
        items,
        ..Group::default()
    }]
}

fn year_of(ts: i64) -> u32 {
    // Days since 1970 -> civil year (Howard Hinnant's algorithm, year part).
    let z = ts.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let year = yoe + era * 400 + if mp >= 10 { 1 } else { 0 };
    year as u32
}

/// The poster for a title: an image in its folder (or the files' folder),
/// preferring names that say what they are.
fn pick_poster(
    images: &HashMap<String, Vec<(String, String)>>,
    dir_url: &str,
    file_dir: Option<&String>,
) -> Option<String> {
    let candidates: Vec<&(String, String)> = images
        .get(dir_url)
        .or_else(|| file_dir.and_then(|d| images.get(d)))?
        .iter()
        // Machine-made pictures (the Internet Archive's spectrograms) aren't
        // artwork.
        .filter(|(_, name)| !name.to_lowercase().contains("spectrogram"))
        .collect();
    let score = |name: &str| {
        let n = name.to_lowercase();
        if n.contains("poster") || n.contains("cover") || n.starts_with("folder") || n.contains("thumb") {
            0
        } else if n.contains("banner") || n.contains("backdrop") || n.contains("fanart") {
            2
        } else {
            1
        }
    };
    candidates
        .into_iter()
        .min_by_key(|(_, name)| score(name))
        .map(|(url, _)| url.clone())
}

// ---------------------------------------------------------------- views ---

/// A card in a grid or row.
#[derive(Debug, Clone, Serialize)]
pub struct Card {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub year: Option<u32>,
    pub speaker: Option<String>,
    pub topics: Vec<String>,
    pub poster: Option<String>,
    pub media: Option<String>,
    pub language: Option<String>,
    pub file_count: i64,
    pub total_duration: Option<i64>,
    pub root: String,
    pub saved: bool,
    /// How far into it the listener is, 0..1, when started.
    pub progress: Option<f64>,
}

const CARD_COLUMNS: &str = "t.id, t.kind, t.name, t.year, t.speaker, t.topics, t.poster, t.media,
    t.language, t.file_count, t.total_duration, t.root,
    EXISTS(SELECT 1 FROM watchlist w WHERE w.title_id = t.id) AS saved,
    (SELECT CASE WHEN t.file_count = 1 THEN MIN(1.0, p.position / NULLIF(p.duration, 0))
                 ELSE (SELECT COUNT(*) FROM watched w2 JOIN files f2 ON f2.id = w2.file_id WHERE f2.title_id = t.id) * 1.0 / t.file_count END
     FROM progress p JOIN files f ON f.id = p.file_id WHERE f.title_id = t.id
     ORDER BY p.updated_at DESC LIMIT 1) AS progress";

fn card_from_row(r: &Row) -> rusqlite::Result<Card> {
    let id: i64 = r.get(0)?;
    let poster: Option<String> = r.get(6)?;
    let topics: Option<String> = r.get(5)?;
    Ok(Card {
        id,
        kind: r.get(1)?,
        name: r.get(2)?,
        year: r.get(3)?,
        speaker: r.get(4)?,
        topics: split_list(topics.as_deref()),
        poster: poster_url(id, poster.as_deref()),
        media: r.get(7)?,
        language: r.get(8)?,
        file_count: r.get(9)?,
        total_duration: r.get(10)?,
        root: r.get(11)?,
        saved: r.get(12)?,
        progress: r.get(13)?,
    })
}

fn split_list(v: Option<&str>) -> Vec<String> {
    v.unwrap_or("").split(',').filter(|s| !s.is_empty()).map(String::from).collect()
}

/// Where the UI loads a poster from: images on a server or YouTube are
/// used directly; local ones go through the app's own /poster route.
pub fn poster_url(id: i64, poster: Option<&str>) -> Option<String> {
    match poster {
        Some(p) if p.starts_with("file:") => Some(format!("/poster/{id}")),
        Some(p) => Some(p.to_string()),
        None => None,
    }
}

/// Filters and sort for the grid pages.
#[derive(Debug, Default, serde::Deserialize)]
pub struct Query {
    pub kind: Option<String>,
    pub root: Option<String>,
    pub speaker: Option<String>,
    pub topic: Option<String>,
    pub language: Option<String>,
    pub media: Option<String>,
    pub length: Option<String>,
    pub year: Option<String>,
    pub folder: Option<String>,
    pub q: Option<String>,
    pub watchlist: Option<bool>,
    pub sort: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct Page {
    pub total: i64,
    pub items: Vec<Card>,
}

fn non_empty(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty() && *s != "any")
}

/// Builds the WHERE clause shared by grid queries and facet counts. `skip`
/// leaves one filter out, so a dropdown's options aren't narrowed by its own
/// current choice.
fn filter_sql(q: &Query, skip: Option<&str>) -> (String, Vec<rusqlite::types::Value>) {
    use rusqlite::types::Value;
    let mut sql = String::from(" WHERE 1=1");
    let mut args: Vec<Value> = Vec::new();
    let text = |v: &str| Value::Text(v.to_string());
    let mut add = |clause: &str, value: Option<Value>| {
        sql.push_str(clause);
        args.extend(value);
    };
    let active = |name: &str| skip != Some(name);

    if let Some(k) = non_empty(&q.kind) {
        add(" AND t.kind = ?", Some(text(k)));
    }
    if let Some(r) = non_empty(&q.root) {
        add(" AND t.root = ?", Some(text(r)));
    }
    if let Some(s) = non_empty(&q.speaker).filter(|_| active("speaker")) {
        add(" AND t.speaker = ?", Some(text(s)));
    }
    if let Some(t) = non_empty(&q.topic).filter(|_| active("topic")) {
        add(" AND t.topics LIKE ?", Some(Value::Text(format!("%,{t},%"))));
    }
    if let Some(l) = non_empty(&q.language).filter(|_| active("language")) {
        add(" AND t.language = ?", Some(text(l)));
    }
    if let Some(m) = non_empty(&q.media).filter(|_| active("media")) {
        add(" AND t.media = ?", Some(text(m)));
    }
    match non_empty(&q.length).filter(|_| active("length")) {
        Some("short") => add(" AND t.total_duration < 1200", None),
        Some("medium") => add(" AND t.total_duration BETWEEN 1200 AND 3600", None),
        Some("long") => add(" AND t.total_duration > 3600", None),
        _ => {}
    }
    if let Some(y) = non_empty(&q.year).filter(|_| active("year")).and_then(|y| y.parse::<i64>().ok()) {
        add(" AND t.year = ?", Some(Value::Integer(y)));
    }
    if let Some(v) = non_empty(&q.folder).filter(|_| active("folder")) {
        add(" AND t.folder = ?", Some(text(v)));
    }
    if q.watchlist == Some(true) {
        add(" AND t.id IN (SELECT title_id FROM watchlist)", None);
    }
    if let Some(m) = non_empty(&q.q).and_then(fts_query) {
        add(
            " AND t.id IN (SELECT rowid FROM titles_fts WHERE titles_fts MATCH ?)",
            Some(Value::Text(m)),
        );
    }
    (sql, args)
}

pub fn titles(conn: &Connection, q: &Query) -> Result<Page> {
    let (filter, args) = filter_sql(q, None);
    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM titles t{filter}"),
        params_from_iter(args.iter()),
        |r| r.get(0),
    )?;
    let order = match q.sort.as_deref().unwrap_or("added") {
        "updated" => "t.updated_at DESC, t.id DESC",
        "title" => "t.name COLLATE NOCASE ASC",
        "oldest" => "t.added_at ASC, t.id ASC",
        "longest" => "t.total_duration IS NULL, t.total_duration DESC",
        "shortest" => "t.total_duration IS NULL, t.total_duration ASC",
        "watchlist" => "(SELECT added_at FROM watchlist w WHERE w.title_id = t.id) DESC",
        _ => "t.added_at DESC, t.id DESC",
    };
    let limit = q.limit.unwrap_or(60).clamp(1, 500);
    let sql = format!(
        "SELECT {CARD_COLUMNS} FROM titles t{filter} ORDER BY {order} LIMIT {limit} OFFSET {}",
        q.offset.unwrap_or(0)
    );
    let mut stmt = conn.prepare(&sql)?;
    let items = stmt
        .query_map(params_from_iter(args.iter()), card_from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Page { total, items })
}

#[derive(Serialize)]
pub struct Facet {
    pub value: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct Facets {
    pub speakers: Vec<Facet>,
    pub topics: Vec<Facet>,
    pub languages: Vec<Facet>,
    pub media: Vec<Facet>,
    pub years: Vec<Facet>,
    pub folders: Vec<Facet>,
}

/// Options for each filter dropdown, counted under the other active filters.
pub fn facets(conn: &Connection, q: &Query) -> Result<Facets> {
    let simple = |column: &str, skip: &str, order: &str, limit: usize| -> Result<Vec<Facet>> {
        let (filter, args) = filter_sql(q, Some(skip));
        let sql = format!(
            "SELECT {column} AS v, COUNT(*) FROM titles t{filter} AND {column} IS NOT NULL AND {column} != ''
             GROUP BY v ORDER BY {order} LIMIT {limit}"
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(args.iter()), |r| {
            let v: rusqlite::types::Value = r.get(0)?;
            let value = match v {
                rusqlite::types::Value::Integer(i) => i.to_string(),
                rusqlite::types::Value::Text(s) => s,
                _ => String::new(),
            };
            Ok(Facet { value, count: r.get(1)? })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    };

    // Topics are stored as ",A,B," lists, so split them in Rust.
    let (filter, args) = filter_sql(q, Some("topic"));
    let mut stmt = conn.prepare(&format!(
        "SELECT t.topics FROM titles t{filter} AND t.topics IS NOT NULL AND t.topics != ''"
    ))?;
    let mut counts: BTreeMap<String, i64> = BTreeMap::new();
    for row in stmt.query_map(params_from_iter(args.iter()), |r| r.get::<_, String>(0))? {
        for t in row?.split(',').filter(|t| !t.is_empty()) {
            *counts.entry(t.to_string()).or_default() += 1;
        }
    }
    let mut topics: Vec<Facet> = counts.into_iter().map(|(value, count)| Facet { value, count }).collect();
    topics.sort_by(|a, b| b.count.cmp(&a.count));

    Ok(Facets {
        speakers: simple("t.speaker", "speaker", "COUNT(*) DESC, v", 400)?,
        topics,
        languages: simple("t.language", "language", "COUNT(*) DESC", 20)?,
        media: simple("t.media", "media", "COUNT(*) DESC", 5)?,
        years: simple("t.year", "year", "v DESC", 100)?,
        folders: simple("t.folder", "folder", "v COLLATE NOCASE", 300)?,
    })
}

// --------------------------------------------------------------- detail ---

#[derive(Serialize)]
pub struct Detail {
    #[serde(flatten)]
    pub card: Card,
    pub description: Option<String>,
    pub dir_url: String,
    pub folder: Option<String>,
    pub total_size: Option<i64>,
    pub speaker_image: Option<String>,
    pub items: Vec<ItemView>,
    pub seasons: Vec<SeasonView>,
    pub next_up: Option<i64>,
}

/// One playable file: a lecture part, an episode or a surah.
#[derive(Serialize, Clone)]
pub struct ItemView {
    pub file_id: i64,
    pub url: String,
    pub name: String,
    pub label: String,
    pub season: Option<u32>,
    pub number: Option<u32>,
    pub size: Option<i64>,
    pub duration: Option<i64>,
    pub published_at: Option<i64>,
    pub thumb: Option<String>,
    pub description: Option<String>,
    pub watched: bool,
    pub played_at: Option<i64>,
    pub position: Option<f64>,
    pub surah: Option<&'static Surah>,
}

#[derive(Serialize)]
pub struct SeasonView {
    pub season: u32,
    pub items: Vec<ItemView>,
    pub duration: i64,
}

pub fn detail(conn: &Connection, id: i64) -> Result<Option<Detail>> {
    let card = conn
        .query_row(&format!("SELECT {CARD_COLUMNS} FROM titles t WHERE t.id = ?1"), [id], card_from_row)
        .optional()?;
    let Some(card) = card else {
        return Ok(None);
    };
    let (description, dir_url, folder, total_size): (Option<String>, String, Option<String>, Option<i64>) =
        conn.query_row(
            "SELECT description, dir_url, folder, total_size FROM titles WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )?;
    let speaker_image: Option<String> = match &card.speaker {
        Some(s) => conn
            .query_row("SELECT image FROM speakers WHERE name = ?1", [s], |r| r.get(0))
            .optional()?
            .flatten(),
        None => None,
    };

    let is_quran = card.kind == "quran";
    let mut stmt = conn.prepare(
        "SELECT f.id, f.url, f.name, f.label, f.season, f.episode, f.size, f.duration,
                f.published_at, f.thumb, f.description,
                (SELECT 1 FROM watched w WHERE w.file_id = f.id) IS NOT NULL,
                (SELECT played_at FROM plays p WHERE p.file_id = f.id),
                (SELECT position FROM progress g WHERE g.file_id = f.id)
         FROM files f WHERE f.title_id = ?1
         ORDER BY f.season IS NULL, f.season, f.episode IS NULL, f.episode, f.name",
    )?;
    let items: Vec<ItemView> = stmt
        .query_map([id], |r| {
            let name: String = r.get(2)?;
            let label: Option<String> = r.get(3)?;
            let number: Option<u32> = r.get(5)?;
            Ok(ItemView {
                file_id: r.get(0)?,
                url: r.get(1)?,
                label: label.unwrap_or_else(|| naming::clean_label(&name)),
                name,
                season: r.get(4)?,
                number,
                size: r.get(6)?,
                duration: r.get(7)?,
                published_at: r.get(8)?,
                thumb: r.get(9)?,
                description: r.get(10)?,
                watched: r.get(11)?,
                played_at: r.get(12)?,
                position: r.get(13)?,
                surah: if is_quran { number.and_then(quran::surah) } else { None },
            })
        })?
        .collect::<rusqlite::Result<_>>()?;

    let mut seasons: Vec<SeasonView> = Vec::new();
    if items.iter().any(|i| i.season.is_some()) {
        for it in &items {
            let s = it.season.unwrap_or(1);
            if seasons.last().map(|x| x.season) != Some(s) {
                seasons.push(SeasonView { season: s, items: Vec::new(), duration: 0 });
            }
            let last = seasons.last_mut().unwrap();
            last.duration += it.duration.unwrap_or(0);
            last.items.push(it.clone());
        }
    }

    // Up next: an item paused part-way, else the one after the last played,
    // else the first not yet finished.
    let next_up = items
        .iter()
        .filter(|i| i.position.is_some_and(|p| p > 30.0) && !i.watched)
        .max_by_key(|i| i.played_at)
        .map(|i| i.file_id)
        .or_else(|| {
            let last = items
                .iter()
                .enumerate()
                .filter_map(|(n, i)| i.played_at.map(|t| (t, n)))
                .max()
                .map(|(_, n)| n);
            match last {
                Some(n) if n + 1 < items.len() => Some(items[n + 1].file_id),
                _ => None,
            }
        })
        .or_else(|| items.iter().find(|i| !i.watched).map(|i| i.file_id))
        .or_else(|| items.first().map(|i| i.file_id));

    Ok(Some(Detail {
        card,
        description,
        dir_url,
        folder,
        total_size,
        speaker_image,
        items,
        seasons,
        next_up,
    }))
}

// ----------------------------------------------------------------- home ---

#[derive(Serialize)]
pub struct HomeRow {
    pub key: String,
    pub link: Option<String>,
    pub items: Vec<Card>,
}

fn cards(conn: &Connection, where_order: &str, limit: usize) -> Result<Vec<Card>> {
    let mut stmt = conn.prepare(&format!("SELECT {CARD_COLUMNS} FROM titles t {where_order} LIMIT {limit}"))?;
    let items = stmt.query_map([], card_from_row)?.collect::<rusqlite::Result<_>>()?;
    Ok(items)
}

/// Home page rows. `key` names the row; the UI translates it.
pub fn home(conn: &Connection) -> Result<Vec<HomeRow>> {
    let mut rows = Vec::new();
    let mut push = |key: &str, link: Option<&str>, items: Vec<Card>| {
        if !items.is_empty() {
            rows.push(HomeRow { key: key.into(), link: link.map(String::from), items });
        }
    };
    push(
        "continue",
        Some("#/recent"),
        cards(
            conn,
            "JOIN (SELECT f.title_id AS tid, MAX(p.updated_at) AS last FROM progress p
                   JOIN files f ON f.id = p.file_id GROUP BY f.title_id
                   UNION SELECT title_id, MAX(played_at) FROM plays GROUP BY title_id) r
             ON r.tid = t.id GROUP BY t.id ORDER BY MAX(r.last) DESC",
            20,
        )?,
    );
    push("lectures", Some("#/lectures"), cards(conn, "WHERE t.kind = 'lecture' ORDER BY t.added_at DESC, t.id DESC", 20)?);
    push("series", Some("#/series"), cards(conn, "WHERE t.kind = 'series' ORDER BY t.updated_at DESC, t.id DESC", 20)?);
    push("quran", Some("#/quran"), cards(conn, "WHERE t.kind = 'quran' ORDER BY t.name COLLATE NOCASE", 20)?);
    push("saved", Some("#/saved"), cards(conn, "JOIN watchlist w ON w.title_id = t.id ORDER BY w.added_at DESC", 20)?);
    for topic in ["Seerah", "Ramadan", "Family", "Akhirah"] {
        push(
            &format!("topic:{topic}"),
            Some(&format!("#/topic/{topic}")),
            cards(conn, &format!("WHERE t.topics LIKE '%,{topic},%' ORDER BY t.added_at DESC"), 20)?,
        );
    }
    Ok(rows)
}

/// Titles by most recent listening.
pub fn recently_played(conn: &Connection, limit: usize) -> Result<Vec<Card>> {
    cards(
        conn,
        "JOIN (SELECT title_id, MAX(played_at) AS last FROM plays GROUP BY title_id) p
         ON p.title_id = t.id ORDER BY p.last DESC",
        limit,
    )
}

pub fn cards_by_ids(conn: &Connection, ids: &[i64]) -> Result<Vec<Card>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare(&format!("SELECT {CARD_COLUMNS} FROM titles t WHERE t.id = ?1"))?;
    for id in ids {
        if let Some(c) = stmt.query_row([id], card_from_row).optional()? {
            out.push(c);
        }
    }
    Ok(out)
}

#[derive(Serialize)]
pub struct SpeakerView {
    pub name: String,
    pub count: i64,
    pub image: Option<String>,
    pub topics: Vec<String>,
}

/// Every speaker with how many titles they have.
pub fn speakers(conn: &Connection, q: Option<&str>) -> Result<Vec<SpeakerView>> {
    let like = format!("%{}%", q.unwrap_or("").trim());
    let mut stmt = conn.prepare(
        "SELECT t.speaker, COUNT(*), s.image, GROUP_CONCAT(t.topics, '')
         FROM titles t LEFT JOIN speakers s ON s.name = t.speaker
         WHERE t.speaker IS NOT NULL AND t.speaker != '' AND t.speaker LIKE ?1
         GROUP BY t.speaker ORDER BY COUNT(*) DESC, t.speaker LIMIT 1000",
    )?;
    let rows = stmt.query_map([like], |r| {
        let all: Option<String> = r.get(3)?;
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for t in all.unwrap_or_default().split(',').filter(|t| !t.is_empty()) {
            *counts.entry(t.to_string()).or_default() += 1;
        }
        let mut top: Vec<(String, usize)> = counts.into_iter().collect();
        top.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(SpeakerView {
            name: r.get(0)?,
            count: r.get(1)?,
            image: r.get(2)?,
            topics: top.into_iter().take(3).map(|(t, _)| t).collect(),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

#[derive(Serialize, Default)]
pub struct Counts {
    pub lectures: i64,
    pub series: i64,
    pub quran: i64,
    pub speakers: i64,
    pub saved: i64,
    pub recent: i64,
    pub files: i64,
    pub bytes: i64,
    pub seconds: i64,
    pub per_root: HashMap<String, i64>,
}

pub fn counts(conn: &Connection) -> Result<Counts> {
    let one = |sql: &str| -> Result<i64> { Ok(conn.query_row(sql, [], |r| r.get(0))?) };
    let mut per_root = HashMap::new();
    let mut stmt = conn.prepare("SELECT root, COUNT(*) FROM titles GROUP BY root")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))? {
        let (root, n) = row?;
        per_root.insert(root, n);
    }
    Ok(Counts {
        lectures: one("SELECT COUNT(*) FROM titles WHERE kind = 'lecture'")?,
        series: one("SELECT COUNT(*) FROM titles WHERE kind = 'series'")?,
        quran: one("SELECT COUNT(*) FROM titles WHERE kind = 'quran'")?,
        speakers: one("SELECT COUNT(DISTINCT speaker) FROM titles WHERE speaker IS NOT NULL AND speaker != ''")?,
        saved: one("SELECT COUNT(*) FROM watchlist")?,
        recent: one("SELECT COUNT(DISTINCT title_id) FROM plays")?,
        files: one("SELECT COUNT(*) FROM files")?,
        bytes: one("SELECT COALESCE(SUM(size), 0) FROM files")?,
        seconds: one("SELECT COALESCE(SUM(duration), 0) FROM files")?,
        per_root,
    })
}

// -------------------------------------------------------------- actions ---

pub fn set_watchlist(conn: &Connection, title_id: i64, on: bool) -> Result<()> {
    if on {
        conn.execute(
            "INSERT OR IGNORE INTO watchlist (title_id, added_at) VALUES (?1, ?2)",
            params![title_id, unix_now()],
        )?;
    } else {
        conn.execute("DELETE FROM watchlist WHERE title_id = ?1", [title_id])?;
    }
    Ok(())
}

pub fn set_watched(conn: &Connection, file_id: i64, on: bool) -> Result<()> {
    if on {
        conn.execute(
            "INSERT OR REPLACE INTO watched (file_id, at) VALUES (?1, ?2)",
            params![file_id, unix_now()],
        )?;
    } else {
        conn.execute("DELETE FROM watched WHERE file_id = ?1", [file_id])?;
        conn.execute("DELETE FROM progress WHERE file_id = ?1", [file_id])?;
    }
    Ok(())
}

/// Records that a file started playing: it shows in "Recently played" and
/// moves "Up next" along. External players can't report back, so playing
/// there also counts as finished.
pub fn record_play(conn: &Connection, file_id: i64, external: bool) -> Result<()> {
    let now = unix_now();
    conn.execute(
        "INSERT INTO plays (file_id, title_id, played_at)
         SELECT id, title_id, ?2 FROM files WHERE id = ?1 AND title_id IS NOT NULL
         ON CONFLICT(file_id) DO UPDATE SET played_at = excluded.played_at",
        params![file_id, now],
    )?;
    if external {
        set_watched(conn, file_id, true)?;
    }
    Ok(())
}

/// Saves the in-app player's position. Near the end counts as finished.
pub fn save_progress(conn: &Connection, file_id: i64, position: f64, duration: Option<f64>) -> Result<()> {
    let finished = duration.is_some_and(|d| d > 0.0 && position >= d * 0.95 - 5.0);
    if finished {
        conn.execute("DELETE FROM progress WHERE file_id = ?1", [file_id])?;
        return set_watched(conn, file_id, true);
    }
    conn.execute(
        "INSERT INTO progress (file_id, position, duration, updated_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(file_id) DO UPDATE SET position = excluded.position,
             duration = COALESCE(excluded.duration, progress.duration), updated_at = excluded.updated_at",
        params![file_id, position, duration, unix_now()],
    )?;
    // Remember the real length for files the server listed without one.
    if let Some(d) = duration.filter(|d| *d > 0.0) {
        conn.execute(
            "UPDATE files SET duration = ?2 WHERE id = ?1 AND duration IS NULL",
            params![file_id, d.round() as i64],
        )?;
    }
    Ok(())
}

/// (url, name, title_id) of a file.
pub fn file_info(conn: &Connection, file_id: i64) -> Result<Option<(String, String, Option<i64>)>> {
    Ok(conn
        .query_row("SELECT url, name, title_id FROM files WHERE id = ?1", [file_id], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })
        .optional()?)
}

/// Every file of a title in play order, for playlists.
pub fn title_files(conn: &Connection, title_id: i64) -> Result<Vec<(i64, String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, url, COALESCE(label, name) FROM files WHERE title_id = ?1
         ORDER BY season IS NULL, season, episode IS NULL, episode, name",
    )?;
    let rows = stmt.query_map([title_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn title_dir(conn: &Connection, title_id: i64) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT dir_url FROM titles WHERE id = ?1", [title_id], |r| r.get(0))
        .optional()?)
}

pub fn local_poster(conn: &Connection, title_id: i64) -> Result<Option<String>> {
    Ok(conn
        .query_row("SELECT poster FROM titles WHERE id = ?1", [title_id], |r| r.get(0))
        .optional()?
        .flatten())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{FileRow, ImageRow};

    fn file(root: &str, url: &str) -> FileRow {
        let name = percent_decode(url.rsplit('/').next().unwrap());
        FileRow {
            root: root.into(),
            url: url.into(),
            dir: percent_decode(Url::parse(url).unwrap().path()),
            size: Some(1000),
            title: name.clone(),
            name,
            ..FileRow::default()
        }
    }

    fn root(name: &str, url: &str, kind: SourceKind) -> Root {
        Root {
            name: name.into(),
            url: Some(url.into()),
            path: None,
            youtube: None,
            feed: None,
            kind,
            speaker: None,
            speaker_folders: false,
        }
    }

    #[test]
    fn helpers() {
        assert_eq!(base_label("Patience Part 2"), "Patience");
        assert_eq!(base_label("ধৈর্য পর্ব 3"), "ধৈর্য");
        assert_eq!(base_label("Surah Al-Kahf"), "Surah Al Kahf");
        assert_eq!(volume_folder("Volume 2"), Some(2));
        assert_eq!(volume_folder("খণ্ড ৩"), Some(3));
        assert_eq!(volume_folder("Tafsir"), None);
        assert_eq!(year_of(1_700_000_000), 2023);
        assert_eq!(year_of(0), 1970);
    }

    #[test]
    fn lectures_series_and_quran() {
        let mut idx = Index::open(":memory:").unwrap();

        // Lectures with speaker folders; a two-part talk becomes one title.
        let w = "http://h/Waz/";
        let mut lectures = root("Waz", w, SourceKind::Lectures);
        lectures.speaker_folders = true;
        let rows = [
            file("Waz", &format!("{w}Speaker%20One/Patience%20Part%201.mp3")),
            file("Waz", &format!("{w}Speaker%20One/Patience%20Part%202.mp3")),
            file("Waz", &format!("{w}Speaker%20One/The%20Night%20Prayer.mp3")),
            file("Waz", &format!("{w}Speaker%20Two/%E0%A6%B0%E0%A6%AE%E0%A6%9C%E0%A6%BE%E0%A6%A8.mp3")),
        ];
        idx.record_level("Waz", &rows, &[], &[], &[]).unwrap();
        assert_eq!(rebuild_titles(&mut idx, &lectures).unwrap(), 3);
        let page = titles(idx.conn(), &Query { kind: Some("lecture".into()), sort: Some("title".into()), ..Query::default() }).unwrap();
        let got: Vec<_> = page.items.iter().map(|c| (c.name.as_str(), c.speaker.as_deref(), c.file_count)).collect();
        assert!(got.contains(&("Patience", Some("Speaker One"), 2)), "{got:?}");
        assert!(got.contains(&("The Night Prayer", Some("Speaker One"), 1)), "{got:?}");
        let prayer = page.items.iter().find(|c| c.name == "The Night Prayer").unwrap();
        assert_eq!(prayer.topics, vec!["Salah"]);
        let ramadan = page.items.iter().find(|c| c.speaker.as_deref() == Some("Speaker Two")).unwrap();
        assert_eq!(ramadan.language.as_deref(), Some("bn"));
        assert_eq!(ramadan.topics, vec!["Ramadan"]);
        let speakers = speakers(idx.conn(), None).unwrap();
        assert_eq!(speakers[0].name, "Speaker One");
        assert_eq!(speakers[0].count, 2);

        // A series folder with numbered episodes in a volume folder.
        let s = "http://h/Series/";
        let series = root("Series", s, SourceKind::Series);
        let rows = [
            file("Series", &format!("{s}Seerah%20Series/Volume%201/02%20-%20Hijrah.mp3")),
            file("Series", &format!("{s}Seerah%20Series/Volume%201/01%20-%20Birth.mp3")),
        ];
        let images = [ImageRow {
            root: "Series".into(),
            dir: format!("{s}Seerah%20Series/"),
            url: format!("{s}Seerah%20Series/cover.jpg"),
            name: "cover.jpg".into(),
        }];
        idx.record_level("Series", &rows, &images, &[], &[]).unwrap();
        assert_eq!(rebuild_titles(&mut idx, &series).unwrap(), 1);
        let page = titles(idx.conn(), &Query { kind: Some("series".into()), ..Query::default() }).unwrap();
        let c = &page.items[0];
        assert_eq!((c.name.as_str(), c.file_count), ("Seerah Series", 2));
        assert!(c.topics.contains(&"Seerah".to_string()));
        assert!(c.poster.as_deref().unwrap().ends_with("cover.jpg"));
        let d = detail(idx.conn(), c.id).unwrap().unwrap();
        assert_eq!(d.items[0].label, "Birth");
        assert_eq!(d.items[0].number, Some(1));
        assert_eq!(d.seasons.len(), 1);

        // Quran: a reciter folder of numbered surahs.
        let q = "http://h/Quran/";
        let quran_root = root("Quran", q, SourceKind::Quran);
        let rows = [
            file("Quran", &format!("{q}Mishary%20Alafasy/018.mp3")),
            file("Quran", &format!("{q}Mishary%20Alafasy/001.mp3")),
        ];
        idx.record_level("Quran", &rows, &[], &[], &[]).unwrap();
        assert_eq!(rebuild_titles(&mut idx, &quran_root).unwrap(), 1);
        let page = titles(idx.conn(), &Query { kind: Some("quran".into()), ..Query::default() }).unwrap();
        let d = detail(idx.conn(), page.items[0].id).unwrap().unwrap();
        assert_eq!(d.card.name, "Mishary Alafasy");
        assert_eq!(d.items[0].surah.unwrap().name, "Al-Fatihah");
        assert_eq!(d.items[1].surah.unwrap().name, "Al-Kahf");

        // Progress: half-way through The Night Prayer shows on its card,
        // and up next resumes it.
        let fid = idx
            .conn()
            .query_row("SELECT id FROM files WHERE name = 'The Night Prayer.mp3'", [], |r| r.get::<_, i64>(0))
            .unwrap();
        save_progress(idx.conn(), fid, 300.0, Some(600.0)).unwrap();
        let card = cards_by_ids(idx.conn(), &[prayer.id]).unwrap().remove(0);
        assert!((card.progress.unwrap() - 0.5).abs() < 1e-9);
        save_progress(idx.conn(), fid, 599.0, Some(600.0)).unwrap();
        let d = detail(idx.conn(), prayer.id).unwrap().unwrap();
        assert!(d.items[0].watched);

        // Search finds titles by speaker and by episode name.
        let found = titles(idx.conn(), &Query { q: Some("hijrah".into()), ..Query::default() }).unwrap();
        assert_eq!(found.total, 1);
        let found = titles(idx.conn(), &Query { q: Some("speaker two".into()), ..Query::default() }).unwrap();
        assert_eq!(found.total, 1);
    }
}
