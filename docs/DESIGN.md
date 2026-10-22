# Noor Library: design notes

How Noor Library works inside, and why it is built this way. For what the app
does and how to use it, see the [README](../README.md).

## Layout

| File | Role |
|---|---|
| `src/config.rs` | TOML config. A source (`[[roots]]`) has exactly one of `url`, `path`, `youtube` or `feed`, plus `kind` (lectures, series, quran), an optional `speaker`, and `speaker_folders`. |
| `src/crawler.rs` | Breadth-first crawl of HTTP folder listings and local folders, with `visited`/`pending` checkpoints. It sends YouTube to `youtube.rs` and feeds to `feed.rs`. |
| `src/youtube.rs` | YouTube Data API v3: resolves a handle, channel, user or playlist to its uploads playlist, lists the videos, and fetches their durations. |
| `src/feed.rs` | RSS (with iTunes tags) and Atom, parsed with roxmltree. It reads durations, dates and artwork. |
| `src/index.rs` | SQLite schema: `files` (with an external-content FTS5 index), `images`, `titles` and `titles_fts`, `progress`, `source_meta`, `speakers`, and others. |
| `src/naming.rs` | Reads names: episode numbers (Latin and Bangla digits), surah numbers, language detection, honorifics, and speakers named in titles. |
| `src/topics.rs` | Topic keys with English and Bangla keywords, matched on whole words. |
| `src/quran.rs` | All 114 surahs: Arabic, English and Bangla names, verse counts, and Makki or Madani. |
| `src/library.rs` | Turns files into titles (`rebuild_titles`), and holds every query the UI needs. |
| `src/web.rs` | axum server: JSON API, `/media/{id}` with Range support, and the embedded UI. |
| `src/desktop.rs` | tao + wry window, single-instance lock, and data folder `%APPDATA%\NoorLibrary`. |
| `src/assistant.rs` | Gemini assistant in two passes. |
| `src/ui/` | `index.html`, `app.css` and `app.js`: the whole UI, with no build step. |

## Fetch, then write

The crawler never holds the SQLite connection across an `.await`. Each
folder level is fetched concurrently (in chunks of 200 with `block_on`). The
results are then written in one transaction, which also records which folders
are done and which are next. If you stop or close the app mid-sync, the next
sync carries on from the checkpoint.

## Grouping

`rebuild_titles` runs after each sync and on **Rebuild library**. It clears
every `title_id`, then groups each source by its `kind`:

- **Lectures:**
  - With `speaker_folders`, the first folder level is the speaker.
  - Files that differ only by a part number ("Tawheed 01", "Tawheed 02")
    become one lecture with parts.
  - A folder holding one talk becomes that talk.
- **Series:** each folder, or each volume folder, becomes one series, with
  episodes ordered by their parsed numbers.
- **Quran:** each reciter folder becomes one title. Files are matched to
  surahs by number (1–114).
- **YouTube and feeds:**
  - As lectures, every video or episode is its own title, and the speaker
    comes from the title ("… | Dr. X", "… by Sh. X").
  - Joint talks are credited to their lead speaker.
  - As a series, the whole source is one title, ordered by position or date.

Before grouping, `drop_alternate_formats` keeps one file for each name stem in
a folder, preferring MP3. This means `talk.mp3` plus `talk.wav` shows once.

Artwork is the folder's own picture, so `poster`, `cover`, `folder` and
`thumb` win. The Internet Archive adds machine-made images to every audio
item:

- `X_spectrogram.png` is a spectrogram.
- `X.png` next to it is a waveform.
- `__ia_thumb.jpg` is cut from one of those unless the item has a real
  picture.

All of these are skipped. A title with no artwork gets a generated tile in
the UI.

## Progress

The player posts `/api/progress` about every 8 seconds and on pause. A
position at 95% or more counts as finished. *Up next* on a series or Quran
page is the first unfinished item after the last one played, and it resumes
from the saved position once that is past 30 seconds.

## Security

The UI server binds to `127.0.0.1` and rejects requests whose `Host` header
isn't the loopback address. This blocks DNS-rebinding pages. `/media/{id}`
serves only files in the index, and `/api/open-url` opens only a short
allowlist of sites:

- Google AI Studio
- Google Cloud Console
- Google developer docs
- VideoLAN
- YouTube

API keys are stored only in the local config file. The settings page reads
them back through the loopback-only API so you can view and edit them.

## Gemini assistant

1. **Intent:** Gemini turns the request into a plan (speakers, topics,
   language, kind, maximum minutes, and keywords in English and Bangla). It
   uses a summary of the library to do this.
2. **Candidates:** the app runs that plan against the local index (FTS,
   speaker, topic, then recent items) and collects up to 120 candidates.
3. **Choose:** Gemini picks from the candidates by id. Any id that isn't a
   candidate is dropped, so the assistant can't recommend something you
   don't have.

The prompts forbid religious rulings. Only chat-capable models are offered,
and a busy model falls back to a lighter one.
