//! The app's backend: an axum server on 127.0.0.1 serving the single-page
//! UI and its JSON API. Used both by the desktop window and by
//! `noor serve`.
//!
//! Why local and plain HTTP: the media server is usually on a LAN address.
//! A page served from any public origin has no route to it, an HTTPS page is
//! blocked from loading HTTP media (mixed content), and listing servers send
//! no CORS headers. A page on http://127.0.0.1 has none of those problems.
//! `<video src>` and `<img src>` need no CORS at all, so the page can load
//! posters and stream video straight from the media server.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock, RwLockReadGuard};

use anyhow::{anyhow, bail, Context, Result};
use axum::body::Body;
use axum::extract::{Path as UrlPath, Query, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use url::Url;

use crate::assistant;
use crate::config::{Config, Root, SourceKind};
use crate::crawler::{self, percent_decode};
use crate::index::Index;
use crate::jobs::{self, Job, Jobs};
use crate::library::{self, Query as TitleQuery};
use crate::player;
use crate::youtube::{self, YouTube};
use crate::{feed, quran};

// `rusqlite::Connection` is `Send` but not `Sync`; the Mutex is what lets
// handlers on different threads share it. Handlers take a guard, use it and
// drop it within one statement or block, never across an `.await` (a std
// guard isn't `Send`, so the compiler would refuse anyway).
#[derive(Clone)]
struct AppState {
    index: Arc<Mutex<Index>>,
    config: Arc<RwLock<Config>>,
    config_path: Arc<PathBuf>,
    jobs: Jobs,
}

impl AppState {
    // A panic elsewhere shouldn't brick the UI, so poisoned locks are
    // recovered rather than propagated. SQLite rolls back unfinished
    // transactions itself, so no invariant spans a panic.
    fn index(&self) -> MutexGuard<'_, Index> {
        self.index.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn config(&self) -> RwLockReadGuard<'_, Config> {
        self.config.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs `f` against the shared connection.
    fn db<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T, ApiError> {
        f(self.index().conn()).map_err(ApiError::internal)
    }

    /// Applies an edit to a copy of the config, validates it, writes it to
    /// disk, and only then makes it current, so a rejected edit changes
    /// nothing and the file is always loadable.
    fn update_config(&self, edit: impl FnOnce(&mut Config) -> Result<()>) -> Result<(), ApiError> {
        let mut current = self.config.write().unwrap_or_else(PoisonError::into_inner);
        let mut next = current.clone();
        edit(&mut next).map_err(ApiError::bad_request)?;
        next.normalise().map_err(ApiError::bad_request)?;
        next.save(&self.config_path).map_err(ApiError::internal)?;
        *current = next;
        Ok(())
    }

    /// A second connection for heavy work run on the blocking pool.
    fn open_index(&self) -> Result<Index> {
        Index::open(self.config().db_file())
    }
}

/// A bound-but-not-yet-running server, so the desktop app can learn the
/// port (it asks the OS for a free one) before opening its window.
pub struct Server {
    listener: TcpListener,
    app: Router,
}

impl Server {
    pub async fn bind(config: Config, config_path: PathBuf, port: u16) -> Result<Server> {
        let index = Index::open(config.db_file())?;
        let config = Arc::new(RwLock::new(config));
        let state = AppState {
            index: Arc::new(Mutex::new(index)),
            jobs: Jobs::start(Arc::clone(&config)),
            config,
            config_path: Arc::new(config_path),
        };
        // Loopback only: /api/play starts a process on this machine, so it
        // must not be reachable from the network.
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)
            .await
            .with_context(|| format!("binding {addr}"))?;
        Ok(Server {
            listener,
            app: router(state),
        })
    }

    pub fn addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub async fn run(self) -> Result<()> {
        axum::serve(self.listener, self.app).await?;
        Ok(())
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { Html(INDEX_HTML) }))
        .route("/app.css", get(|| async { ([(header::CONTENT_TYPE, "text/css")], APP_CSS) }))
        .route("/app.js", get(|| async { ([(header::CONTENT_TYPE, "text/javascript")], APP_JS) }))
        .route("/logo.png", get(|| async { ([(header::CONTENT_TYPE, "image/png")], LOGO_PNG) }))
        .route("/api/state", get(get_state))
        .route("/api/home", get(home))
        .route("/api/titles", get(titles))
        .route("/api/facets", get(facets))
        .route("/api/titles/{id}", get(title_detail))
        .route("/api/speakers", get(speakers))
        .route("/api/surahs", get(|| async { Json(json!({ "surahs": &quran::SURAHS[..] })) }))
        .route("/api/recent", get(recent))
        .route("/api/progress", post(progress))
        .route("/api/watchlist", post(watchlist))
        .route("/api/watched", post(watched))
        .route("/api/play", post(play))
        .route("/api/playlist", post(playlist))
        .route("/api/open-folder", post(open_folder))
        .route("/api/open-url", post(open_url))
        .route("/api/pick", post(pick))
        .route("/api/sources", post(add_source))
        .route("/api/sources/check", post(check_source))
        .route("/api/sources/{name}", put(edit_source).delete(delete_source))
        .route("/api/sources/{name}/sync", post(sync_source))
        .route("/api/sources/{name}/stop", post(stop_source))
        .route("/api/sync", post(sync_all))
        .route("/api/jobs/stop", post(stop_jobs))
        .route("/api/settings", get(get_settings))
        .route("/api/settings/player", put(save_player))
        .route("/api/settings/youtube", put(save_youtube))
        .route("/api/settings/ui", put(save_ui))
        .route("/api/settings/assistant", put(save_assistant))
        .route("/api/settings/library", put(save_library))
        .route("/api/youtube/test", post(youtube_test))
        .route("/api/assistant", get(chat_history).post(chat).delete(chat_clear))
        .route("/api/assistant/models", get(models))
        .route("/api/storage", get(storage))
        .route("/api/storage/{action}", post(storage_action))
        .route("/media/{id}", get(media))
        .route("/poster/{id}", get(poster))
        .layer(middleware::from_fn(require_local_host))
        .with_state(state)
}

/// Rejects requests whose Host header isn't loopback. Binding to 127.0.0.1
/// stops other machines; this stops DNS rebinding, where a hostile web page
/// points its own hostname at 127.0.0.1 to talk to us same-origin.
async fn require_local_host(req: Request, next: Next) -> Response {
    let host = req
        .headers()
        .get(header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let hostname = match host.rsplit_once(':') {
        Some((h, port)) if port.chars().all(|c| c.is_ascii_digit()) => h,
        _ => host,
    };
    if matches!(hostname, "127.0.0.1" | "localhost" | "[::1]") {
        next.run(req).await
    } else {
        ApiError::new(StatusCode::FORBIDDEN, anyhow!("unexpected Host header")).into_response()
    }
}

type ApiResult<T = Json<Value>> = Result<T, ApiError>;

fn ok() -> ApiResult {
    Ok(Json(json!({ "ok": true })))
}

// ---------------------------------------------------------------- state ---

#[derive(Serialize)]
struct SourceView {
    name: String,
    kind: SourceKind,
    location: String,
    origin: &'static str,
    speaker: Option<String>,
    speaker_folders: bool,
    count: i64,
    last_sync: Option<i64>,
    status: Option<String>,
    error: Option<String>,
    syncing: bool,
    queued: bool,
}

async fn get_state(State(st): State<AppState>) -> ApiResult {
    let counts = st.db(library::counts)?;
    let states: Vec<(String, Option<i64>, Option<String>, Option<String>)> = st.db(|c| {
        let mut stmt = c.prepare("SELECT root, last_sync, status, error FROM sources_state")?;
        let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    })?;
    let job = st.jobs.status();
    let cfg = st.config().clone();
    let sources: Vec<SourceView> = cfg
        .roots
        .iter()
        .map(|r| {
            let s = states.iter().find(|s| s.0 == r.name);
            SourceView {
                name: r.name.clone(),
                kind: r.kind,
                location: r.location().to_string(),
                origin: r.origin(),
                speaker: r.speaker.clone(),
                speaker_folders: r.speaker_folders,
                count: counts.per_root.get(&r.name).copied().unwrap_or(0),
                last_sync: s.and_then(|s| s.1),
                status: s.and_then(|s| s.2.clone()),
                error: s.and_then(|s| s.3.clone()),
                syncing: job.current_root.as_deref() == Some(&r.name),
                queued: job.queued_roots.contains(&r.name),
            }
        })
        .collect();
    let mut job = job;
    let log_len = job.log.len();
    job.log.drain(..log_len.saturating_sub(120));
    Ok(Json(json!({
        "counts": counts,
        "sources": sources,
        "job": job,
        "flags": {
            "youtube": !cfg.youtube.api_key.is_empty(),
            "assistant": !cfg.assistant.api_key.is_empty(),
            "model": cfg.assistant.model,
            "external_only": cfg.player.external_only,
            "player": cfg.player.command,
            "language": cfg.ui.language,
        },
    })))
}

async fn home(State(st): State<AppState>) -> ApiResult {
    let rows = st.db(library::home)?;
    Ok(Json(json!({ "rows": rows })))
}

async fn titles(State(st): State<AppState>, Query(q): Query<TitleQuery>) -> ApiResult {
    let page = st.db(|c| library::titles(c, &q))?;
    Ok(Json(serde_json::to_value(page).map_err(|e| ApiError::internal(e.into()))?))
}

async fn facets(State(st): State<AppState>, Query(q): Query<TitleQuery>) -> ApiResult {
    let f = st.db(|c| library::facets(c, &q))?;
    Ok(Json(serde_json::to_value(f).map_err(|e| ApiError::internal(e.into()))?))
}

async fn recent(State(st): State<AppState>) -> ApiResult {
    let items = st.db(|c| library::recently_played(c, 200))?;
    Ok(Json(json!({ "total": items.len(), "items": items })))
}

async fn title_detail(State(st): State<AppState>, UrlPath(id): UrlPath<i64>) -> ApiResult {
    let detail = st
        .db(|c| library::detail(c, id))?
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, anyhow!("no such title")))?;
    Ok(Json(serde_json::to_value(detail).map_err(|e| ApiError::internal(e.into()))?))
}

#[derive(Deserialize)]
struct SpeakerQuery {
    q: Option<String>,
}

async fn speakers(State(st): State<AppState>, Query(q): Query<SpeakerQuery>) -> ApiResult {
    let list = st.db(|c| library::speakers(c, q.q.as_deref()))?;
    Ok(Json(json!({ "speakers": list })))
}

#[derive(Deserialize)]
struct ProgressRequest {
    file_id: i64,
    position: f64,
    duration: Option<f64>,
}

/// The in-app player reports where it is every few seconds and on pause,
/// so "Continue listening" can resume at the right spot.
async fn progress(State(st): State<AppState>, Json(p): Json<ProgressRequest>) -> ApiResult {
    let duration = p.duration.filter(|d| d.is_finite() && *d > 0.0);
    if p.position.is_finite() && p.position >= 0.0 {
        st.db(|c| library::save_progress(c, p.file_id, p.position, duration))?;
    }
    ok()
}

#[derive(Deserialize)]
struct Toggle {
    #[serde(default)]
    title_id: i64,
    #[serde(default)]
    file_id: i64,
    on: bool,
}

async fn watchlist(State(st): State<AppState>, Json(t): Json<Toggle>) -> ApiResult {
    st.db(|c| library::set_watchlist(c, t.title_id, t.on))?;
    ok()
}

async fn watched(State(st): State<AppState>, Json(t): Json<Toggle>) -> ApiResult {
    st.db(|c| library::set_watched(c, t.file_id, t.on))?;
    ok()
}

// ----------------------------------------------------------------- play ---

#[derive(Deserialize)]
struct PlayRequest {
    file_id: Option<i64>,
    url: Option<String>,
    /// The UI played it in its own player; just record the play.
    #[serde(default)]
    record_only: bool,
}

fn is_youtube(url: &str) -> bool {
    youtube::video_id(url).is_some()
}

async fn play(State(st): State<AppState>, Json(req): Json<PlayRequest>) -> ApiResult {
    // Only play what was indexed. Even with the scheme check, an open
    // "launch the player on any URL" endpoint is more power than the UI needs.
    let file = st.db(|c| match (req.file_id, &req.url) {
        (Some(id), _) => Ok(library::file_info(c, id)?.map(|f| (id, f.0))),
        (None, Some(url)) => Ok(c
            .query_row("SELECT id, url FROM files WHERE url = ?1", [url], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .ok()),
        _ => Ok(None),
    })?;
    let Some((file_id, url)) = file else {
        if let Some(url) = &req.url {
            player::validate_url(url).map_err(ApiError::bad_request)?;
        }
        return Err(ApiError::bad_request(anyhow!("that video is not in the library")));
    };
    if !req.record_only {
        if is_youtube(&url) {
            // A YouTube page is best in the browser; desktop players can't
            // reliably stream it.
            open_external(&url).map_err(ApiError::internal)?;
        } else {
            let player_cfg = st.config().player.clone();
            player::play_indexed(&player_cfg, &url).map_err(ApiError::bad_request)?;
        }
    }
    st.db(|c| library::record_play(c, file_id, !req.record_only))?;
    ok()
}

#[derive(Deserialize)]
struct PlaylistRequest {
    title_id: i64,
    from_file_id: Option<i64>,
    season: Option<u32>,
}

/// Writes an M3U playlist of a title's files and opens it in the player,
/// so a whole season plays back to back.
async fn playlist(State(st): State<AppState>, Json(req): Json<PlaylistRequest>) -> ApiResult {
    let (files, name) = st.db(|c| {
        let mut files = library::title_files(c, req.title_id)?;
        if let Some(season) = req.season {
            let in_season: Vec<i64> = {
                let mut stmt = c.prepare("SELECT id FROM files WHERE title_id = ?1 AND season = ?2")?;
                let rows = stmt.query_map(rusqlite::params![req.title_id, season], |r| r.get(0))?;
                rows.collect::<rusqlite::Result<_>>()?
            };
            files.retain(|f| in_season.contains(&f.0));
        }
        if let Some(from) = req.from_file_id {
            if let Some(pos) = files.iter().position(|f| f.0 == from) {
                files.drain(..pos);
            }
        }
        let name: String = c.query_row("SELECT name FROM titles WHERE id = ?1", [req.title_id], |r| r.get(0))?;
        Ok((files, name))
    })?;
    if files.is_empty() {
        return Err(ApiError::bad_request(anyhow!("nothing to play")));
    }
    let mut m3u = String::from("#EXTM3U\n");
    for (_, url, file_name) in &files {
        let target = match Url::parse(url) {
            Ok(u) if u.scheme() == "file" => u
                .to_file_path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| url.clone()),
            _ => url.clone(),
        };
        m3u.push_str(&format!("#EXTINF:-1,{name} - {file_name}\n{target}\n"));
    }
    let (path, player_cfg) = {
        let cfg = st.config();
        (cfg.base_dir.join("playlist.m3u8"), cfg.player.clone())
    };
    std::fs::write(&path, m3u).map_err(|e| ApiError::internal(e.into()))?;
    player::play_file(&player_cfg, &path).map_err(ApiError::bad_request)?;
    let first = files[0].0;
    st.db(|c| library::record_play(c, first, true))?;
    Ok(Json(json!({ "ok": true, "count": files.len() })))
}

#[derive(Deserialize)]
struct FolderRequest {
    title_id: Option<i64>,
    path: Option<String>,
}

/// Opens a title's folder: Explorer for local folders, the browser for
/// folders on a media server.
async fn open_folder(State(st): State<AppState>, Json(req): Json<FolderRequest>) -> ApiResult {
    let target = match (req.title_id, req.path) {
        (Some(id), _) => st
            .db(|c| library::title_dir(c, id))?
            .ok_or_else(|| ApiError::bad_request(anyhow!("no such title")))?,
        // Only the app's own data folder may be opened by path.
        (None, Some(p)) if Path::new(&p) == st.config().base_dir => p,
        _ => return Err(ApiError::bad_request(anyhow!("nothing to open"))),
    };
    let target = match Url::parse(&target) {
        Ok(u) if u.scheme() == "file" => u
            .to_file_path()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or(target),
        _ => target,
    };
    open_external(&target).map_err(ApiError::internal)?;
    ok()
}

#[derive(Deserialize)]
struct UrlRequest {
    url: String,
}

/// Opens help links in the real browser. Allow-listed, so the endpoint
/// can't be used to launch arbitrary URLs.
async fn open_url(Json(req): Json<UrlRequest>) -> ApiResult {
    let url = Url::parse(&req.url).map_err(|e| ApiError::bad_request(e.into()))?;
    let allowed = [
        "aistudio.google.com",
        "console.cloud.google.com",
        "developers.google.com",
        "www.videolan.org",
        "www.youtube.com",
    ];
    if url.scheme() != "https" || !allowed.contains(&url.host_str().unwrap_or("")) {
        return Err(ApiError::bad_request(anyhow!("that link can't be opened from here")));
    }
    open_external(url.as_str()).map_err(ApiError::internal)?;
    ok()
}

fn open_external(target: &str) -> Result<()> {
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("explorer");
        c.arg(target);
        c
    } else if cfg!(target_os = "macos") {
        let mut c = std::process::Command::new("open");
        c.arg(target);
        c
    } else {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(target);
        c
    };
    cmd.spawn().context("opening")?;
    Ok(())
}

#[derive(Deserialize)]
struct PickRequest {
    kind: String,
}

/// Native folder / program picker. The dialog blocks, so it runs on the
/// blocking pool rather than an async worker.
async fn pick(Json(req): Json<PickRequest>) -> ApiResult {
    let path = tokio::task::spawn_blocking(move || match req.kind.as_str() {
        "program" => rfd::FileDialog::new()
            .set_title("Choose your video player")
            .add_filter("Programs", &["exe"])
            .pick_file(),
        _ => rfd::FileDialog::new()
            .set_title("Choose a folder with videos")
            .pick_folder(),
    })
    .await
    .map_err(|e| ApiError::internal(anyhow!(e)))?;
    Ok(Json(json!({ "path": path.map(|p| p.display().to_string()) })))
}

// -------------------------------------------------------------- sources ---

#[derive(Deserialize)]
struct SourceRequest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    location: String,
    #[serde(default)]
    kind: SourceKind,
    /// "web", "folder", "youtube" or "feed", when the UI already knows
    /// (from Check link); otherwise worked out from the location.
    origin: Option<String>,
    #[serde(default)]
    speaker: Option<String>,
    #[serde(default)]
    speaker_folders: bool,
    #[serde(default = "yes")]
    sync: bool,
}

fn yes() -> bool {
    true
}

/// Where a source lives: exactly one field is set.
#[derive(Debug, Clone, Default)]
struct Location {
    url: Option<String>,
    path: Option<String>,
    youtube: Option<String>,
    feed: Option<String>,
}

impl Location {
    fn as_str(&self) -> &str {
        self.url
            .as_deref()
            .or(self.path.as_deref())
            .or(self.youtube.as_deref())
            .or(self.feed.as_deref())
            .unwrap_or("")
    }
}

fn looks_like_youtube(loc: &str) -> bool {
    loc.starts_with('@') || loc.contains("youtube.com/") || loc.contains("youtu.be/")
}

/// Works out what a pasted location is: a YouTube link, a folder on this PC,
/// a podcast feed (an RSS / Atom document) or a server's folder listing.
async fn resolve_location(cfg: &Config, location: &str, hint: Option<&str>) -> Result<Location> {
    let loc = location.trim();
    if loc.is_empty() {
        bail!("paste a link or choose a folder");
    }
    if hint == Some("youtube") || looks_like_youtube(loc) {
        youtube::parse_target(loc)?;
        return Ok(Location { youtube: Some(loc.to_string()), ..Location::default() });
    }
    if !(loc.starts_with("http://") || loc.starts_with("https://")) {
        if Path::new(loc).is_dir() {
            return Ok(Location { path: Some(loc.to_string()), ..Location::default() });
        }
        bail!("that is not a web link (http://...), a YouTube link, or a folder on this PC");
    }
    match hint {
        Some("feed") => return Ok(Location { feed: Some(loc.to_string()), ..Location::default() }),
        Some("web") => return Ok(Location { url: Some(loc.to_string()), ..Location::default() }),
        _ => {}
    }
    let (is_feed, _) = sniff(cfg, loc).await?;
    Ok(if is_feed {
        Location { feed: Some(loc.to_string()), ..Location::default() }
    } else {
        Location { url: Some(loc.to_string()), ..Location::default() }
    })
}

/// Downloads a link once: (is it a feed?, body).
async fn sniff(cfg: &Config, link: &str) -> Result<(bool, String)> {
    let client = crawler::client(cfg)?;
    let resp = client
        .get(link)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| anyhow!("couldn't open the link: {e}"))?;
    let ctype = resp
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();
    let body = resp.text().await?;
    let head: String = body.trim_start().chars().take(600).collect::<String>().to_ascii_lowercase();
    let xml_doc = head.starts_with("<?xml") || head.starts_with("<rss") || head.starts_with("<feed");
    let has_feed_root = head.contains("<rss") || head.contains("<feed") || body.contains("<channel>");
    let is_feed = !ctype.contains("html") && (ctype.contains("xml") || ctype.contains("rss") || xml_doc) && has_feed_root;
    Ok((is_feed, body))
}

async fn add_source(State(st): State<AppState>, Json(req): Json<SourceRequest>) -> ApiResult {
    let cfg = st.config().clone();
    let loc = resolve_location(&cfg, &req.location, req.origin.as_deref())
        .await
        .map_err(ApiError::bad_request)?;
    let mut name = req.name.trim().to_string();
    if let Some(existing) = cfg.roots.iter().find(|r| same_location(r.location(), loc.as_str())) {
        return Err(ApiError::bad_request(anyhow!("already added as \"{}\"", existing.name)));
    }
    if name.is_empty() {
        name = default_name(loc.as_str());
    }
    name = unique_name(&cfg, &name);
    st.update_config(|cfg| {
        cfg.roots.push(Root {
            name: name.clone(),
            url: loc.url.clone(),
            path: loc.path.clone(),
            youtube: loc.youtube.clone(),
            feed: loc.feed.clone(),
            kind: req.kind,
            speaker: req.speaker.clone(),
            speaker_folders: req.speaker_folders,
        });
        Ok(())
    })?;
    if req.sync {
        st.jobs.enqueue(Job::Sync { root: name.clone(), full: false });
    }
    Ok(Json(json!({ "ok": true, "name": name })))
}

#[derive(Deserialize)]
struct CheckRequest {
    location: String,
}

/// "Check link": looks at the location once and reports what is there,
/// with a guess at the source type and a name.
async fn check_source(State(st): State<AppState>, Json(req): Json<CheckRequest>) -> ApiResult {
    let cfg = st.config().clone();
    let loc = req.location.trim().to_string();

    if looks_like_youtube(&loc) {
        let target = youtube::parse_target(&loc).map_err(ApiError::bad_request)?;
        let yt = YouTube::new(&cfg.youtube.api_key).map_err(ApiError::bad_request)?;
        let (title, author, count, playlist) = yt.describe(&target).await.map_err(ApiError::bad_request)?;
        return Ok(Json(json!({
            "ok": true, "origin": "youtube", "name": title, "speaker": author,
            "items": count, "folders": 0,
            "kind": if playlist { "series" } else { "lectures" },
            "sample": [],
        })));
    }

    let (origin, dirs, files): (&str, Vec<String>, Vec<String>) =
        if loc.starts_with("http://") || loc.starts_with("https://") {
            let (is_feed, body) = sniff(&cfg, &loc).await.map_err(ApiError::bad_request)?;
            if is_feed {
                let f = feed::parse(&body).map_err(ApiError::bad_request)?;
                if f.items.is_empty() {
                    return Err(ApiError::bad_request(anyhow!(
                        "the feed works, but has no audio or video episodes"
                    )));
                }
                return Ok(Json(json!({
                    "ok": true, "origin": "feed",
                    "name": f.title.clone().unwrap_or_else(|| default_name(&loc)),
                    "speaker": f.author, "items": f.items.len(), "folders": 0, "kind": "series",
                    "sample": f.items.iter().take(5).map(|e| e.title.clone()).collect::<Vec<_>>(),
                })));
            }
            let mut base = Url::parse(&loc).map_err(|e| ApiError::bad_request(e.into()))?;
            if !base.path().ends_with('/') {
                base.set_path(&format!("{}/", base.path()));
            }
            let listing = crawler::parse_listing(&base, &body);
            (
                "web",
                listing.dirs.into_iter().map(|d| d.name).collect(),
                listing.files.into_iter().map(|f| f.name).collect(),
            )
        } else {
            let dir = PathBuf::from(&loc);
            if !dir.is_dir() {
                return Err(ApiError::bad_request(anyhow!(
                    "that is not a web link (http://...), a YouTube link, or a folder on this PC"
                )));
            }
            let mut dirs = Vec::new();
            let mut files = Vec::new();
            for e in std::fs::read_dir(&dir).map_err(|e| ApiError::bad_request(e.into()))?.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                match e.file_type() {
                    Ok(t) if t.is_dir() => dirs.push(name),
                    Ok(t) if t.is_file() => files.push(name),
                    _ => {}
                }
            }
            ("folder", dirs, files)
        };
    let media: Vec<&String> = files.iter().filter(|f| cfg.is_media(f)).collect();
    if dirs.is_empty() && media.is_empty() {
        return Err(ApiError::bad_request(anyhow!(
            "the link works, but there are no folders or audio/video files in it"
        )));
    }
    let surahs = media.iter().filter(|f| crate::naming::surah_number(f).is_some()).count();
    let lower: Vec<String> = dirs.iter().map(|d| d.to_lowercase()).collect();
    let quran_words = ["quran", "recit", "tilawa", "কুরআন", "কোরআন", "তিলাওয়াত"];
    let quranish = lower.iter().any(|d| quran_words.iter().any(|w| d.contains(w)))
        || (media.len() >= 5 && surahs * 10 >= media.len() * 8);
    let series_words = ["part", "volume", "series", "tafsir", "season", "পর্ব", "সিরিজ", "তাফসীর"];
    let seriesish = lower.iter().filter(|d| series_words.iter().any(|w| d.contains(w))).count();
    let kind = if quranish {
        "quran"
    } else if seriesish * 3 >= dirs.len().max(1) && seriesish > 0 {
        "series"
    } else {
        "lectures"
    };
    Ok(Json(json!({
        "ok": true, "origin": origin,
        "folders": dirs.len(), "items": media.len(), "kind": kind,
        "name": default_name(&loc), "speaker": null,
        "sample": dirs.iter().chain(media.iter().copied()).take(6).collect::<Vec<_>>(),
    })))
}

#[derive(Deserialize)]
struct EditRequest {
    name: String,
    location: String,
    kind: SourceKind,
    origin: Option<String>,
    #[serde(default)]
    speaker: Option<String>,
    #[serde(default)]
    speaker_folders: bool,
}

async fn edit_source(
    State(st): State<AppState>,
    UrlPath(old): UrlPath<String>,
    Json(req): Json<EditRequest>,
) -> ApiResult {
    if st.jobs.involves(&old) {
        return Err(ApiError::conflict(anyhow!("stop this source's sync before editing it")));
    }
    let before = st
        .config()
        .root(&old)
        .cloned()
        .ok_or_else(|| ApiError::bad_request(anyhow!("no source named {old:?}")))?;
    // Keep the source's type unless the link itself changed.
    let hint = if same_location(before.location(), &req.location) {
        Some(before.origin())
    } else {
        req.origin.as_deref()
    };
    let cfg = st.config().clone();
    let loc = resolve_location(&cfg, &req.location, hint).await.map_err(ApiError::bad_request)?;
    let new_name = req.name.trim().to_string();
    st.update_config(|cfg| {
        if new_name != old && cfg.roots.iter().any(|r| r.name.eq_ignore_ascii_case(&new_name)) {
            bail!("another source is already called {new_name:?}");
        }
        let r = cfg
            .roots
            .iter_mut()
            .find(|r| r.name == old)
            .ok_or_else(|| anyhow!("no source named {old:?}"))?;
        r.name = new_name.clone();
        r.url = loc.url.clone();
        r.path = loc.path.clone();
        r.youtube = loc.youtube.clone();
        r.feed = loc.feed.clone();
        r.kind = req.kind;
        r.speaker = req.speaker.clone();
        r.speaker_folders = req.speaker_folders;
        Ok(())
    })?;
    let after = st.config().root(&new_name).cloned().expect("just saved");
    let moved = !same_location(before.location(), after.location());
    let regroup = before.kind != after.kind
        || before.speaker != after.speaker
        || before.speaker_folders != after.speaker_folders;
    let state = st.clone();
    let old_name = old.clone();
    tokio::task::spawn_blocking(move || -> Result<()> {
        let mut index = state.open_index()?;
        if old_name != after.name {
            index.rename_root(&old_name, &after.name)?;
        }
        if moved {
            // A different place: drop what came from the old one.
            index.remove_root(&after.name)?;
        } else if regroup {
            library::rebuild_titles(&mut index, &after)?;
        }
        Ok(())
    })
    .await
    .map_err(|e| ApiError::internal(anyhow!(e)))?
    .map_err(ApiError::internal)?;
    if moved {
        st.jobs.enqueue(Job::Sync { root: new_name, full: true });
    }
    ok()
}

async fn delete_source(State(st): State<AppState>, UrlPath(name): UrlPath<String>) -> ApiResult {
    st.jobs.stop_root(&name);
    if st.jobs.status().current_root.as_deref() == Some(name.as_str()) {
        return Err(ApiError::conflict(anyhow!(
            "stopping this source's sync; try again in a moment"
        )));
    }
    st.update_config(|cfg| {
        let before = cfg.roots.len();
        cfg.roots.retain(|r| r.name != name);
        if cfg.roots.len() == before {
            bail!("no source named {name:?}");
        }
        Ok(())
    })?;
    let state = st.clone();
    let removed = tokio::task::spawn_blocking(move || state.open_index()?.remove_root(&name))
        .await
        .map_err(|e| ApiError::internal(anyhow!(e)))?
        .map_err(ApiError::internal)?;
    Ok(Json(json!({ "ok": true, "removed_files": removed })))
}

async fn sync_source(State(st): State<AppState>, UrlPath(name): UrlPath<String>) -> ApiResult {
    if st.config().root(&name).is_none() {
        return Err(ApiError::bad_request(anyhow!("no source named {name:?}")));
    }
    st.jobs.enqueue(Job::Sync { root: name, full: false });
    ok()
}

async fn stop_source(State(st): State<AppState>, UrlPath(name): UrlPath<String>) -> ApiResult {
    st.jobs.stop_root(&name);
    ok()
}

#[derive(Deserialize, Default)]
struct SyncAllRequest {
    #[serde(default)]
    full: bool,
}

async fn sync_all(State(st): State<AppState>, body: Option<Json<SyncAllRequest>>) -> ApiResult {
    let full = body.map(|Json(b)| b.full).unwrap_or(false);
    let roots = st.config().roots.clone();
    if roots.is_empty() {
        return Err(ApiError::bad_request(anyhow!("add a source first")));
    }
    jobs::sync_all(&st.jobs, &roots, full);
    ok()
}

async fn stop_jobs(State(st): State<AppState>) -> ApiResult {
    st.jobs.stop_all();
    ok()
}

fn same_location(a: &str, b: &str) -> bool {
    let norm = |s: &str| s.trim().trim_end_matches(['/', '\\']).to_lowercase();
    norm(a) == norm(b)
}

/// "D:\Videos\Movies" -> "Movies"; "http://nas/media/" -> "media".
fn default_name(location: &str) -> String {
    let location = location.trim();
    if let Some(handle) = location.strip_prefix('@') {
        return handle.to_string();
    }
    if let Ok(url) = Url::parse(location) {
        if matches!(url.scheme(), "http" | "https") {
            let last = url
                .path_segments()
                .and_then(|mut s| s.rfind(|seg| !seg.is_empty()))
                .map(percent_decode)
                .map(|s| s.trim_start_matches('@').to_string());
            return last.unwrap_or_else(|| url.host_str().unwrap_or("server").to_string());
        }
    }
    Path::new(location.trim())
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| location.trim().trim_end_matches(['\\', '/']).to_string())
}

fn unique_name(cfg: &Config, base: &str) -> String {
    let taken = |n: &str| cfg.roots.iter().any(|r| r.name.eq_ignore_ascii_case(n));
    if !taken(base) {
        return base.to_string();
    }
    (2..)
        .map(|i| format!("{base} {i}"))
        .find(|n| !taken(n))
        .expect("unbounded range")
}

// ------------------------------------------------------------- settings ---

async fn get_settings(State(st): State<AppState>) -> ApiResult {
    let cfg = st.config().clone();
    Ok(Json(json!({
        "ui": cfg.ui,
        "player": cfg.player,
        "youtube": cfg.youtube,
        "assistant": cfg.assistant,
        "library": {
            "skip_dirs": cfg.skip_dirs,
            "media_extensions": cfg.media_extensions,
            "concurrency": cfg.concurrency,
            "delay_ms": cfg.delay_ms,
            "timeout_secs": cfg.timeout_secs,
            "max_depth": cfg.max_depth,
        },
        "paths": {
            "config": st.config_path.display().to_string(),
            "data": cfg.base_dir.display().to_string(),
            "db": cfg.db_file().display().to_string(),
        },
    })))
}

async fn save_player(State(st): State<AppState>, Json(p): Json<crate::config::Player>) -> ApiResult {
    st.update_config(|cfg| {
        if p.command.trim().is_empty() {
            bail!("choose a player program");
        }
        cfg.player = crate::config::Player {
            command: p.command.trim().to_string(),
            args: p.args.into_iter().filter(|a| !a.trim().is_empty()).collect(),
            external_only: p.external_only,
        };
        Ok(())
    })?;
    ok()
}

async fn save_youtube(State(st): State<AppState>, Json(y): Json<crate::config::YouTube>) -> ApiResult {
    st.update_config(|cfg| {
        cfg.youtube = y;
        Ok(())
    })?;
    ok()
}

#[derive(Deserialize)]
struct KeyTest {
    api_key: String,
}

async fn youtube_test(Json(k): Json<KeyTest>) -> ApiResult {
    let yt = YouTube::new(&k.api_key).map_err(ApiError::bad_request)?;
    yt.check().await.map_err(ApiError::bad_request)?;
    ok()
}

async fn save_ui(State(st): State<AppState>, Json(u): Json<crate::config::Ui>) -> ApiResult {
    st.update_config(|cfg| {
        if !matches!(u.language.as_str(), "bn" | "en") {
            bail!("unsupported language {:?}", u.language);
        }
        cfg.ui = u;
        Ok(())
    })?;
    ok()
}

async fn save_assistant(State(st): State<AppState>, Json(a): Json<crate::config::Assistant>) -> ApiResult {
    st.update_config(|cfg| {
        if !assistant::is_chat_model(&a.model) {
            bail!(
                "{} can't write text replies (it's a speech or image model); \
                 choose a chat model such as gemini-flash-latest",
                a.model
            );
        }
        cfg.assistant = a;
        Ok(())
    })?;
    ok()
}

#[derive(Deserialize)]
struct LibrarySettings {
    skip_dirs: Vec<String>,
    media_extensions: Vec<String>,
    concurrency: usize,
    delay_ms: u64,
    timeout_secs: u64,
    max_depth: usize,
}

async fn save_library(State(st): State<AppState>, Json(s): Json<LibrarySettings>) -> ApiResult {
    st.update_config(|cfg| {
        if s.media_extensions.iter().all(|e| e.trim().is_empty()) {
            bail!("list at least one audio or video file type");
        }
        cfg.skip_dirs = s.skip_dirs;
        cfg.media_extensions = s.media_extensions;
        cfg.concurrency = s.concurrency;
        cfg.delay_ms = s.delay_ms;
        cfg.timeout_secs = s.timeout_secs;
        cfg.max_depth = s.max_depth;
        Ok(())
    })?;
    ok()
}

// ------------------------------------------------------------ assistant ---

async fn chat_history(State(st): State<AppState>) -> ApiResult {
    let messages = st.db(|c| assistant::history(c, 100))?;
    Ok(Json(json!({ "messages": messages })))
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

async fn chat(State(st): State<AppState>, Json(req): Json<ChatRequest>) -> ApiResult {
    let message = req.message.trim();
    if message.is_empty() {
        return Err(ApiError::bad_request(anyhow!("type a message")));
    }
    let cfg = st.config().assistant.clone();
    let state = st.clone();
    let with_db = move |f: &mut dyn FnMut(&Connection) -> Result<()>| -> Result<()> {
        f(state.index().conn())
    };
    let answer = assistant::ask(&cfg, message, with_db)
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(serde_json::to_value(answer).map_err(|e| ApiError::internal(e.into()))?))
}

async fn chat_clear(State(st): State<AppState>) -> ApiResult {
    st.db(assistant::clear)?;
    ok()
}

#[derive(Deserialize)]
struct ModelsQuery {
    key: Option<String>,
}

async fn models(State(st): State<AppState>, Query(q): Query<ModelsQuery>) -> ApiResult {
    let key = q
        .key
        .filter(|k| !k.trim().is_empty())
        .unwrap_or_else(|| st.config().assistant.api_key.clone());
    let models = assistant::list_models(key.trim())
        .await
        .map_err(ApiError::bad_request)?;
    Ok(Json(json!({ "models": models })))
}

// -------------------------------------------------------------- storage ---

async fn storage(State(st): State<AppState>) -> ApiResult {
    let db = st.config().db_file();
    let size = |p: &Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let db_bytes = size(&db) + size(&PathBuf::from(format!("{}-wal", db.display())));
    let stats = st.db(|c| {
        let one = |sql: &str| -> Result<i64> { Ok(c.query_row(sql, [], |r| r.get(0))?) };
        Ok(json!({
            "files": one("SELECT COUNT(*) FROM files")?,
            "titles": one("SELECT COUNT(*) FROM titles")?,
            "lectures": one("SELECT COUNT(*) FROM titles WHERE kind = 'lecture'")?,
            "series": one("SELECT COUNT(*) FROM titles WHERE kind = 'series'")?,
            "quran": one("SELECT COUNT(*) FROM titles WHERE kind = 'quran'")?,
            "speakers": one("SELECT COUNT(DISTINCT speaker) FROM titles WHERE speaker IS NOT NULL AND speaker != ''")?,
            "posters": one("SELECT COUNT(*) FROM titles WHERE poster IS NOT NULL")?,
            "watchlist": one("SELECT COUNT(*) FROM watchlist")?,
            "plays": one("SELECT COUNT(*) FROM plays")?,
            "chat": one("SELECT COUNT(*) FROM chat")?,
            "bytes": one("SELECT COALESCE(SUM(size), 0) FROM files")?,
            "seconds": one("SELECT COALESCE(SUM(duration), 0) FROM files")?,
        }))
    })?;
    let cfg = st.config().clone();
    Ok(Json(json!({
        "stats": stats,
        "db_bytes": db_bytes,
        "db_path": db.display().to_string(),
        "data_dir": cfg.base_dir.display().to_string(),
        "config_path": st.config_path.display().to_string(),
    })))
}

async fn storage_action(State(st): State<AppState>, UrlPath(action): UrlPath<String>) -> ApiResult {
    match action.as_str() {
        "clear-history" => st.db(|c| {
            c.execute_batch("DELETE FROM plays; DELETE FROM watched; DELETE FROM progress;")?;
            Ok(())
        })?,
        "rebuild" => {
            if st.jobs.status().running {
                return Err(ApiError::conflict(anyhow!("wait for the current sync to finish")));
            }
            let state = st.clone();
            tokio::task::spawn_blocking(move || -> Result<()> {
                let roots = state.config().roots.clone();
                let mut index = state.open_index()?;
                for r in &roots {
                    library::rebuild_titles(&mut index, r)?;
                }
                Ok(())
            })
            .await
            .map_err(|e| ApiError::internal(anyhow!(e)))?
            .map_err(ApiError::internal)?;
        }
        "reset" => {
            st.jobs.stop_all();
            if st.jobs.status().running {
                return Err(ApiError::conflict(anyhow!("stopping the current job; try again in a moment")));
            }
            st.db(|c| {
                c.execute_batch(
                    "DELETE FROM files; DELETE FROM images; DELETE FROM titles; DELETE FROM titles_fts;
                     DELETE FROM visited; DELETE FROM pending; DELETE FROM sources_state;
                     DELETE FROM source_meta; DELETE FROM speakers; DELETE FROM watchlist;
                     DELETE FROM plays; DELETE FROM watched; DELETE FROM progress; DELETE FROM chat;",
                )?;
                Ok(())
            })?;
        }
        "open" => {
            let dir = st.config().base_dir.clone();
            open_external(&dir.display().to_string()).map_err(ApiError::internal)?;
        }
        _ => return Err(ApiError::bad_request(anyhow!("unknown action"))),
    }
    ok()
}

// ---------------------------------------------------------------- media ---

/// Streams an indexed local file to the `<video>` element. `ServeFile`
/// handles Range requests, which is what makes seeking work.
async fn media(State(st): State<AppState>, UrlPath(id): UrlPath<i64>, req: Request) -> Result<Response, ApiError> {
    let url = st.db(|c| Ok(library::file_info(c, id)?.map(|f| f.0)))?;
    serve_local(url, req).await
}

/// A poster image from a local source folder.
async fn poster(State(st): State<AppState>, UrlPath(id): UrlPath<i64>, req: Request) -> Result<Response, ApiError> {
    let url = st.db(|c| library::local_poster(c, id))?;
    serve_local(url, req).await
}

async fn serve_local(url: Option<String>, req: Request) -> Result<Response, ApiError> {
    let path = url
        .and_then(|u| Url::parse(&u).ok())
        .filter(|u| u.scheme() == "file")
        .and_then(|u| u.to_file_path().ok())
        .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, anyhow!("no such local file")))?;
    match ServeFile::new(path).oneshot(req).await {
        Ok(res) => Ok(res.map(Body::new)),
        Err(never) => match never {},
    }
}

/// Can a browser `<video>` element likely play this file?
///
/// Keyed on the container only, which is a hint and nothing more: the
/// extension can't say which codec is inside. Chromium (and so the desktop
/// app's WebView2) has no official Matroska support, and HEVC/x265 is
/// unlicensed in most builds whatever the container, so a typical `.mkv`
/// x265 release won't play. An `.mp4` can still fail (HEVC video, or AC3/DTS
/// audio that plays silent); the UI's `onerror` fallback covers that.
pub fn playability(name: &str) -> &'static str {
    let ext = name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "mp3" | "m4a" | "aac" | "ogg" | "opus" | "wav" | "flac" | "mp4" | "m4v" | "webm" => "likely",
        "mov" | "ogv" | "3gp" => "maybe",
        _ => "no",
    }
}

// --------------------------------------------------------------- errors ---

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, err: anyhow::Error) -> Self {
        ApiError {
            status,
            message: format!("{err:#}"),
        }
    }
    fn bad_request(err: anyhow::Error) -> Self {
        Self::new(StatusCode::BAD_REQUEST, err)
    }
    fn conflict(err: anyhow::Error) -> Self {
        Self::new(StatusCode::CONFLICT, err)
    }
    fn internal(err: anyhow::Error) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

const INDEX_HTML: &str = include_str!("ui/index.html");
const APP_CSS: &str = include_str!("ui/app.css");
const APP_JS: &str = include_str!("ui/app.js");
const LOGO_PNG: &[u8] = include_bytes!("../assets/icon-256.png");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playability_by_container() {
        assert_eq!(playability("Lecture.mp3"), "likely");
        assert_eq!(playability("Talk.M4A"), "likely");
        assert_eq!(playability("clip.webm"), "likely");
        assert_eq!(playability("clip.mov"), "maybe");
        assert_eq!(playability("Waz.2024.720p.x265.mkv"), "no");
    }

    #[test]
    fn source_names() {
        assert_eq!(default_name(r"D:\Lectures\Tafsir"), "Tafsir");
        assert_eq!(default_name("http://nas.local/my%20waz/"), "my waz");
        assert_eq!(default_name("http://nas.local/"), "nas.local");
        assert_eq!(default_name("https://www.youtube.com/@SomeChannel"), "SomeChannel");
        assert_eq!(default_name("@another"), "another");
        let mut cfg = Config::default();
        cfg.roots.push(Root {
            name: "Waz".into(),
            url: None,
            path: Some("x".into()),
            youtube: None,
            feed: None,
            kind: SourceKind::Lectures,
            speaker: None,
            speaker_folders: false,
        });
        assert_eq!(unique_name(&cfg, "waz"), "waz 2");
        assert!(same_location(r"D:\Lectures\", r"d:\lectures"));
        assert!(looks_like_youtube("https://youtu.be/abc"));
        assert!(!looks_like_youtube("http://nas.local/waz/"));
    }
}
