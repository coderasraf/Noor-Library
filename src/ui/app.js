'use strict';

/* =====================================================================
   Noor Library front end. No framework: a tiny element builder, a hash
   router, one function per page, and a player bar that survives page
   changes. Every value from the server is inserted with textContent or
   attributes, never as HTML, because file and folder names are untrusted.
   ===================================================================== */

// ------------------------------------------------------------- icons
const ICONS = {
  home: '<path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M9 22V12h6v10"/>',
  mic: '<rect x="9" y="2" width="6" height="12" rx="3"/><path d="M19 10v1a7 7 0 0 1-14 0v-1M12 18v4M8 22h8"/>',
  layers: '<path d="m12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83Z"/><path d="m22 17.65-9.17 4.16a2 2 0 0 1-1.66 0L2 17.65"/><path d="m22 12.65-9.17 4.16a2 2 0 0 1-1.66 0L2 12.65"/>',
  film: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M7 3v18M17 3v18M3 7.5h4M3 12h18M3 16.5h4M17 7.5h4M17 16.5h4"/>',
  book: '<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>',
  users: '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"/>',
  tag: '<path d="M12.6 2.6A2 2 0 0 0 11.2 2H4a2 2 0 0 0-2 2v7.2a2 2 0 0 0 .6 1.4l8.7 8.7a2.4 2.4 0 0 0 3.4 0l6.6-6.6a2.4 2.4 0 0 0 0-3.4z"/><circle cx="7.5" cy="7.5" r=".5" fill="currentColor"/>',
  bookmark: '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>',
  history: '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  sliders: '<path d="M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3M14 2v4M8 10v4M16 18v4"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
  play: '<path d="M6 3.5v17a1 1 0 0 0 1.5.86l14-8.5a1 1 0 0 0 0-1.72l-14-8.5A1 1 0 0 0 6 3.5z"/>',
  pause: '<rect x="5" y="3" width="5" height="18" rx="1.5"/><rect x="14" y="3" width="5" height="18" rx="1.5"/>',
  prev: '<path d="M19 20 9 12l10-8z"/><path d="M5 19V5"/>',
  next: '<path d="m5 4 10 8-10 8z"/><path d="M19 5v14"/>',
  back15: '<path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>',
  fwd30: '<path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/>',
  volume: '<path d="M11 5 6 9H2v6h4l5 4z"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07M19.07 4.93a10 10 0 0 1 0 14.14"/>',
  expand: '<path d="M15 3h6v6M9 21H3v-6M21 3l-7 7M3 21l7-7"/>',
  shrink: '<path d="M4 14h6v6M20 10h-6V4M14 10l7-7M3 21l7-7"/>',
  folder: '<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z"/>',
  sync: '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
  stop: '<rect x="5" y="5" width="14" height="14" rx="2"/>',
  pencil: '<path d="M21.17 6.81a1 1 0 0 0-3.99-3.99L3.84 16.17a2 2 0 0 0-.5.83l-1.32 4.35a.5.5 0 0 0 .62.62l4.35-1.32a2 2 0 0 0 .83-.5z"/><path d="m15 5 4 4"/>',
  trash: '<path d="M3 6h18M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>',
  sparkles: '<path d="M12 3l1.9 5.8a2 2 0 0 0 1.3 1.3L21 12l-5.8 1.9a2 2 0 0 0-1.3 1.3L12 21l-1.9-5.8a2 2 0 0 0-1.3-1.3L3 12l5.8-1.9a2 2 0 0 0 1.3-1.3z"/><path d="M5 3v4M3 5h4M19 17v4M17 19h4"/>',
  x: '<path d="M18 6 6 18M6 6l12 12"/>',
  thumbsUp: '<path d="M7 10v12"/><path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2a3.13 3.13 0 0 1 3 3.88Z"/>',
  comment: '<path d="M7.9 20A9 9 0 1 0 4 16.1L2 22Z"/>',
  link: '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>',
  chevronRight: '<path d="m9 18 6-6-6-6"/>',
  chevronLeft: '<path d="m15 18-6-6 6-6"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  checkCircle: '<circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/>',
  send: '<path d="M14.5 21.7a.5.5 0 0 0 .9 0l6.5-19a.5.5 0 0 0-.6-.6l-19 6.5a.5.5 0 0 0 0 .9l7.9 3.2a2 2 0 0 1 1.1 1.1z"/><path d="m21.9 2.1-11 11"/>',
  database: '<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5M3 12a9 3 0 0 0 18 0"/>',
  monitor: '<rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8M12 17v4"/>',
  info: '<circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/>',
  drive: '<path d="M22 12H2M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11zM6 16h.01M10 16h.01"/>',
  globe: '<circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20M2 12h20"/>',
  external: '<path d="M15 3h6v6M10 14 21 3M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
  eye: '<path d="M2.06 12.35a1 1 0 0 1 0-.7 10.75 10.75 0 0 1 19.88 0 1 1 0 0 1 0 .7 10.75 10.75 0 0 1-19.88 0"/><circle cx="12" cy="12" r="3"/>',
  youtube: '<path d="M2.5 17a24.12 24.12 0 0 1 0-10 2 2 0 0 1 1.4-1.4 49.56 49.56 0 0 1 16.2 0A2 2 0 0 1 21.5 7a24.12 24.12 0 0 1 0 10 2 2 0 0 1-1.4 1.4 49.55 49.55 0 0 1-16.2 0A2 2 0 0 1 2.5 17"/><path d="m10 15 5-3-5-3z"/>',
  rss: '<path d="M4 11a9 9 0 0 1 9 9M4 4a16 16 0 0 1 16 16"/><circle cx="5" cy="19" r="1"/>',
  headphones: '<path d="M3 14h3a2 2 0 0 1 2 2v3a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-7a9 9 0 0 1 18 0v7a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3"/>',
  languages: '<path d="m5 8 6 6M4 14l6-6 2-3M2 5h12M7 2h1M22 22l-5-10-5 10M14 18h6"/>',
};
const FILLED = new Set(['play', 'pause', 'prev', 'next']);

function icon(name, cls = '') {
  const t = document.createElement('template');
  const fill = FILLED.has(name) ? ' fill' : '';
  t.innerHTML = `<svg class="i${fill} ${cls}" viewBox="0 0 24 24" aria-hidden="true">${ICONS[name] || ''}</svg>`;
  return t.content.firstChild;
}

// -------------------------------------------------------------- i18n
// Interface text in Bangla and English. Topic and language names are
// translated too; content (titles, speakers) is shown as published.
const STR = {
  en: {
    appName: 'Noor Library', tagline: 'Islamic media library',
    home: 'Home', lectures: 'Lectures', series: 'Series', quran: 'Quran', films: 'Films & Docs', speakers: 'Speakers', topics: 'Topics',
    navGroups: { youtube: 'YouTube', quran: 'Quran reciters', web: 'Servers', feed: 'Podcasts', films: 'Films', folder: 'This PC' },
    navLibrary: 'Library', navSources: 'Sources', saved: 'Saved', recent: 'Continue listening', settings: 'Settings',
    addSource: 'Add a source', searchPh: 'Search lectures, speakers, topics…',
    season: 'Season', lecture: 'Lecture', seriesOne: 'Series', quranOne: 'Recitation', filmOne: 'Film',
    nLectures: (n) => `${n} lecture${n == 1 ? '' : 's'}`, nSeries: (n) => `${n} series`, nQuran: (n) => `${n} recitation${n == 1 ? '' : 's'}`, nFilms: (n) => `${n} film${n == 1 ? '' : 's'} & documentaries`,
    nItems: (n) => `${n} item${n == 1 ? '' : 's'}`, nResults: (n) => `${n} result${n == 1 ? '' : 's'}`, nSpeakers: (n) => `${n} speaker${n == 1 ? '' : 's'}`,
    nEpisodes: (n) => `${n} episode${n == 1 ? '' : 's'}`, nParts: (n) => `${n} part${n == 1 ? '' : 's'}`, nSurahs: (n) => `${n} surah${n == 1 ? '' : 's'}`,
    nLinks: (n) => `${n} source${n == 1 ? '' : 's'}`,
    seeAll: 'See all', anySpeaker: 'Any speaker', anyTopic: 'Any topic', anyLanguage: 'Any language', anyLength: 'Any length',
    anyMedia: 'Audio & video', allFolders: 'All folders', anyYear: 'Any year', allTypes: 'All types', sort: 'Sort', clear: 'Clear',
    short: 'Under 20 min', medium: '20–60 min', long: 'Over 1 hour', audio: 'Audio', video: 'Video', youtube: 'YouTube',
    sortNewest: 'Newest first', sortOldest: 'Oldest first', sortTitle: 'Title A–Z', sortLongest: 'Longest', sortShortest: 'Shortest',
    sortUpdated: 'Recently updated', sortSaved: 'Recently saved',
    noMatches: 'No matches', noMatchesText: 'Nothing fits these filters.', clearFilters: 'Clear filters',
    emptyTitle: 'Your library is empty', emptyText: 'Add a server folder, a folder on this PC, a YouTube channel or a podcast feed to get started.',
    jobPhase: { starting: 'starting', syncing: 'syncing', organising: 'organising' }, nFiles: (n) => `${num(n)} file${n == 1 ? '' : 's'}`, nFolders: (n) => `${num(n)} folders`,
    syncingTitle: 'Syncing your sources…', syncingText: 'Items appear as soon as each source finishes.',
    nothingYet: 'Nothing here yet', nothingYetText: 'Sync your sources to fill the library.', syncNow: 'Sync now',
    welcomeTitle: 'Welcome to Noor Library', welcomeText: 'Lectures, series and Quran recitations from your servers, folders, YouTube channels and podcasts, in one place.',
    ayahTr: '“Read in the name of your Lord who created.” (Al-‘Alaq 96:1)', startAdd: 'Add your first source',
    rowContinue: 'Continue listening', rowLectures: 'New lectures', rowSeries: 'Series', rowQuran: 'Quran recitations', rowFilms: 'Films & documentaries', rowSaved: 'Saved for later',
    rowSpeakers: 'Speakers', rowTopics: 'Browse by topic', rowTopic: (t) => t,
    play: 'Play', resume: 'Resume', playAll: 'Play all', playFrom: (x) => `Play ${x}`, save: 'Save', saveOn: 'Saved', openFolder: 'Open folder',
    nowPlaying: 'Playing', prevVideo: 'Previous', nextVideo: 'Next', nViews: (n) => `${n} views`, autoplay: 'Autoplay', moreFrom: (x) => `More from ${x}`, copyLink: 'Copy link', linkCopied: 'Link copied', episodeOf: (n, m) => `Episode ${n} of ${m}`, openPlayer: 'Open player', sortPopular: 'Most viewed',
    openYouTube: 'Open on YouTube', openIn: (p) => `Open in ${p}`, inPlayer: 'External player', watchHere: 'Play here',
    upNext: 'Up next', part: 'Part', episode: 'Episode', surah: 'Surah', verses: (n) => `${n} verses`, makki: 'Makki', madani: 'Madani',
    markWatched: 'Mark as finished', markUnwatched: 'Mark as not finished', more: 'More', less: 'Less', minLeft: (m) => `${m} min left`,
    backTo: (x) => x, speakerTitles: (n) => `${n} titles`, noEpisodes: 'Nothing playable found.',
    cantPlayHere: "This file can't play inside the app (usually MKV or HEVC video).", skipNext: 'Skip to next',
    opening: (x, p) => `Opening ${x} in ${p}`, openedYouTube: 'Opened on YouTube', speed: 'Speed', close: 'Close',
    nothingPlayed: 'Nothing played yet', nothingPlayedText: 'What you listen to shows up here, newest first.',
    nothingSaved: 'Nothing saved yet', nothingSavedText: 'Use the bookmark on any card to save it for later.',
    resultsFor: (q) => `Results for “${q}”`, searchSpeakers: 'Search speakers…',
    // settings
    tabSources: 'Sources', tabPlayer: 'Player', tabYouTube: 'YouTube', tabAssistant: 'AI assistant', tabLanguage: 'Language', tabLibrary: 'Library & storage',
    addLink: 'Add a source', addLead: 'Paste a server folder link, a YouTube channel or playlist, or a podcast feed — or choose a folder on this PC.',
    location: 'Link or folder', locationHint: 'Everything inside is scanned.', type: 'Type', name: 'Name', nameHint: 'Shown in the sidebar.',
    speakerField: 'Speaker (optional)', speakerHint: 'For a source with one speaker.', speakerFolders: 'Folders inside are speaker names (Speaker/lecture.mp3)',
    checkLink: 'Check link', addAndSync: 'Add and sync', checking: 'Checking…',
    kindLectures: 'Lectures', kindSeries: 'Series', kindQuran: 'Quran', kindFilms: 'Films',
    kindHint: { lectures: 'Each file (or folder) is one talk', series: 'Folders of numbered parts or episodes', quran: 'Reciter folders with surah files', films: 'A folder per film or documentary' },
    checkOk: (o, n, f) => `✓ ${o}: ${n}${f ? `, ${f} folders` : ''}.`, originNames: { web: 'Server folder', folder: 'Folder on this PC', youtube: 'YouTube', feed: 'Podcast feed' },
    looksLike: (k) => `Looks like: ${k}.`, added: (n) => `Added “${n}”. Syncing now — items appear when it finishes.`,
    yourSources: 'Your sources', fullRescan: 'Full rescan', syncAll: 'Sync all', syncing: 'Syncing…', sync: 'Sync', stop: 'Stop', edit: 'Edit', del: 'Delete', cancel: 'Cancel', saveBtn: 'Save', savedOk: 'Saved.',
    queued: 'Queued', neverSynced: 'Never synced', syncedAgo: (x) => `Synced ${x}`, stoppedEarly: 'stopped before finishing',
    justNow: 'just now', minAgo: (n) => `${n} min ago`, hAgo: (n) => `${n} h ago`, dAgo: (n) => `${n} d ago`,
    noSources: 'No sources yet. Add one above.', activity: 'Activity', activityLead: 'What the last syncs did.', nothing: 'Nothing yet.',
    deleteQ: (n) => `Delete “${n}”?`, deleteText: 'Its items leave your library (saved items and progress too). Nothing on the server or disk is touched.',
    rescanQ: 'Full rescan', rescanText: 'Re-read every source from scratch. Items that are gone are removed. Large sources take a while.', rescanOk: 'Rescan everything',
    playerTitle: 'Player', playerLead: 'Audio, MP4/WebM video and YouTube play inside the app. Other video (MKV, AVI…) opens in this program.',
    program: 'Program', programHint: 'Full path to vlc.exe, mpv.exe or another player.', args: 'Arguments', argsHint: '{url} becomes the link or file path.',
    externalOnly: 'Always use the external player (except YouTube)', browse: 'Browse…', getVlc: 'Get VLC',
    ytTitle: 'YouTube channels and playlists', ytLead: 'Noor Library reads channels and playlists through the free YouTube Data API. It needs a key (about two minutes to get).',
    ytSteps: ['Open Google Cloud Console and create a project (any name).', 'Enable “YouTube Data API v3” for it.', 'Create credentials → API key, and paste it here.'],
    ytKey: 'YouTube API key', testKey: 'Test key', keyWorks: '✓ The key works.', openConsole: 'Open Google Cloud Console',
    asTitle: 'Watch assistant', asLead: 'Ask in Bangla or English — “a short Bangla lecture on patience”, “Surah Al-Kahf recitation”. It uses Google Gemini, recommends only what is in your library, and never gives religious rulings.',
    geminiKey: 'Gemini API key', freeFrom: 'Free from', model: 'Model', loadList: 'Load list', modelsOk: (n) => `✓ Key works. ${n} chat models — pick one in the Model box.`, clearChat: 'Clear chat history',
    clearChatQ: 'Clear chat history?', clearChatText: 'The assistant forgets the conversation so far.', chatCleared: 'Chat cleared',
    langTitle: 'Language', langLead: 'Language of menus and buttons. Lectures keep their own titles.',
    libTitle: 'Library', scanning: 'Scanning', scanningLead: 'Applies to the next sync.', skipFolders: 'Skip folders whose name contains', commaHint: 'Comma-separated, not case-sensitive.',
    fileTypes: 'Audio and video file types', parallel: 'Parallel requests', pause: 'Pause between levels (ms)', timeout: 'Timeout (s)', depth: 'Max folder depth',
    maintenance: 'Maintenance', maintenanceLead: 'None of these touch your media files.', rebuild: 'Rebuild library', rebuildText: 'Regroup every source again. Saved items and progress are kept.',
    clearHistory: 'Clear listening history', clearHistoryText: 'Forget what you played, your progress and finished marks.', reset: 'Reset library',
    resetText: 'Empty the whole library: items, history, saved and chat. Your sources stay; sync them to rebuild.', resetOk: 'Reset everything', done: 'Done',
    openData: 'Open data folder', dataFolder: 'Data folder', settingsFile: 'Settings file', database: 'Database',
    statLectures: 'lectures', statSeries: 'series', statQuran: 'recitations', statFilms: 'films', statSpeakers: 'speakers', statFiles: 'files', statHours: 'hours of audio/video', statDb: 'database',
    // assistant
    asSub: (m) => `Finds picks in your library · ${m} · chat is saved`, asSetupTitle: 'Set up the assistant', asSetupText: 'Add a free Google Gemini API key and the assistant will suggest lectures from your own library.',
    asOpenSettings: 'Open assistant settings', asHello: 'What would you like to listen to?',
    asHelloText: (l, s, q) => `Ask for a topic, a speaker, a length or a mood. I'll pick from your ${l} lectures, ${s} series and ${q} recitations.`,
    asPh: 'e.g. a short lecture on patience…',
    suggestions: ['A short Bangla lecture on patience', 'Seerah series for beginners', 'Surah Al-Kahf recitation', 'Talks about family and marriage'],
    stopping: 'Stopping…', selectFirst: 'Choose a folder', notConnected: 'Not connected', somethingWrong: 'Something went wrong',
  },
  bn: {
    appName: 'নূর লাইব্রেরি', tagline: 'ইসলামিক মিডিয়া লাইব্রেরি',
    home: 'হোম', lectures: 'লেকচার', series: 'সিরিজ', quran: 'কুরআন', films: 'চলচ্চিত্র ও ডকুমেন্টারি', speakers: 'বক্তা', topics: 'বিষয়',
    navGroups: { youtube: 'ইউটিউব', quran: 'ক্বারী', web: 'সার্ভার', feed: 'পডকাস্ট', films: 'চলচ্চিত্র', folder: 'এই পিসি' },
    navLibrary: 'লাইব্রেরি', navSources: 'উৎস', saved: 'সংরক্ষিত', recent: 'আবার শুনুন', settings: 'সেটিংস',
    addSource: 'উৎস যোগ করুন', searchPh: 'লেকচার, বক্তা, বিষয় খুঁজুন…',
    season: 'সিজন', lecture: 'লেকচার', seriesOne: 'সিরিজ', quranOne: 'তিলাওয়াত', filmOne: 'চলচ্চিত্র',
    nLectures: (n) => `${n}টি লেকচার`, nSeries: (n) => `${n}টি সিরিজ`, nQuran: (n) => `${n}টি তিলাওয়াত`, nFilms: (n) => `${n}টি চলচ্চিত্র ও ডকুমেন্টারি`,
    nItems: (n) => `${n}টি আইটেম`, nResults: (n) => `${n}টি ফলাফল`, nSpeakers: (n) => `${n} জন বক্তা`,
    nEpisodes: (n) => `${n}টি পর্ব`, nParts: (n) => `${n}টি অংশ`, nSurahs: (n) => `${n}টি সূরা`,
    nLinks: (n) => `${n}টি উৎস`,
    seeAll: 'সব দেখুন', anySpeaker: 'সব বক্তা', anyTopic: 'সব বিষয়', anyLanguage: 'সব ভাষা', anyLength: 'যেকোনো দৈর্ঘ্য',
    anyMedia: 'অডিও ও ভিডিও', allFolders: 'সব ফোল্ডার', anyYear: 'সব বছর', allTypes: 'সব ধরন', sort: 'সাজান', clear: 'মুছুন',
    short: '২০ মিনিটের কম', medium: '২০–৬০ মিনিট', long: '১ ঘণ্টার বেশি', audio: 'অডিও', video: 'ভিডিও', youtube: 'ইউটিউব',
    sortNewest: 'নতুন আগে', sortOldest: 'পুরনো আগে', sortTitle: 'নাম অনুযায়ী', sortLongest: 'দীর্ঘতম', sortShortest: 'সংক্ষিপ্ততম',
    sortUpdated: 'সম্প্রতি হালনাগাদ', sortSaved: 'সম্প্রতি সংরক্ষিত',
    noMatches: 'কিছু পাওয়া যায়নি', noMatchesText: 'এই ফিল্টারে কিছু মেলে না।', clearFilters: 'ফিল্টার মুছুন',
    emptyTitle: 'আপনার লাইব্রেরি খালি', emptyText: 'শুরু করতে সার্ভারের ফোল্ডার, এই পিসির ফোল্ডার, ইউটিউব চ্যানেল বা পডকাস্ট ফিড যোগ করুন।',
    jobPhase: { starting: 'শুরু হচ্ছে', syncing: 'সিঙ্ক হচ্ছে', organising: 'সাজানো হচ্ছে' }, nFiles: (n) => `${num(n)}টি ফাইল`, nFolders: (n) => `${num(n)}টি ফোল্ডার`,
    syncingTitle: 'উৎসগুলো সিঙ্ক হচ্ছে…', syncingText: 'প্রতিটি উৎস শেষ হলেই আইটেম দেখা যাবে।',
    nothingYet: 'এখনও কিছু নেই', nothingYetText: 'লাইব্রেরি ভরাতে উৎসগুলো সিঙ্ক করুন।', syncNow: 'এখন সিঙ্ক করুন',
    welcomeTitle: 'নূর লাইব্রেরিতে স্বাগতম', welcomeText: 'আপনার সার্ভার, ফোল্ডার, ইউটিউব চ্যানেল ও পডকাস্টের লেকচার, সিরিজ ও কুরআন তিলাওয়াত — এক জায়গায়।',
    ayahTr: '“পড়ো তোমার প্রতিপালকের নামে, যিনি সৃষ্টি করেছেন।” (সূরা আল-আলাক ৯৬:১)', startAdd: 'প্রথম উৎস যোগ করুন',
    rowContinue: 'আবার শুনুন', rowLectures: 'নতুন লেকচার', rowSeries: 'সিরিজ', rowQuran: 'কুরআন তিলাওয়াত', rowFilms: 'চলচ্চিত্র ও ডকুমেন্টারি', rowSaved: 'পরে শোনার জন্য সংরক্ষিত',
    rowSpeakers: 'বক্তা', rowTopics: 'বিষয় অনুযায়ী দেখুন', rowTopic: (t) => t,
    play: 'চালান', resume: 'আবার শুরু', playAll: 'সব চালান', playFrom: (x) => `${x} চালান`, save: 'সংরক্ষণ', saveOn: 'সংরক্ষিত', openFolder: 'ফোল্ডার খুলুন',
    nowPlaying: 'চলছে', prevVideo: 'আগের', nextVideo: 'পরের', nViews: (n) => `${n} বার দেখা`, autoplay: 'অটোপ্লে', moreFrom: (x) => `${x} থেকে আরও`, copyLink: 'লিংক কপি করুন', linkCopied: 'লিংক কপি হয়েছে', episodeOf: (n, m) => `পর্ব ${n} / ${m}`, openPlayer: 'প্লেয়ার খুলুন', sortPopular: 'সবচেয়ে বেশি দেখা',
    openYouTube: 'ইউটিউবে খুলুন', openIn: (p) => `${p}-এ খুলুন`, inPlayer: 'বাইরের প্লেয়ার', watchHere: 'এখানে চালান',
    upNext: 'পরবর্তী', part: 'অংশ', episode: 'পর্ব', surah: 'সূরা', verses: (n) => `${n} আয়াত`, makki: 'মাক্কী', madani: 'মাদানী',
    markWatched: 'শেষ হয়েছে চিহ্নিত করুন', markUnwatched: 'শেষ হয়নি চিহ্নিত করুন', more: 'আরও', less: 'কম', minLeft: (m) => `${m} মিনিট বাকি`,
    backTo: (x) => x, speakerTitles: (n) => `${n}টি আইটেম`, noEpisodes: 'চালানোর মতো কিছু পাওয়া যায়নি।',
    cantPlayHere: 'এই ফাইলটি অ্যাপের ভেতরে চলে না (সাধারণত MKV বা HEVC ভিডিও)।', skipNext: 'পরেরটিতে যান',
    opening: (x, p) => `${p}-এ খোলা হচ্ছে: ${x}`, openedYouTube: 'ইউটিউবে খোলা হয়েছে', speed: 'গতি', close: 'বন্ধ',
    nothingPlayed: 'এখনও কিছু শোনা হয়নি', nothingPlayedText: 'আপনি যা শোনেন তা এখানে দেখা যাবে, নতুনটি আগে।',
    nothingSaved: 'এখনও কিছু সংরক্ষিত নেই', nothingSavedText: 'যেকোনো কার্ডের বুকমার্ক চাপুন, পরে শোনার জন্য সংরক্ষিত হবে।',
    resultsFor: (q) => `“${q}”-এর ফলাফল`, searchSpeakers: 'বক্তা খুঁজুন…',
    tabSources: 'উৎস', tabPlayer: 'প্লেয়ার', tabYouTube: 'ইউটিউব', tabAssistant: 'এআই সহকারী', tabLanguage: 'ভাষা', tabLibrary: 'লাইব্রেরি ও স্টোরেজ',
    addLink: 'উৎস যোগ করুন', addLead: 'সার্ভারের ফোল্ডার লিংক, ইউটিউব চ্যানেল বা প্লেলিস্ট, অথবা পডকাস্ট ফিড দিন — অথবা এই পিসির একটি ফোল্ডার বেছে নিন।',
    location: 'লিংক বা ফোল্ডার', locationHint: 'ভেতরের সবকিছু স্ক্যান করা হবে।', type: 'ধরন', name: 'নাম', nameHint: 'সাইডবারে দেখাবে।',
    speakerField: 'বক্তা (ঐচ্ছিক)', speakerHint: 'যে উৎসে একজনই বক্তা।', speakerFolders: 'ভেতরের ফোল্ডারগুলো বক্তাদের নাম (বক্তা/লেকচার.mp3)',
    checkLink: 'লিংক যাচাই', addAndSync: 'যোগ করে সিঙ্ক করুন', checking: 'যাচাই হচ্ছে…',
    kindLectures: 'লেকচার', kindSeries: 'সিরিজ', kindQuran: 'কুরআন', kindFilms: 'চলচ্চিত্র',
    kindHint: { lectures: 'প্রতিটি ফাইল (বা ফোল্ডার) একটি লেকচার', series: 'ক্রমিক অংশ বা পর্বের ফোল্ডার', quran: 'ক্বারীর ফোল্ডারে সূরার ফাইল' , films: 'প্রতিটি চলচ্চিত্র বা ডকুমেন্টারির আলাদা ফোল্ডার' },
    checkOk: (o, n, f) => `✓ ${o}: ${n}${f ? `, ${f}টি ফোল্ডার` : ''}।`, originNames: { web: 'সার্ভারের ফোল্ডার', folder: 'এই পিসির ফোল্ডার', youtube: 'ইউটিউব', feed: 'পডকাস্ট ফিড' },
    looksLike: (k) => `মনে হচ্ছে: ${k}।`, added: (n) => `“${n}” যোগ হয়েছে। এখন সিঙ্ক হচ্ছে — শেষ হলে আইটেম দেখা যাবে।`,
    yourSources: 'আপনার উৎসসমূহ', fullRescan: 'সম্পূর্ণ রিস্ক্যান', syncAll: 'সব সিঙ্ক', syncing: 'সিঙ্ক হচ্ছে…', sync: 'সিঙ্ক', stop: 'থামান', edit: 'সম্পাদনা', del: 'মুছুন', cancel: 'বাতিল', saveBtn: 'সংরক্ষণ', savedOk: 'সংরক্ষিত হয়েছে।',
    queued: 'অপেক্ষমাণ', neverSynced: 'কখনও সিঙ্ক হয়নি', syncedAgo: (x) => `সিঙ্ক হয়েছে ${x}`, stoppedEarly: 'শেষ হওয়ার আগে থামানো হয়েছে',
    justNow: 'এইমাত্র', minAgo: (n) => `${n} মিনিট আগে`, hAgo: (n) => `${n} ঘণ্টা আগে`, dAgo: (n) => `${n} দিন আগে`,
    noSources: 'এখনও কোনো উৎস নেই। উপরে যোগ করুন।', activity: 'কার্যক্রম', activityLead: 'সাম্প্রতিক সিঙ্কে কী হয়েছে।', nothing: 'এখনও কিছু নেই।',
    deleteQ: (n) => `“${n}” মুছবেন?`, deleteText: 'এর আইটেমগুলো লাইব্রেরি থেকে সরে যাবে (সংরক্ষণ ও অগ্রগতিসহ)। সার্ভার বা ডিস্কের কিছুই মুছবে না।',
    rescanQ: 'সম্পূর্ণ রিস্ক্যান', rescanText: 'সব উৎস নতুন করে পড়া হবে। যেগুলো আর নেই সেগুলো সরানো হবে। বড় উৎসে সময় লাগে।', rescanOk: 'সব রিস্ক্যান করুন',
    playerTitle: 'প্লেয়ার', playerLead: 'অডিও, MP4/WebM ভিডিও ও ইউটিউব অ্যাপের ভেতরেই চলে। অন্য ভিডিও (MKV, AVI…) এই প্রোগ্রামে খুলবে।',
    program: 'প্রোগ্রাম', programHint: 'vlc.exe, mpv.exe বা অন্য প্লেয়ারের পুরো ঠিকানা।', args: 'আর্গুমেন্ট', argsHint: '{url} এর জায়গায় লিংক বা ফাইলের ঠিকানা বসবে।',
    externalOnly: 'সবসময় বাইরের প্লেয়ার ব্যবহার করুন (ইউটিউব ছাড়া)', browse: 'ব্রাউজ…', getVlc: 'VLC নামান',
    ytTitle: 'ইউটিউব চ্যানেল ও প্লেলিস্ট', ytLead: 'নূর লাইব্রেরি ফ্রি YouTube Data API দিয়ে চ্যানেল ও প্লেলিস্ট পড়ে। এর জন্য একটি কী লাগে (দুই মিনিটে পাওয়া যায়)।',
    ytSteps: ['Google Cloud Console খুলে একটি প্রজেক্ট তৈরি করুন (যেকোনো নাম)।', 'প্রজেক্টে “YouTube Data API v3” চালু করুন।', 'Credentials → API key তৈরি করে এখানে পেস্ট করুন।'],
    ytKey: 'ইউটিউব API কী', testKey: 'কী যাচাই', keyWorks: '✓ কী কাজ করছে।', openConsole: 'Google Cloud Console খুলুন',
    asTitle: 'সহকারী', asLead: 'বাংলা বা ইংরেজিতে জিজ্ঞেস করুন — “ধৈর্য নিয়ে ছোট বাংলা লেকচার”, “সূরা কাহফ তিলাওয়াত”। এটি Google Gemini ব্যবহার করে, শুধু আপনার লাইব্রেরি থেকে সাজেস্ট করে এবং কোনো ফতোয়া দেয় না।',
    geminiKey: 'Gemini API কী', freeFrom: 'ফ্রি পাবেন', model: 'মডেল', loadList: 'তালিকা আনুন', modelsOk: (n) => `✓ কী কাজ করছে। ${n}টি চ্যাট মডেল — মডেল বক্সে একটি বেছে নিন।`, clearChat: 'চ্যাট ইতিহাস মুছুন',
    clearChatQ: 'চ্যাট ইতিহাস মুছবেন?', clearChatText: 'সহকারী এ পর্যন্ত কথোপকথন ভুলে যাবে।', chatCleared: 'চ্যাট মুছে ফেলা হয়েছে',
    langTitle: 'ভাষা', langLead: 'মেনু ও বাটনের ভাষা। লেকচারের নাম যেমন আছে তেমনই থাকবে।',
    libTitle: 'লাইব্রেরি', scanning: 'স্ক্যানিং', scanningLead: 'পরের সিঙ্ক থেকে কার্যকর।', skipFolders: 'যে ফোল্ডারের নামে এগুলো আছে বাদ দিন', commaHint: 'কমা দিয়ে আলাদা করুন, বড়-ছোট হাতের অক্ষরে পার্থক্য নেই।',
    fileTypes: 'অডিও ও ভিডিও ফাইলের ধরন', parallel: 'একসাথে অনুরোধ', pause: 'স্তরের মাঝে বিরতি (ms)', timeout: 'টাইমআউট (সেকেন্ড)', depth: 'সর্বোচ্চ ফোল্ডার গভীরতা',
    maintenance: 'রক্ষণাবেক্ষণ', maintenanceLead: 'এগুলোর কোনোটিই আপনার মিডিয়া ফাইলে হাত দেয় না।', rebuild: 'লাইব্রেরি পুনর্গঠন', rebuildText: 'সব উৎস আবার সাজানো হবে। সংরক্ষিত ও অগ্রগতি থাকবে।',
    clearHistory: 'শোনার ইতিহাস মুছুন', clearHistoryText: 'যা শুনেছেন, অগ্রগতি ও শেষ-চিহ্ন মুছে যাবে।', reset: 'লাইব্রেরি রিসেট',
    resetText: 'পুরো লাইব্রেরি খালি হবে: আইটেম, ইতিহাস, সংরক্ষিত ও চ্যাট। উৎসগুলো থাকবে; আবার সিঙ্ক করলে তৈরি হবে।', resetOk: 'সব রিসেট করুন', done: 'সম্পন্ন',
    openData: 'ডেটা ফোল্ডার খুলুন', dataFolder: 'ডেটা ফোল্ডার', settingsFile: 'সেটিংস ফাইল', database: 'ডেটাবেস',
    statLectures: 'লেকচার', statSeries: 'সিরিজ', statQuran: 'তিলাওয়াত', statFilms: 'চলচ্চিত্র', statSpeakers: 'বক্তা', statFiles: 'ফাইল', statHours: 'ঘণ্টা অডিও/ভিডিও', statDb: 'ডেটাবেস',
    asSub: (m) => `আপনার লাইব্রেরি থেকে খুঁজে দেয় · ${m} · চ্যাট সংরক্ষিত থাকে`, asSetupTitle: 'সহকারী চালু করুন', asSetupText: 'একটি ফ্রি Google Gemini API কী দিন, সহকারী আপনার লাইব্রেরি থেকে লেকচার সাজেস্ট করবে।',
    asOpenSettings: 'সহকারীর সেটিংস খুলুন', asHello: 'আজ কী শুনতে চান?',
    asHelloText: (l, s, q) => `বিষয়, বক্তা, সময় বা মন অনুযায়ী জিজ্ঞেস করুন। আপনার ${l}টি লেকচার, ${s}টি সিরিজ ও ${q}টি তিলাওয়াত থেকে বেছে দেব।`,
    asPh: 'যেমন: ধৈর্য নিয়ে ছোট একটি লেকচার…',
    suggestions: ['ধৈর্য নিয়ে ছোট একটি বাংলা লেকচার', 'নতুনদের জন্য সীরাত সিরিজ', 'সূরা কাহফ তিলাওয়াত', 'পরিবার ও দাম্পত্য নিয়ে আলোচনা'],
    stopping: 'থামানো হচ্ছে…', selectFirst: 'একটি ফোল্ডার বেছে নিন', notConnected: 'সংযোগ নেই', somethingWrong: 'কিছু একটা ভুল হয়েছে',
  },
};

// Topic keys from the server, with names, a calligraphy glyph and colours.
const TOPICS = {
  Quran: { en: 'Quran', bn: 'কুরআন', g: 'ق', c: ['#0f5a44', '#0a2a21'] },
  Tafsir: { en: 'Tafsir', bn: 'তাফসীর', g: 'ت', c: ['#145a5a', '#0b2627'] },
  Hadith: { en: 'Hadith', bn: 'হাদিস', g: 'ح', c: ['#4b3a1a', '#1f180b'] },
  Seerah: { en: 'Seerah', bn: 'সীরাত', g: 'س', c: ['#5a3d14', '#24190a'] },
  Prophets: { en: 'Stories of the Prophets', bn: 'নবীদের কাহিনী', g: 'ن', c: ['#3b3f7a', '#171930'] },
  Sahaba: { en: 'Sahaba', bn: 'সাহাবী', g: 'ص', c: ['#42305e', '#1b1427'] },
  Aqeedah: { en: 'Aqeedah (Belief)', bn: 'আকিদা', g: 'ع', c: ['#1d4f6e', '#0c2130'] },
  Salah: { en: 'Prayer (Salah)', bn: 'নামাজ', g: 'ص', c: ['#16614a', '#0a2a20'] },
  Ramadan: { en: 'Ramadan & Fasting', bn: 'রমজান ও রোজা', g: 'ر', c: ['#5e2f5a', '#271326'] },
  Hajj: { en: 'Hajj & Umrah', bn: 'হজ ও উমরা', g: 'ح', c: ['#3d3d3d', '#151515'] },
  Zakat: { en: 'Zakat & Charity', bn: 'যাকাত ও সদকা', g: 'ز', c: ['#5a5414', '#24210a'] },
  Dua: { en: 'Dua & Dhikr', bn: 'দোয়া ও জিকির', g: 'د', c: ['#1d5a6e', '#0b2430'] },
  Family: { en: 'Family & Marriage', bn: 'পরিবার ও বিবাহ', g: 'أ', c: ['#6e3a2a', '#2c1710'] },
  Parenting: { en: 'Parenting', bn: 'সন্তান প্রতিপালন', g: 'و', c: ['#6e5a2a', '#2c2410'] },
  Youth: { en: 'Youth', bn: 'তরুণদের জন্য', g: 'ش', c: ['#2a5e6e', '#10262c'] },
  Women: { en: 'Women', bn: 'নারী', g: 'ن', c: ['#6e2a4f', '#2c1020'] },
  Akhirah: { en: 'Death & Hereafter', bn: 'মৃত্যু ও আখিরাত', g: 'آ', c: ['#33384a', '#14161d'] },
  Jannah: { en: 'Jannah & Jahannam', bn: 'জান্নাত ও জাহান্নাম', g: 'ج', c: ['#1f6e3f', '#0c2c19'] },
  Character: { en: 'Character (Akhlaq)', bn: 'চরিত্র ও আখলাক', g: 'خ', c: ['#4f6e2a', '#202c10'] },
  Patience: { en: 'Patience & Gratitude', bn: 'ধৈর্য ও কৃতজ্ঞতা', g: 'ص', c: ['#2a4f6e', '#10202c'] },
  Tawbah: { en: 'Repentance', bn: 'তওবা', g: 'ت', c: ['#3a2a6e', '#17102c'] },
  Fiqh: { en: 'Fiqh & Rulings', bn: 'ফিকহ ও মাসআলা', g: 'ف', c: ['#5a4632', '#241c14'] },
  Business: { en: 'Halal Earning', bn: 'হালাল উপার্জন', g: 'ر', c: ['#2a6e5a', '#102c24'] },
  History: { en: 'Islamic History', bn: 'ইসলামের ইতিহাস', g: 'ت', c: ['#6e4a2a', '#2c1e10'] },
  Science: { en: 'Islam & Science', bn: 'ইসলাম ও বিজ্ঞান', g: 'ع', c: ['#2a3f6e', '#10192c'] },
  Dawah: { en: 'Dawah', bn: 'দাওয়াহ', g: 'د', c: ['#6e2a2a', '#2c1010'] },
  Kids: { en: 'For Kids', bn: 'শিশুদের জন্য', g: 'ط', c: ['#6e5f2a', '#2c2610'] },
  Nasheed: { en: 'Nasheed & Naat', bn: 'নাশিদ ও গজল', g: 'ن', c: ['#4a2a6e', '#1e102c'] },
  Waz: { en: 'Waz & Khutbah', bn: 'ওয়াজ ও খুতবা', g: 'و', c: ['#2a6e4a', '#102c1e'] },
  Arabic: { en: 'Learn Arabic', bn: 'আরবি শিক্ষা', g: 'ض', c: ['#5a2a6e', '#24102c'] },
};
const LANGS = { bn: { en: 'Bangla', bn: 'বাংলা' }, en: { en: 'English', bn: 'ইংরেজি' }, ar: { en: 'Arabic', bn: 'আরবি' }, ur: { en: 'Urdu', bn: 'উর্দু' }, hi: { en: 'Hindi', bn: 'হিন্দি' } };

let lang = 'bn';
try { lang = localStorage.getItem('noor.lang') || 'bn'; } catch { /* storage may be blocked */ }
function t(key, ...args) {
  const v = STR[lang]?.[key] ?? STR.en[key];
  if (typeof v === 'function') return v(...args.map((a) => (typeof a === 'number' ? num(a) : a)));
  return v ?? key;
}
const topicName = (k) => TOPICS[k]?.[lang] || k;
const langName = (k) => LANGS[k]?.[lang] || k;

// ----------------------------------------------------------- helpers
const $ = (sel, root = document) => root.querySelector(sel);
const page = $('#page');

/** Element builder: h('div', {class, text, onclick, ...attrs}, ...children). */
function h(tag, props, ...kids) {
  const el = document.createElement(tag);
  let value;
  if (props) {
    for (const [k, v] of Object.entries(props)) {
      if (v == null || v === false) continue;
      if (k === 'class') el.className = v;
      else if (k === 'text') el.textContent = v;
      else if (k === 'value') value = v;
      else if (k === 'style') el.style.cssText = v;
      else if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2), v);
      else if (v === true) el.setAttribute(k, '');
      else el.setAttribute(k, v);
    }
  }
  for (const kid of kids.flat(Infinity)) {
    if (kid == null || kid === false) continue;
    el.append(kid.nodeType ? kid : document.createTextNode(String(kid)));
  }
  if (value !== undefined) el.value = value;
  return el;
}
/** replaceChildren that skips null/false, like h() does. */
function fill(el, ...kids) {
  el.replaceChildren(...kids.flat(Infinity).filter((k) => k != null && k !== false));
  return el;
}

async function api(method, path, body) {
  const opts = { method, headers: {} };
  if (body !== undefined) { opts.headers['Content-Type'] = 'application/json'; opts.body = JSON.stringify(body); }
  const r = await fetch(path, opts);
  const d = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(d.error || r.statusText || 'Request failed');
  return d;
}
const GET = (p) => api('GET', p);
const POST = (p, b) => api('POST', p, b ?? {});
const PUT = (p, b) => api('PUT', p, b);
const DEL = (p) => api('DELETE', p);
function qs(obj) {
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(obj)) if (v !== undefined && v !== null && v !== '' && v !== false) p.set(k, v);
  return p.toString();
}
function toast(msg, err = false) {
  const el = h('div', { class: 'toast' + (err ? ' err' : ''), text: msg });
  $('#toasts').append(el);
  setTimeout(() => el.remove(), err ? 6000 : 3200);
}

const nf = () => new Intl.NumberFormat(lang === 'bn' ? 'bn-BD' : 'en-US');
const num = (n) => nf().format(Number(n || 0));
const digits = (s) => (lang === 'bn' ? String(s).replace(/\d/g, (d) => '০১২৩৪৫৬৭৮৯'[d]) : String(s));
const pad = (n) => String(n).padStart(2, '0');
/** 3723 -> "1:02:03". */
function clock(sec) {
  if (sec == null || !isFinite(sec)) return '';
  sec = Math.max(0, Math.round(sec));
  const h_ = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60), s = sec % 60;
  return digits(h_ ? `${h_}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`);
}
/** 3723 -> "1 h 2 min" / "১ ঘণ্টা ২ মিনিট". */
function human(sec) {
  if (!sec) return '';
  const h_ = Math.floor(sec / 3600), m = Math.round((sec % 3600) / 60);
  if (lang === 'bn') return [h_ && `${num(h_)} ঘণ্টা`, m && `${num(m)} মিনিট`].filter(Boolean).join(' ');
  return [h_ && `${h_} h`, m && `${m} min`].filter(Boolean).join(' ');
}
function size(b) {
  if (b == null) return '';
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (b >= 1024 && i < u.length - 1) { b /= 1024; i++; }
  return digits((i >= 3 ? b.toFixed(1) : Math.round(b)) + ' ') + u[i];
}
function ago(ts) {
  if (!ts) return t('neverSynced');
  const s = Math.max(0, Date.now() / 1000 - ts);
  const x = s < 60 ? t('justNow') : s < 3600 ? t('minAgo', Math.round(s / 60)) : s < 86400 ? t('hAgo', Math.round(s / 3600)) : t('dAgo', Math.round(s / 86400));
  return t('syncedAgo', x);
}
function fmtDate(ts) {
  if (!ts) return '';
  return new Date(ts * 1000).toLocaleDateString(lang === 'bn' ? 'bn-BD' : 'en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}
const go = (hash) => { location.hash = hash; };
function hashColor(s) {
  let x = 0;
  for (const c of s || '') x = (x * 31 + c.charCodeAt(0)) | 0;
  const hue = Math.abs(x) % 360;
  return `linear-gradient(135deg, hsl(${hue} 45% 32%), hsl(${(hue + 40) % 360} 50% 16%))`;
}
// Titles such as "Dr." or "শায়খ" are skipped, so "Sh. Elshinawy" reads "E".
const HONORIFICS = new Set(['dr.', 'dr', 'sh.', 'sh', 'shaykh', 'sheikh', 'shaikh', 'imam', 'ustadh', 'ustadha', 'ustaz', 'mufti', 'maulana', 'mawlana', 'hafiz', 'qari', 'prof.', 'শায়খ', 'শাইখ', 'শায়েখ', 'ড.', 'ডক্টর', 'মাওলানা', 'মুফতি', 'হাফেজ', 'ক্বারী', 'কারী', 'ইমাম', 'উস্তাদ']);
const initials = (name) => {
  const words = (name || '?').split(/\s+/).filter(Boolean);
  const plain = words.filter((w) => !HONORIFICS.has(w.toLowerCase()));
  return (plain.length ? plain : words).slice(0, 2).map((w) => [...w][0]).join('').toUpperCase();
};
const ext = (name) => (String(name).split(/[?#]/)[0].split('.').pop() || '').toLowerCase();
const AUDIO = new Set(['mp3', 'm4a', 'aac', 'ogg', 'opus', 'wav', 'flac']);
const VIDEO_OK = new Set(['mp4', 'm4v', 'webm', 'mov', '3gp']);
function ytId(url) {
  try {
    const u = new URL(url);
    if (u.hostname === 'youtu.be') return u.pathname.slice(1);
    if (u.hostname.endsWith('youtube.com')) return u.searchParams.get('v');
  } catch { /* not a URL */ }
  return null;
}
function playerName() {
  const cmd = app.state?.flags?.player || 'player';
  const base = cmd.split(/[\\/]/).pop().replace(/\.exe$/i, '');
  return base.toLowerCase() === 'vlc' ? 'VLC' : base;
}

// --------------------------------------------------------------- app
const app = { state: null, wasBusy: false, pageCtl: null, gridMemo: {}, detailSeason: {}, editing: null, navCount: 0 };

function applyStaticText() {
  document.documentElement.lang = lang;
  for (const el of document.querySelectorAll('[data-t]')) el.textContent = t(el.dataset.t);
  $('#search').placeholder = t('searchPh');
  $('#lang-toggle').textContent = lang === 'bn' ? 'English' : 'বাংলা';
  $('#lang-toggle').title = t('tabLanguage');
  $('#add-source').title = t('addSource');
  $('#top-settings').title = t('settings');
  $('#fab').title = t('asTitle');
}

async function setLang(l) {
  lang = l;
  try { localStorage.setItem('noor.lang', l); } catch { /* ignore */ }
  applyStaticText();
  renderSidebar();
  if (app.state) renderJob();
  PUT('/api/settings/ui', { language: l }).catch(() => {});
  if (assistant.open) renderAssistant();
  player.render();
  route();
}

// ----------------------------------------------------------- sidebar
function navItem(href, iconName, label, count, extra) {
  return h('a', { class: 'nav-item', href, 'data-href': href },
    icon(iconName), h('span', { class: 'label', text: label }), extra,
    count ? h('span', { class: 'count', text: num(count) }) : null);
}
const ORIGIN_ICON = { web: 'globe', folder: 'folder', youtube: 'youtube', feed: 'rss' };
function renderSidebar() {
  const c = app.state?.counts || {};
  fill($('#nav-main'),
    navItem('#/', 'home', t('home')),
    navItem('#/lectures', 'mic', t('lectures'), c.lectures),
    navItem('#/series', 'layers', t('series'), c.series),
    navItem('#/quran', 'book', t('quran'), c.quran),
    c.films ? navItem('#/films', 'film', t('films'), c.films) : null,
    navItem('#/speakers', 'users', t('speakers'), c.speakers),
    navItem('#/topics', 'tag', t('topics')));
  fill($('#nav-library'),
    navItem('#/saved', 'bookmark', t('saved'), c.saved),
    navItem('#/recent', 'history', t('recent'), c.recent));
  const sources = app.state?.sources || [];
  const sourceItem = (s) => {
    const a = navItem('#/source/' + encodeURIComponent(s.name), ORIGIN_ICON[s.origin] || 'folder', s.name, s.count,
      s.syncing ? h('span', { class: 'nav-dot' }) : null);
    a.classList.add('small');
    a.title = s.name;
    return a;
  };
  let list;
  if (!sources.length) {
    list = h('a', { class: 'nav-item small', href: '#/settings/sources' }, icon('plus'), h('span', { class: 'label', text: t('addSource') }));
  } else if (sources.length <= 8) {
    list = sources.map(sourceItem);
  } else {
    // Many sources: one collapsible group per type, so the list stays short.
    const groupOf = (s) => s.kind === 'quran' ? 'quran' : s.kind === 'films' ? 'films' : s.origin;
    let open = new Set();
    try { open = new Set(JSON.parse(localStorage.getItem('noor.navGroups') || '[]')); } catch { /* ignore */ }
    const current = decodeURIComponent((location.hash.match(/^#\/source\/(.+)$/) || [])[1] || '');
    list = ['youtube', 'quran', 'web', 'feed', 'films', 'folder'].map((g) => {
      const members = sources.filter((s) => groupOf(s) === g);
      if (!members.length) return null;
      const isOpen = open.has(g) || members.some((s) => s.name === current);
      const head = h('button', {
        class: 'nav-item small nav-group' + (isOpen ? ' open' : ''),
        onclick() {
          open.has(g) ? open.delete(g) : open.add(g);
          try { localStorage.setItem('noor.navGroups', JSON.stringify([...open])); } catch { /* ignore */ }
          renderSidebar();
        },
      }, icon(g === 'quran' ? 'book' : g === 'films' ? 'film' : ORIGIN_ICON[g] || 'folder'), h('span', { class: 'label', text: t('navGroups')[g] }),
        h('span', { class: 'count', text: num(members.length) }), icon('chevronRight'));
      return [head, isOpen ? h('div', { class: 'nav-group-items' }, members.map(sourceItem)) : null];
    });
  }
  fill($('#nav-sources'), list);
  fill($('#nav-bottom'), navItem('#/settings/sources', 'sliders', t('settings')));
  markActiveNav();
}
function markActiveNav() {
  const hash = location.hash || '#/';
  for (const a of document.querySelectorAll('#sidebar .nav-item')) {
    const href = a.getAttribute('data-href') || '';
    const on = href === '#/' ? (hash === '#/' || hash === '#' || hash === '')
      : href.startsWith('#/settings') ? hash.startsWith('#/settings')
        : href === '#/speakers' ? (hash === href || hash.startsWith('#/speaker/'))
          : href === '#/topics' ? (hash === href || hash.startsWith('#/topic/'))
            : hash === href;
    a.classList.toggle('on', on);
  }
}

// ------------------------------------------------- state & job pill
let pollTimer = 0;
async function refreshState() {
  clearTimeout(pollTimer);
  try {
    app.state = await GET('/api/state');
  } catch {
    pollTimer = setTimeout(refreshState, 3000);
    return;
  }
  renderSidebar();
  renderJob();
  const job = app.state.job;
  const busy = job.running || job.queued_roots.length > 0;
  if (app.wasBusy && !busy) {
    if (job.message) toast(job.message);
    if (app.pageCtl?.refresh && page.scrollTop < 300) app.pageCtl.refresh();
  }
  app.wasBusy = busy;
  app.pageCtl?.onState?.(app.state);
  pollTimer = setTimeout(refreshState, busy ? 1200 : 8000);
}
function renderJob() {
  const job = app.state.job;
  const busy = job.running || job.queued_roots.length > 0;
  $('#job').hidden = !busy;
  const progress = $('#progress');
  if (!busy) { progress.classList.remove('indeterminate'); $('#progress-bar').style.width = '0'; return; }
  let text = job.label || '…';
  if (job.detail) text += ' · ' + job.detail;
  if (job.current_root && job.phase) {
    const bits = [job.current_root, t('jobPhase')[job.phase] || job.phase];
    if (job.files) bits.push(t('nFiles', job.files));
    if (job.dirs > 1) bits.push(t('nFolders', job.dirs));
    text = bits.join(' · ');
  }
  if (job.queued_roots.length) text += ` (+${num(job.queued_roots.length)})`;
  $('#job-text').textContent = text;
  if (job.total) { progress.classList.remove('indeterminate'); $('#progress-bar').style.width = `${Math.round((100 * (job.done || 0)) / job.total)}%`; }
  else { progress.classList.add('indeterminate'); $('#progress-bar').style.width = ''; }
}

// ------------------------------------------------------------ router
const routes = [
  [/^#?\/?$/, () => homePage()],
  [/^#\/lectures$/, () => gridPage({ kind: 'lecture', title: t('lectures'), noun: 'nLectures', sorts: 'lecture' })],
  [/^#\/series$/, () => gridPage({ kind: 'series', title: t('series'), noun: 'nSeries', sorts: 'series' })],
  [/^#\/quran$/, () => gridPage({ kind: 'quran', title: t('quran'), noun: 'nQuran', sorts: 'quran' })],
  [/^#\/films$/, () => gridPage({ kind: 'film', title: t('films'), noun: 'nFilms', sorts: 'lecture' })],
  [/^#\/watch\/(\d+)(?:\/(\d+))?$/, (m) => watchPage(Number(m[1]), m[2] ? Number(m[2]) : null)],
  [/^#\/speakers$/, () => speakersPage()],
  [/^#\/speaker\/(.+)$/, (m) => speakerPage(decodeURIComponent(m[1]))],
  [/^#\/topics$/, () => topicsPage()],
  [/^#\/topic\/(.+)$/, (m) => { const k = decodeURIComponent(m[1]); return gridPage({ topic: k, title: topicName(k), noun: 'nItems', sorts: 'mixed', mixed: true }); }],
  [/^#\/saved$/, () => gridPage({ watchlist: true, title: t('saved'), noun: 'nItems', mixed: true, sorts: 'saved', empty: ['bookmark', t('nothingSaved'), t('nothingSavedText')] })],
  [/^#\/recent$/, () => recentPage()],
  [/^#\/search\/(.*)$/, (m) => searchPage(decodeURIComponent(m[1]))],
  [/^#\/source\/(.+)$/, (m) => sourcePage(decodeURIComponent(m[1]))],
  [/^#\/title\/(\d+)$/, (m) => detailPage(Number(m[1]))],
  [/^#\/settings(?:\/(\w+))?$/, (m) => settingsPage(m[1] || 'sources')],
];
async function route() {
  app.pageCtl?.cleanup?.();
  app.pageCtl = null;
  app.navCount++;
  markActiveNav();
  const hash = location.hash || '#/';
  if (!hash.startsWith('#/search/') && document.activeElement !== $('#search')) $('#search').value = '';
  page.replaceChildren();
  page.scrollTop = 0;
  for (const [re, fn] of routes) {
    const m = hash.match(re);
    if (m) {
      try { app.pageCtl = (await fn(m)) || null; }
      catch (e) { page.replaceChildren(emptyState('info', t('somethingWrong'), e.message)); }
      return;
    }
  }
  go('#/');
}
function emptyState(iconName, title, text, ...actions) {
  return h('div', { class: 'empty' }, h('div', { class: 'big-icon' }, icon(iconName)), h('h2', { text: title }), h('div', { text }),
    actions.filter(Boolean).length ? h('div', { class: 'actions' }, actions) : null);
}

// ------------------------------------------------------------- cards
/** Artwork for a card: the image, or a generated tile. */
function art(c, { big = false } = {}) {
  const tile = () => {
    const glyph = c.kind === 'quran' ? 'القرآن' : TOPICS[c.topics?.[0]]?.g || 'ن';
    return h('div', { class: 'tile', style: `background:${c.kind === 'quran' ? 'linear-gradient(135deg,#0f5a44,#0a2119)' : hashColor(c.speaker || c.name)}` },
      h('span', { class: 'glyph', text: glyph }),
      h('div', { class: 't', text: c.name }),
      c.speaker && c.speaker !== c.name ? h('div', { class: 's', text: c.speaker }) : null);
  };
  if (!c.poster) return tile();
  return h('img', { src: c.poster, loading: big ? null : 'lazy', alt: '', onerror() { this.replaceWith(tile()); } });
}
function kindBadge(c) {
  if (c.media === 'youtube') return h('span', { class: 'badge yt' }, icon('youtube'), 'YouTube');
  if (c.kind === 'series') return h('span', { class: 'badge series', text: `${t('seriesOne')} · ${num(c.file_count)}` });
  if (c.kind === 'quran') return h('span', { class: 'badge quran', text: t('quranOne') });
  if (c.kind === 'film') return h('span', { class: 'badge series', text: c.file_count > 1 ? `${t('filmOne')} · ${num(c.file_count)}` : t('filmOne') });
  return null;
}
function cardMeta(c) {
  const bits = [];
  if (c.speaker && c.kind !== 'quran') bits.push(c.speaker);
  if (c.kind === 'series') bits.push(t('nEpisodes', c.file_count));
  else if (c.kind === 'quran') bits.push(t('nSurahs', c.file_count));
  else if (c.file_count > 1) bits.push(t('nParts', c.file_count));
  if (c.media === 'youtube' && c.views != null) bits.push(t('nViews', compact(c.views)));
  if (c.media === 'youtube' && c.file_count === 1) bits.push(since(c.added_at));
  else if (c.language && c.language !== lang) bits.push(langName(c.language));
  return bits.join(' · ');
}
function card(c) {
  const mark = h('button', {
    class: 'mark' + (c.saved ? ' on' : ''), title: c.saved ? t('saveOn') : t('save'),
    async onclick(e) {
      e.stopPropagation();
      c.saved = !c.saved;
      mark.classList.toggle('on', c.saved);
      try { await POST('/api/watchlist', { title_id: c.id, on: c.saved }); refreshState(); } catch (err) { toast(err.message, true); }
    },
  }, icon('bookmark'));
  const playBtn = h('button', { class: 'round-play', title: t('play'), onclick(e) { e.stopPropagation(); quickPlay(c.id); } }, icon('play'));
  const dur = c.total_duration && c.kind === 'lecture' ? h('span', { class: 'duration', text: clock(c.total_duration) }) : null;
  return h('div', { class: 'card' + (player.title()?.id === c.id ? ' now' : ''), 'data-title': c.id, onclick: () => go(isYouTubeTitle(c) ? '#/watch/' + c.id : '#/title/' + c.id) },
    h('div', { class: 'thumb' }, art(c), h('div', { class: 'badges' }, kindBadge(c)), dur, nowBadge(),
      c.progress ? h('div', { class: 'bar' }, h('i', { style: `width:${Math.min(100, Math.round(c.progress * 100))}%` })) : null,
      h('div', { class: 'hover-actions' }, playBtn, mark)),
    h('div', { class: 'card-title', text: c.name, title: c.name }),
    h('div', { class: 'card-meta', text: cardMeta(c) }));
}
/** The moving bars that mark what is playing. */
const eq = () => h('span', { class: 'eq' }, h('i'), h('i'), h('i'));
const nowBadge = () => h('span', { class: 'now-badge' }, eq(), t('nowPlaying'));
/** Marks cards and rows of whatever is playing, wherever they are. */
function markPlaying() {
  const title = player.title()?.id, file = player.current()?.file_id;
  for (const el of document.querySelectorAll('[data-title]')) el.classList.toggle('now', Number(el.dataset.title) === title && file != null);
  for (const el of document.querySelectorAll('[data-file]')) el.classList.toggle('now', Number(el.dataset.file) === file);
}
function avatar(name, image) {
  return h('div', { class: 'avatar', style: image ? '' : `background:${hashColor(name)}` },
    image ? h('img', { src: image, alt: '', loading: 'lazy', onerror() { this.parentElement.style.background = hashColor(name); this.replaceWith(initials(name)); } }) : initials(name));
}
function personCard(s) {
  return h('div', { class: 'person', onclick: () => go('#/speaker/' + encodeURIComponent(s.name)) },
    avatar(s.name, s.image), h('b', { text: s.name }), h('span', { text: t('speakerTitles', s.count) }));
}
function topicTile(k, count) {
  const m = TOPICS[k] || { g: '•', c: ['#2a3532', '#121816'] };
  return h('div', { class: 'topic', style: `background:linear-gradient(135deg,${m.c[0]},${m.c[1]})`, onclick: () => go('#/topic/' + encodeURIComponent(k)) },
    h('span', { class: 'glyph', text: m.g }), h('b', { text: topicName(k) }), count != null ? h('span', { text: t('nItems', count) }) : null);
}

async function quickPlay(titleId) {
  try {
    const d = await GET('/api/titles/' + titleId);
    const list = d.items;
    const idx = Math.max(0, list.findIndex((i) => i.file_id === d.next_up));
    player.start(d, list, idx);
  } catch (e) { toast(e.message, true); }
}

// ------------------------------------------------------------- grids
const SORTS = {
  lecture: [['added', 'sortNewest'], ['popular', 'sortPopular'], ['oldest', 'sortOldest'], ['title', 'sortTitle'], ['longest', 'sortLongest'], ['shortest', 'sortShortest']],
  series: [['updated', 'sortUpdated'], ['added', 'sortNewest'], ['title', 'sortTitle']],
  quran: [['title', 'sortTitle'], ['added', 'sortNewest']],
  mixed: [['added', 'sortNewest'], ['popular', 'sortPopular'], ['title', 'sortTitle'], ['longest', 'sortLongest'], ['shortest', 'sortShortest']],
  saved: [['watchlist', 'sortSaved'], ['added', 'sortNewest'], ['title', 'sortTitle']],
};
function selectBox(options, value, onchange, placeholderLabel) {
  const sel = h('select', { onchange() { onchange(sel.value); sel.classList.toggle('active', !!sel.value); } },
    placeholderLabel != null ? h('option', { value: '', text: placeholderLabel }) : null,
    options.map(([v, label]) => h('option', { value: v, text: label })));
  sel.value = value ?? '';
  if (placeholderLabel != null && sel.value) sel.classList.add('active');
  return h('div', { class: 'select' }, sel);
}

/** A grid of cards with filters, sort and infinite scroll. */
async function gridPage(o) {
  const key = location.hash;
  const sortSet = SORTS[o.sorts] || SORTS.mixed;
  const memo = app.gridMemo[key] || (app.gridMemo[key] = { f: { sort: sortSet[0][0] }, loaded: 0, top: 0 });
  const defs = [];
  if (o.mixed) defs.push(['kind', t('allTypes')]);
  if (!o.speaker) defs.push(['speaker', t('anySpeaker')]);
  if (!o.topic && o.kind !== 'quran') defs.push(['topic', t('anyTopic')]);
  defs.push(['language', t('anyLanguage')]);
  if (o.kind !== 'quran') defs.push(['length', t('anyLength')]);
  defs.push(['media', t('anyMedia')]);
  defs.push(['folder', t('allFolders')]);

  const sub = h('div', { class: 'subtitle', text: ' ' });
  const filters = h('div', { class: 'filters' });
  const grid = h('div', { class: 'grid' });
  const more = h('div', { class: 'loading-more' });
  const sentinel = h('div', { class: 'sentinel' });
  const head = h('div', { class: 'actions', style: 'justify-content:space-between;align-items:flex-end' },
    h('div', { style: 'display:flex;gap:18px;align-items:center' }, o.headArt || null, h('div', {}, h('h1', { class: 'page-title', text: o.title }), sub)), o.headActions || null);
  page.append(h('div', { class: 'wrap' }, head, filters, grid, more, sentinel));

  const base = () => ({ kind: o.kind, root: o.root, speaker: o.speaker, topic: o.topic, watchlist: o.watchlist, q: o.q, ...memo.f });
  let offset = 0, done = false, loading = false, gen = 0;

  async function loadFacets() {
    let f;
    try { f = await GET('/api/facets?' + qs(base())); } catch { return; }
    const lists = {
      kind: [['lecture', t('lectures')], ['series', t('series')], ['quran', t('quran')], ['film', t('films')]],
      speaker: f.speakers.map((x) => [x.value, `${x.value} (${num(x.count)})`]),
      topic: f.topics.map((x) => [x.value, `${topicName(x.value)} (${num(x.count)})`]),
      language: f.languages.map((x) => [x.value, `${langName(x.value)} (${num(x.count)})`]),
      length: [['short', t('short')], ['medium', t('medium')], ['long', t('long')]],
      media: f.media.map((x) => [x.value, t(x.value)]),
      folder: f.folders.map((x) => [x.value, x.value]),
    };
    const boxes = defs
      .filter(([name]) => !(name === 'folder' && !lists.folder.length && !memo.f.folder))
      .filter(([name]) => !(name === 'media' && lists.media.length < 2 && !memo.f.media))
      .map(([name, label]) => selectBox(lists[name], memo.f[name], (v) => { memo.f[name] = v || undefined; reset(); }, label));
    const active = defs.some(([n]) => memo.f[n]);
    fill(filters, ...boxes,
      active ? h('button', { class: 'clear-filters', text: t('clear'), onclick() { for (const [n] of defs) delete memo.f[n]; reset(); } }) : null,
      h('div', { class: 'spacer' }),
      h('span', { class: 'sort-label', text: t('sort') }),
      selectBox(sortSet.map(([v, k]) => [v, t(k)]), memo.f.sort, (v) => { memo.f.sort = v; reset(); }));
  }

  async function loadMore(n = 48) {
    if (loading || done) return;
    loading = true;
    const my = gen;
    let p;
    try { p = await GET('/api/titles?' + qs({ ...base(), offset, limit: n })); }
    catch (e) { loading = false; more.textContent = e.message; return; }
    if (my !== gen) return;
    sub.textContent = t(o.noun, p.total);
    if (!p.total) {
      const filtered = defs.some(([n2]) => memo.f[n2]);
      fill(grid, filtered
        ? emptyState('search', t('noMatches'), t('noMatchesText'), h('button', { class: 'btn', text: t('clearFilters'), onclick() { for (const [n2] of defs) delete memo.f[n2]; reset(); } }))
        : o.empty ? emptyState(...o.empty) : libraryEmpty());
      grid.style.display = 'block';
    } else {
      grid.style.display = '';
      grid.append(...p.items.map(card));
    }
    offset += p.items.length;
    memo.loaded = offset;
    done = offset >= p.total;
    more.textContent = '';
    loading = false;
  }
  function reset() { gen++; offset = 0; done = false; loading = false; grid.replaceChildren(); loadFacets(); loadMore(); }

  const obs = new IntersectionObserver((entries) => { if (entries.some((e) => e.isIntersecting)) loadMore(); }, { root: page, rootMargin: '900px' });
  obs.observe(sentinel);
  loadFacets();
  await loadMore(Math.min(400, Math.max(48, memo.loaded)));
  page.scrollTop = memo.top;
  return { cleanup() { obs.disconnect(); memo.top = page.scrollTop; }, refresh() { memo.loaded = 0; reset(); } };
}

function libraryEmpty() {
  if (!app.state?.sources?.length) {
    return emptyState('folder', t('emptyTitle'), t('emptyText'), h('a', { class: 'btn primary', href: '#/settings/sources' }, icon('plus'), t('addSource')));
  }
  const busy = app.state.job.running;
  return emptyState(busy ? 'sync' : 'mic', busy ? t('syncingTitle') : t('nothingYet'), busy ? t('syncingText') : t('nothingYetText'),
    busy ? null : h('button', { class: 'btn primary', async onclick() { await POST('/api/sync').catch((e) => toast(e.message, true)); refreshState(); } }, icon('sync'), t('syncNow')));
}

function sourcePage(name) {
  const src = app.state?.sources?.find((s) => s.name === name);
  const kind = src ? { lectures: 'lecture', series: 'series', quran: 'quran', films: 'film' }[src.kind] : undefined;
  const head = h('div', { class: 'actions' },
    h('button', { class: 'btn', async onclick() { await POST(`/api/sources/${encodeURIComponent(name)}/sync`).catch((e) => toast(e.message, true)); refreshState(); } }, icon('sync'), t('sync')),
    h('a', { class: 'btn ghost', href: '#/settings/sources' }, icon('sliders'), t('settings')));
  return gridPage({ root: name, kind, title: name, noun: 'nItems', sorts: kind || 'mixed', mixed: !kind, headActions: head });
}
function searchPage(q) {
  const input = $('#search');
  if (document.activeElement !== input) input.value = q;
  return gridPage({ q, title: t('resultsFor', q), noun: 'nResults', mixed: true, sorts: 'mixed', empty: ['search', t('noMatches'), ''] });
}
async function speakerPage(name) {
  const list = await GET('/api/speakers?' + qs({ q: name })).catch(() => ({ speakers: [] }));
  const s = list.speakers.find((x) => x.name === name);
  const headArt = h('div', { style: 'width:84px' }, avatar(name, s?.image));
  return gridPage({ speaker: name, title: name, noun: 'nItems', mixed: true, sorts: 'mixed', headArt });
}

async function speakersPage() {
  const wrap = h('div', { class: 'wrap' });
  page.append(wrap);
  const search = h('input', { class: 'input', placeholder: t('searchSpeakers'), style: 'max-width:340px' });
  const box = h('div', { class: 'people' });
  const sub = h('div', { class: 'subtitle' });
  wrap.append(h('div', { class: 'actions', style: 'justify-content:space-between;align-items:flex-end' }, h('div', {}, h('h1', { class: 'page-title', text: t('speakers') }), sub), search), box);
  const { speakers } = await GET('/api/speakers');
  const render = () => {
    const q = search.value.trim().toLowerCase();
    const list = q ? speakers.filter((s) => s.name.toLowerCase().includes(q)) : speakers;
    sub.textContent = t('nSpeakers', list.length);
    fill(box, list.length ? list.map(personCard) : emptyState('users', t('noMatches'), ''));
  };
  search.addEventListener('input', render);
  render();
}
async function topicsPage() {
  const wrap = h('div', { class: 'wrap' });
  page.append(wrap);
  const f = await GET('/api/facets');
  wrap.append(h('h1', { class: 'page-title', text: t('topics') }), h('div', { class: 'subtitle', text: t('nItems', f.topics.reduce((a, x) => a + x.count, 0)) }));
  wrap.append(f.topics.length ? h('div', { class: 'topics' }, f.topics.map((x) => topicTile(x.value, x.count))) : libraryEmpty());
}
async function recentPage() {
  const wrap = h('div', { class: 'wrap' });
  page.append(wrap);
  const d = await GET('/api/recent');
  wrap.append(h('h1', { class: 'page-title', text: t('recent') }), h('div', { class: 'subtitle', text: t('nItems', d.total) }));
  wrap.append(d.items.length ? h('div', { class: 'grid' }, d.items.map(card)) : emptyState('history', t('nothingPlayed'), t('nothingPlayedText')));
}

// -------------------------------------------------------------- home
async function homePage() {
  const wrap = h('div', { class: 'wrap' });
  page.append(wrap);
  if (!app.state) await refreshState();
  if (!app.state?.sources?.length) { wrap.append(welcome()); return; }
  const [{ rows }, sp, facets] = await Promise.all([GET('/api/home'), GET('/api/speakers').catch(() => ({ speakers: [] })), GET('/api/facets').catch(() => null)]);
  if (!rows.length) { wrap.append(libraryEmpty()); return { refresh: () => route() }; }
  const rowTitle = (k) => k.startsWith('topic:') ? topicName(k.slice(6)) : t({ continue: 'rowContinue', lectures: 'rowLectures', series: 'rowSeries', quran: 'rowQuran', films: 'rowFilms', saved: 'rowSaved' }[k] || k);
  const head = (title, link) => h('div', { class: 'row-head' }, h('h2', { text: title }), link && h('a', { href: link, text: t('seeAll') }));
  const ordered = [...rows];
  for (const r of ordered.splice(0, Math.min(2, ordered.length))) wrap.append(head(rowTitle(r.key), r.link), h('div', { class: 'strip' }, r.items.map(card)));
  if (sp.speakers.length) wrap.append(head(t('rowSpeakers'), '#/speakers'), h('div', { class: 'strip people-strip' }, sp.speakers.slice(0, 24).map(personCard)));
  if (facets?.topics?.length) wrap.append(head(t('rowTopics'), '#/topics'), h('div', { class: 'chips' }, facets.topics.slice(0, 16).map((x) => h('a', { class: 'chip-link', href: '#/topic/' + encodeURIComponent(x.value), text: `${topicName(x.value)} · ${num(x.count)}` }))));
  for (const r of ordered) wrap.append(head(rowTitle(r.key), r.link), h('div', { class: 'strip' }, r.items.map(card)));
  return { refresh: () => route() };
}
function welcome() {
  return h('div', { class: 'welcome' },
    h('div', { class: 'ayah', text: 'اقْرَأْ بِاسْمِ رَبِّكَ الَّذِي خَلَقَ' }),
    h('div', { class: 'ayah-tr', text: t('ayahTr') }),
    h('h2', { text: t('welcomeTitle') }),
    h('p', { text: t('welcomeText') }),
    h('a', { class: 'btn primary big', href: '#/settings/sources' }, icon('plus'), t('startAdd')));
}

// ------------------------------------------------------------ detail
async function detailPage(id) {
  const d = await GET('/api/titles/' + id);
  const kindName = { lecture: t('lectures'), series: t('series'), quran: t('quran'), film: t('films') }[d.kind];
  const listHash = { lecture: '#/lectures', series: '#/series', quran: '#/quran', film: '#/films' }[d.kind];
  const next = d.items.find((i) => i.file_id === d.next_up);
  const started = next && next.position > 30;

  const mark = h('button', {
    class: 'btn big square' + (d.saved ? ' on' : ''), title: d.saved ? t('saveOn') : t('save'),
    async onclick() {
      d.saved = !d.saved;
      mark.classList.toggle('on', d.saved);
      try { await POST('/api/watchlist', { title_id: d.id, on: d.saved }); refreshState(); toast(d.saved ? t('saveOn') : t('save')); } catch (e) { toast(e.message, true); }
    },
  }, icon('bookmark'));
  const playLabel = (d.kind === 'lecture' || d.kind === 'film') && d.items.length <= 1
    ? (started ? t('resume') : t('play'))
    : started ? t('resume') : t('playFrom', next ? itemCode(d, next) : '');
  const isYt = d.media === 'youtube';
  const actions = h('div', { class: 'actions detail-actions' },
    d.items.length && h('button', { class: 'btn primary big', onclick: () => player.start(d, d.items, Math.max(0, d.items.indexOf(next))) }, icon('play'), playLabel),
    d.items.length > 1 && h('button', { class: 'btn big', onclick: () => player.start(d, d.items, 0) }, icon('layers'), t('playAll')),
    mark,
    isYt ? h('button', { class: 'btn big square', title: t('openYouTube'), onclick: () => POST('/api/open-url', { url: d.dir_url }).catch(() => POST('/api/play', { file_id: d.next_up })) }, icon('youtube'))
      : h('button', { class: 'btn big square', title: t('openFolder'), async onclick() { try { await POST('/api/open-folder', { title_id: d.id }); } catch (e) { toast(e.message, true); } } }, icon('folder')));

  const meta = h('div', { class: 'meta-row' },
    d.kind !== 'lecture' && h('span', { class: 'chip ' + (d.kind === 'quran' ? 'green' : 'gold'), text: { quran: t('quranOne'), film: t('filmOne') }[d.kind] || t('seriesOne') }),
    d.kind === 'series' && h('span', { text: t('nEpisodes', d.items.length) }),
    d.kind === 'quran' && h('span', { text: t('nSurahs', d.items.length) }),
    (d.kind === 'lecture' || d.kind === 'film') && d.items.length > 1 && h('span', { text: t('nParts', d.items.length) }),
    d.total_duration && h('span', { text: human(d.total_duration) }),
    d.language && h('span', { class: 'chip outline', text: langName(d.language) }),
    d.media && h('span', { class: 'chip outline', text: t(d.media) }),
    d.year && h('span', { style: 'color:var(--muted)', text: digits(d.year) }));
  const topics = d.topics.length ? h('div', { class: 'genres' }, d.topics.map((k) => h('a', { class: 'chip-link', href: '#/topic/' + encodeURIComponent(k), text: topicName(k) }))) : null;
  let desc = null;
  if (d.description) {
    desc = h('p', { class: 'description', text: d.description });
    if (d.description.length > 300) {
      const toggle = h('button', { class: 'link', text: t('more'), onclick() { desc.classList.toggle('open'); toggle.textContent = desc.classList.contains('open') ? t('less') : t('more'); } });
      desc = h('div', {}, desc, toggle);
    }
  }
  const speakerLine = d.speaker ? h('a', { class: 'speaker-line', href: '#/speaker/' + encodeURIComponent(d.speaker) },
    h('span', { class: 'mini', style: d.speaker_image ? '' : `background:${hashColor(d.speaker)}` }, d.speaker_image ? h('img', { src: d.speaker_image, alt: '' }) : initials(d.speaker)), d.speaker) : null;

  const hero = h('div', { class: 'detail-hero' },
    d.poster ? h('div', { class: 'backdrop', style: `background-image:url("${encodeURI(d.poster)}")` }) : null,
    h('div', { class: 'detail-inner' },
      h('a', { class: 'back', onclick() { app.navCount > 1 ? history.back() : go(listHash); } }, icon('chevronLeft'), kindName),
      h('div', { class: 'detail-top' },
        h('div', { class: 'detail-art' }, art(d, { big: true })),
        h('div', { class: 'detail-info' }, h('h1', { text: d.name }), speakerLine, meta, topics, desc, actions))));

  const container = h('div', {}, hero);
  if (d.kind === 'quran') container.append(surahList(d));
  else if (d.items.length > 1 || d.kind === 'series') container.append(...itemsView(d));
  page.append(container);
  return { cleanup() {}, detailId: d.id };
}
function itemCode(d, it) {
  if (d.kind === 'quran' && it.surah) return `${t('surah')} ${digits(it.surah.number)}`;
  if (it.number == null) return '';
  return `${d.kind === 'lecture' ? t('part') : t('episode')} ${digits(it.number)}`;
}
async function refreshDetail(id) {
  if (location.hash !== '#/title/' + id) return;
  const top = page.scrollTop;
  page.replaceChildren();
  app.pageCtl = (await detailPage(id).catch(() => null)) || null;
  page.scrollTop = top;
}
function checkBtn(it) {
  const b = h('button', {
    class: 'check' + (it.watched ? ' on' : ''), title: it.watched ? t('markUnwatched') : t('markWatched'),
    async onclick(ev) {
      ev.stopPropagation();
      it.watched = !it.watched;
      b.classList.toggle('on', it.watched);
      b.replaceChildren(icon(it.watched ? 'checkCircle' : 'check'));
      await POST('/api/watched', { file_id: it.file_id, on: it.watched }).catch((e) => toast(e.message, true));
    },
  }, icon(it.watched ? 'checkCircle' : 'check'));
  return b;
}
function itemsView(d) {
  const seasons = d.seasons.length > 1 ? d.seasons : null;
  let current = seasons ? (app.detailSeason[d.id] ?? seasons.find((s) => s.items.some((i) => i.file_id === d.next_up))?.season ?? seasons[0].season) : null;
  const bar = h('div', { class: 'seasons-bar' });
  const list = h('div', { class: 'items' });
  function render() {
    const items = seasons ? seasons.find((s) => s.season === current).items : d.items;
    const total = items.reduce((a, i) => a + (i.duration || 0), 0);
    fill(bar,
      seasons && seasons.map((s) => h('button', { class: 'season-pill' + (s.season === current ? ' on' : ''), onclick() { current = s.season; app.detailSeason[d.id] = current; render(); } },
        `${d.kind === 'film' ? t('season') : t('part')} ${digits(s.season)}`, h('small', { text: num(s.items.length) }))),
      h('span', { class: 'total', text: [d.kind === 'lecture' ? t('nParts', items.length) : t('nEpisodes', items.length), total && human(total)].filter(Boolean).join(' · ') }));
    fill(list, items.map((it) => itemRow(d, it, items)));
  }
  render();
  return [bar, list];
}
function itemRow(d, it, items) {
  const isNext = it.file_id === d.next_up;
  const playing = player.current()?.file_id === it.file_id;
  const pct = it.position && it.duration ? Math.min(100, (100 * it.position) / it.duration) : 0;
  const thumb = it.thumb || d.poster;
  const left = it.duration && it.position ? Math.max(1, Math.round((it.duration - it.position) / 60)) : null;
  return h('div', { class: 'item' + (isNext ? ' next' : '') + (playing ? ' playing now' : ''), 'data-file': it.file_id, title: it.name, onclick: () => player.start(d, items, items.indexOf(it)) },
    h('div', { class: 'still' },
      thumb ? h('img', { src: thumb, loading: 'lazy', alt: '' }) : h('div', { class: 'num', text: it.number != null ? digits(pad(it.number)) : '▶' }),
      it.duration ? h('span', { class: 'duration', text: clock(it.duration) }) : null,
      h('div', { class: 'play-hint' }, h('span', { class: 'round-play' }, icon('play')))),
    h('div', { class: 'item-body' },
      h('div', { class: 'item-code' }, itemCode(d, it) || null, isNext && h('span', { class: 'up-next', text: t('upNext') })),
      h('div', { class: 'item-title', text: it.label }),
      h('div', { class: 'item-meta', text: [fmtDate(it.published_at), left && t('minLeft', left), it.size && size(it.size)].filter(Boolean).join(' · ') }),
      it.description && h('div', { class: 'item-desc', text: it.description }),
      pct ? h('div', { class: 'bar' }, h('i', { style: `width:${pct}%` })) : null),
    checkBtn(it));
}
function surahList(d) {
  return h('div', { class: 'surahs' }, d.items.map((it) => {
    const s = it.surah;
    const isNext = it.file_id === d.next_up;
    const playing = player.current()?.file_id === it.file_id;
    return h('div', { class: 'surah' + (isNext ? ' next' : '') + (playing ? ' playing now' : ''), 'data-file': it.file_id, title: it.name, onclick: () => player.start(d, d.items, d.items.indexOf(it)) },
      h('div', { class: 'no' }, h('span', { text: digits(s ? s.number : '?') })),
      h('div', { class: 'body' },
        h('div', { class: 'name', text: s ? `${s.name}${lang === 'bn' ? ` · ${s.meaning_bn}` : ` · ${s.meaning_en}`}` : it.label }),
        h('div', { class: 'sub', text: s ? [t('verses', s.verses), s.makki ? t('makki') : t('madani'), it.duration && clock(it.duration)].filter(Boolean).join(' · ') : '' })),
      s ? h('div', { class: 'ar', text: s.arabic }) : null,
      checkBtn(it));
  }));
}

// ------------------------------------------------------------- player
/** The bottom player bar: plays a queue of items in an audio/video element
 *  or an embedded YouTube player, saves the position as it goes and moves
 *  on to the next item. Files the browser can't decode go to VLC. */
const player = (() => {
  const st = { d: null, queue: [], i: 0, media: null, yt: null, ytTimer: 0, saveAt: 0, speed: 1, error: false, anchor: null, detachTimer: 0, onChange: null, onQueueEnd: null };
  try { st.speed = Number(localStorage.getItem('noor.speed')) || 1; } catch { /* ignore */ }
  try { st.autoplay = localStorage.getItem('noor.autoplay') !== '0'; } catch { st.autoplay = true; }
  const root = $('#player');
  // Video plays on a stage laid over the watch page's player box, or in the
  // corner as a mini-player. Moving an iframe in the DOM reloads it, so the
  // stage itself never moves; only its box changes.
  const slot = h('div', { class: 'media-slot' });
  const miniBar = h('div', { class: 'mini-bar' });
  const stage = h('div', { class: 'stage', hidden: true }, h('div', { class: 'stage-video' }, slot), miniBar);
  document.body.append(stage);
  const cur = () => st.queue[st.i] || null;
  const SPEEDS = [0.75, 1, 1.25, 1.5, 1.75, 2];

  function srcOf(it) {
    if (it.url.startsWith('file:')) return '/media/' + it.file_id;
    return it.url;
  }
  function modeOf(it) {
    if (ytId(it.url)) return 'youtube';
    const e = ext(it.name || it.url);
    if (AUDIO.has(e)) return 'audio';
    if (VIDEO_OK.has(e)) return 'video';
    return 'external';
  }
  const isVideo = (it) => !!it && (modeOf(it) === 'youtube' || modeOf(it) === 'video');
  const watchHash = (d, it) => `#/watch/${d.id}/${it.file_id}`;

  function save(force = false) {
    const it = cur();
    if (!it) return;
    let pos, dur;
    if (st.media) { pos = st.media.currentTime; dur = st.media.duration; }
    else if (st.yt?.getCurrentTime) { pos = st.yt.getCurrentTime(); dur = st.yt.getDuration(); }
    else return;
    const now = Date.now();
    if (!force && now - st.saveAt < 8000) return;
    st.saveAt = now;
    if (pos > 1) {
      it.position = pos;
      POST('/api/progress', { file_id: it.file_id, position: pos, duration: isFinite(dur) && dur > 0 ? dur : null }).catch(() => {});
    }
  }
  function teardown() {
    save(true);
    if (st.media) { st.media.pause(); st.media.removeAttribute('src'); st.media.load(); st.media = null; }
    if (st.yt) { try { st.yt.destroy(); } catch { /* ignore */ } st.yt = null; }
    clearInterval(st.ytTimer);
    slot.replaceChildren();
  }

  /** Plays `items[index]` of title `d`. Video opens the watch page unless
   *  that page is the caller. */
  async function start(d, items, index = 0, { fromWatch = false } = {}) {
    if (!items.length) return toast(t('noEpisodes'), true);
    const externalOnly = app.state?.flags?.external_only;
    const it = items[index];
    if (modeOf(it) === 'external' || (externalOnly && modeOf(it) !== 'youtube')) {
      try { await POST('/api/play', { file_id: it.file_id }); toast(t('opening', it.label, playerName())); setTimeout(() => refreshDetail(d.id), 800); }
      catch (e) { toast(e.message, true); }
      return;
    }
    teardown();
    st.d = d; st.queue = items; st.i = index;
    load();
    if (isVideo(it) && !fromWatch && location.hash !== watchHash(d, it)) go(watchHash(d, it));
  }

  function load() {
    const it = cur();
    st.error = false;
    teardown();
    const mode = modeOf(it);
    if (mode === 'external') {
      POST('/api/play', { file_id: it.file_id }).then(() => toast(t('opening', it.label, playerName()))).catch((e) => toast(e.message, true));
      return next();
    }
    POST('/api/play', { file_id: it.file_id, record_only: true }).catch(() => {});
    render();
    if (mode === 'youtube') loadYouTube(it);
    else {
      const video = mode === 'video';
      const el = h(video ? 'video' : 'audio', { preload: 'auto', playsinline: true, controls: video });
      st.media = el;
      el.playbackRate = st.speed;
      el.addEventListener('loadedmetadata', () => {
        if (it.position > 5 && (!el.duration || it.position < el.duration - 10)) el.currentTime = it.position;
        el.playbackRate = st.speed;
        updateTime();
      });
      el.addEventListener('timeupdate', () => { updateTime(); save(); });
      el.addEventListener('pause', () => { save(true); updateButtons(); });
      el.addEventListener('play', updateButtons);
      el.addEventListener('ended', () => { markDone(it); next(); });
      el.addEventListener('error', () => showError(it));
      el.src = srcOf(it);
      slot.replaceChildren(el);
      el.play().catch(() => {});
    }
    // Keep the address in step when the queue moves on inside the watch page.
    if (isVideo(it) && location.hash.startsWith('#/watch/') && location.hash !== watchHash(st.d, it)) {
      history.replaceState(null, '', watchHash(st.d, it));
    }
    highlight();
    markPlaying();
    st.onChange?.(it);
  }

  function markDone(it) {
    it.watched = true;
    it.position = null;
    POST('/api/progress', { file_id: it.file_id, position: 999999, duration: 1 }).catch(() => {});
  }

  function ensureYtApi() {
    if (window.YT?.Player) return Promise.resolve();
    return new Promise((resolve, reject) => {
      window.onYouTubeIframeAPIReady = resolve;
      const s = document.createElement('script');
      s.src = 'https://www.youtube.com/iframe_api';
      s.onerror = reject;
      document.head.append(s);
      setTimeout(() => reject(new Error('timeout')), 15000);
    });
  }
  async function loadYouTube(it) {
    const holder = h('div', { class: 'yt' });
    slot.replaceChildren(holder);
    try { await ensureYtApi(); } catch { return showError(it, true); }
    if (cur() !== it) return;
    st.yt = new window.YT.Player(holder, {
      width: '100%', height: '100%',
      videoId: ytId(it.url),
      playerVars: { autoplay: 1, rel: 0, modestbranding: 1, playsinline: 1, iv_load_policy: 3, start: it.position > 5 ? Math.floor(it.position) : 0, origin: location.origin },
      events: {
        onReady: (e) => { try { e.target.setPlaybackRate(st.speed); } catch { /* some videos fix the rate */ } updateButtons(); },
        onStateChange: (e) => {
          if (e.data === 0) { markDone(it); next(); }
          if (e.data === 2) save(true);
          updateButtons();
        },
        onError: () => showError(it, true),
      },
    });
    st.ytTimer = setInterval(() => { updateTime(); save(); }, 1000);
  }

  function showError(it, youtube = false) {
    if (cur() !== it) return;
    st.error = true;
    slot.append(h('div', { class: 'media-error' }, h('div', {},
      h('p', { text: youtube ? t('openYouTube') : t('cantPlayHere') }),
      h('div', { class: 'actions', style: 'justify-content:center' },
        youtube ? h('button', { class: 'btn primary', onclick: () => POST('/api/play', { file_id: it.file_id }).then(() => toast(t('openedYouTube'))) }, icon('youtube'), t('openYouTube'))
          : h('button', { class: 'btn primary', onclick: () => POST('/api/play', { file_id: it.file_id }).then(() => toast(t('opening', it.label, playerName()))).catch((e) => toast(e.message, true)) }, icon('external'), t('openIn', playerName())),
        st.i + 1 < st.queue.length && h('button', { class: 'btn', onclick: next }, icon('next'), t('skipNext'))))));
  }

  function next() {
    if (st.i + 1 < st.queue.length) { st.i++; load(); return; }
    save(true);
    updateButtons();
    // End of the queue: the watch page may offer something more.
    if (st.autoplay && st.onQueueEnd?.()) return;
    refreshDetail(st.d?.id);
  }
  function prev() {
    const pos = st.media ? st.media.currentTime : st.yt?.getCurrentTime?.() || 0;
    if (pos > 5 || st.i === 0) seek(0);
    else { st.i--; load(); }
  }
  function seek(sec) {
    if (st.media) st.media.currentTime = Math.max(0, sec);
    else st.yt?.seekTo?.(Math.max(0, sec), true);
  }
  function position() {
    if (st.media) return [st.media.currentTime || 0, st.media.duration || cur()?.duration || 0];
    if (st.yt?.getCurrentTime) return [st.yt.getCurrentTime() || 0, st.yt.getDuration() || cur()?.duration || 0];
    return [0, cur()?.duration || 0];
  }
  function toggle() {
    if (st.media) st.media.paused ? st.media.play() : st.media.pause();
    else if (st.yt?.getPlayerState) st.yt.getPlayerState() === 1 ? st.yt.pauseVideo() : st.yt.playVideo();
  }
  function paused() {
    if (st.media) return st.media.paused;
    if (st.yt?.getPlayerState) return st.yt.getPlayerState() !== 1;
    return true;
  }
  function setSpeed() {
    st.speed = SPEEDS[(SPEEDS.indexOf(st.speed) + 1) % SPEEDS.length] || 1;
    try { localStorage.setItem('noor.speed', String(st.speed)); } catch { /* ignore */ }
    if (st.media) st.media.playbackRate = st.speed;
    try { st.yt?.setPlaybackRate?.(st.speed); } catch { /* ignore */ }
    const b = root.querySelector('.speed');
    if (b) b.textContent = digits(st.speed) + '×';
  }
  function setAutoplay(on) {
    st.autoplay = on;
    try { localStorage.setItem('noor.autoplay', on ? '1' : '0'); } catch { /* ignore */ }
  }
  function close() {
    teardown();
    st.d = null; st.queue = [];
    root.hidden = true;
    layout();
    highlight();
    markPlaying();
    st.onChange?.(null);
  }

  // -------------------------------------------------- stage placement
  const resize = new ResizeObserver(() => layout());
  function attach(el) {
    clearTimeout(st.detachTimer);
    st.anchor = el;
    resize.disconnect();
    resize.observe(el);
    layout();
  }
  function detach(el) {
    if (st.anchor !== el) return;
    st.anchor = null;
    resize.disconnect();
    // The next page may be another watch page; give it a moment to claim
    // the stage before shrinking to the corner.
    st.detachTimer = setTimeout(layout, 250);
  }
  function layout() {
    const it = cur();
    const video = isVideo(it);
    stage.hidden = !video;
    if (video) root.hidden = true;
    document.body.classList.toggle('playing', !!it && !video);
    const docked = video && st.anchor?.isConnected;
    document.body.classList.toggle('mini-playing', video && !docked);
    if (!video) return;
    stage.classList.toggle('mini', !docked);
    if (!docked) { stage.style.cssText = ''; renderMini(); return; }
    const r = st.anchor.getBoundingClientRect();
    const view = page.getBoundingClientRect();
    const top = Math.max(0, view.top - r.top), bottom = Math.max(0, r.bottom - view.bottom);
    Object.assign(stage.style, {
      left: `${r.left}px`, top: `${r.top}px`, width: `${r.width}px`, height: `${r.height}px`,
      clipPath: top || bottom ? `inset(${top}px 0 ${bottom}px 0 round 14px)` : '',
      visibility: top + bottom >= r.height ? 'hidden' : '',
    });
  }
  document.addEventListener('scroll', layout, { capture: true, passive: true });
  window.addEventListener('resize', layout);

  // ------------------------------------------------------------ chrome
  let refs = {};
  /** The bottom bar, for audio. */
  function render() {
    const it = cur();
    if (!it) { root.hidden = true; layout(); return; }
    layout();
    if (isVideo(it)) { renderMini(); return; }
    const d = st.d;
    root.hidden = false;
    const slider = h('input', { type: 'range', min: 0, max: 1000, value: 0 });
    slider.addEventListener('input', () => { const [, dur] = position(); seek((slider.value / 1000) * dur); });
    const vol = h('input', { type: 'range', class: 'vol', min: 0, max: 100, value: 100 });
    vol.addEventListener('input', () => { if (st.media) st.media.volume = vol.value / 100; });
    const playBtn = h('button', { class: 'pbtn main', title: t('play'), onclick: toggle }, icon('pause'));
    refs = { slider, playBtn, cur: h('span', { text: '0:00' }), dur: h('span', { text: clock(it.duration) || '0:00' }) };
    const thumb = it.thumb || d.poster;
    fill(root,
      h('div', { class: 'pbar' },
        h('div', { class: 'pnow', onclick: () => go('#/title/' + d.id) },
          h('div', { class: 'art' }, thumb ? h('img', { src: thumb, alt: '' }) : null),
          h('div', { class: 'txt' }, h('b', { text: d.kind === 'quran' && it.surah ? `${it.surah.name} · ${it.surah.arabic}` : it.label }), h('span', { text: [d.speaker || d.name, itemCode(d, it)].filter(Boolean).join(' · ') }))),
        h('div', { class: 'pcenter' },
          h('div', { class: 'pbuttons' },
            h('button', { class: 'pbtn', title: '−15s', onclick: () => seek(position()[0] - 15) }, icon('back15')),
            h('button', { class: 'pbtn', onclick: prev }, icon('prev')),
            playBtn,
            h('button', { class: 'pbtn', onclick: next, disabled: st.i + 1 >= st.queue.length }, icon('next')),
            h('button', { class: 'pbtn', title: '+30s', onclick: () => seek(position()[0] + 30) }, icon('fwd30'))),
          h('div', { class: 'ptime' }, refs.cur, slider, refs.dur)),
        h('div', { class: 'pright' },
          h('button', { class: 'speed', title: t('speed'), onclick: setSpeed, text: digits(st.speed) + '×' }),
          icon('volume'), vol,
          h('button', { class: 'pbtn', title: t('inPlayer'), onclick: () => POST('/api/play', { file_id: it.file_id }).then(() => toast(t('opening', it.label, playerName()))).catch((e) => toast(e.message, true)) }, icon('external')),
          h('button', { class: 'pbtn', title: t('close'), onclick: close }, icon('x')))));
    updateButtons();
  }
  /** The strip under the corner mini-player. */
  function renderMini() {
    const it = cur();
    if (!it || !st.d) return;
    const d = st.d;
    refs.miniPlay = h('button', { class: 'pbtn', title: t('play'), onclick: toggle }, icon(paused() ? 'play' : 'pause'));
    fill(miniBar,
      h('div', { class: 'mini-txt', onclick: () => go(watchHash(d, it)) },
        h('b', { text: it.label || d.name }), h('span', { text: d.speaker || d.source_title || d.name })),
      refs.miniPlay,
      st.queue.length > 1 && h('button', { class: 'pbtn', onclick: next, disabled: st.i + 1 >= st.queue.length }, icon('next')),
      h('button', { class: 'pbtn', title: t('openPlayer'), onclick: () => go(watchHash(d, it)) }, icon('expand')),
      h('button', { class: 'pbtn', title: t('close'), onclick: close }, icon('x')));
  }
  function updateTime() {
    if (!refs.slider || !refs.slider.isConnected) return;
    const [pos, dur] = position();
    refs.cur.textContent = clock(pos);
    if (dur) { refs.dur.textContent = clock(dur); refs.slider.value = Math.round((pos / dur) * 1000); }
  }
  const playButtons = new Set();
  function updateButtons() {
    const ic = paused() ? 'play' : 'pause';
    if (refs.playBtn) refs.playBtn.replaceChildren(icon(ic));
    if (refs.miniPlay) refs.miniPlay.replaceChildren(icon(ic));
    for (const b of playButtons) b.isConnected ? b.replaceChildren(icon(ic)) : playButtons.delete(b);
    document.body.classList.toggle('media-paused', paused());
  }
  /** Next, from the watch page's buttons: the queue first, then whatever
   *  the page offers (the speaker's next talk). */
  function skip() {
    if (st.i + 1 < st.queue.length) { st.i++; load(); }
    else st.onQueueEnd?.();
  }
  function back() {
    if (st.i > 0) { st.i--; load(); }
    else if (app.navCount > 1) history.back();
  }
  function highlight() {
    const id = cur()?.file_id;
    if (app.pageCtl?.detailId === st.d?.id || !id) {
      for (const el of document.querySelectorAll('.item.playing, .surah.playing')) el.classList.remove('playing');
    }
  }

  document.addEventListener('keydown', (e) => {
    if (!cur() || /INPUT|TEXTAREA|SELECT/.test(document.activeElement.tagName)) return;
    if (e.code === 'Space') { e.preventDefault(); toggle(); }
    else if (e.key === 'ArrowRight') seek(position()[0] + 10);
    else if (e.key === 'ArrowLeft') seek(position()[0] - 10);
  });
  window.addEventListener('beforeunload', () => save(true));
  return {
    start, current: cur, title: () => st.d, attach, detach, setAutoplay,
    toggle, skip, back, seek: (d) => seek(position()[0] + d), setSpeed, speed: () => st.speed,
    hasNext: () => st.i + 1 < st.queue.length, index: () => st.i, size: () => st.queue.length,
    playButton(b) { playButtons.add(b); updateButtons(); return b; },
    autoplay: () => st.autoplay,
    set onChange(f) { st.onChange = f; },
    set onQueueEnd(f) { st.onQueueEnd = f; },
    render: () => { if (cur()) render(); },
  };
})();

// ------------------------------------------------------------- watch
/** "1.2M" / "১২ লা". */
const compact = (n) => new Intl.NumberFormat(lang === 'bn' ? 'bn-BD' : 'en-US', { notation: 'compact', maximumFractionDigits: 1 }).format(n);
/** "3 years ago", from a unix time. */
function since(ts) {
  if (!ts) return '';
  const s = Date.now() / 1000 - ts;
  const rtf = new Intl.RelativeTimeFormat(lang === 'bn' ? 'bn' : 'en', { numeric: 'auto' });
  for (const [unit, secs] of [['year', 31536000], ['month', 2592000], ['week', 604800], ['day', 86400], ['hour', 3600], ['minute', 60]]) {
    if (s >= secs) return rtf.format(-Math.floor(s / secs), unit);
  }
  return rtf.format(0, 'minute');
}
const isYouTubeTitle = (c) => c.media === 'youtube' && c.file_count === 1;

async function watchPage(titleId, fileId) {
  const d = await GET('/api/titles/' + titleId);
  const pick = () => d.items.find((i) => i.file_id === fileId) || d.items.find((i) => i.file_id === d.next_up) || d.items[0];
  if (!pick()) { page.append(emptyState('info', t('noEpisodes'), '')); return null; }
  const playing = player.current();
  let it = playing && player.title()?.id === d.id ? playing : pick();
  if (!playing || playing.file_id !== it.file_id) player.start(d, d.items, d.items.indexOf(it), { fromWatch: true });

  // The left column holds the player (pinned) and the details under it; the
  // right column scrolls on its own.
  const box = h('div', { class: 'watch-player' });
  const controls = h('div', { class: 'watch-controls' });
  const info = h('div', { class: 'watch-info' });
  const side = h('aside', { class: 'watch-side' });
  page.append(h('div', { class: 'watch' }, h('div', { class: 'watch-main' }, h('div', { class: 'watch-pin' }, box, controls), info), side));
  player.attach(box);

  // More from the same speaker (or source), for the side list, Next and
  // autoplay.
  const more = [];
  const moreQuery = d.speaker ? { speaker: d.speaker } : { root: d.root };
  GET('/api/titles?' + qs({ ...moreQuery, sort: 'added', limit: 40 })).then((r) => {
    more.push(...r.items.filter((c) => c.id !== d.id));
    renderSide();
    renderControls();
  }).catch(() => {});
  const nextTitle = () => more.find((c) => !(c.progress >= 0.95)) || more[0];
  const openTitle = (c) => go(isYouTubeTitle(c) ? '#/watch/' + c.id : '#/title/' + c.id);

  function renderBox() {
    const poster = it.thumb || d.poster;
    fill(box,
      poster ? h('img', { src: poster, alt: '' }) : null,
      h('button', { class: 'watch-replay', title: t('play'), onclick: () => player.start(d, d.items, d.items.indexOf(it), { fromWatch: true }) }, icon('play')));
  }
  function renderControls() {
    const n = d.items.indexOf(it);
    const upcoming = n + 1 < d.items.length ? d.items[n + 1] : null;
    const after = upcoming ? upcoming.label : nextTitle()?.name;
    const speed = h('button', { class: 'speed', title: t('speed'), text: digits(player.speed()) + '×', onclick() { player.setSpeed(); this.textContent = digits(player.speed()) + '×'; } });
    fill(controls,
      h('div', { class: 'wc-buttons' },
        h('button', { class: 'pbtn', title: t('prevVideo'), disabled: n === 0 && app.navCount <= 1, onclick: () => player.back() }, icon('prev')),
        h('button', { class: 'pbtn', title: '−10s', onclick: () => player.seek(-10) }, icon('back15')),
        player.playButton(h('button', { class: 'pbtn main', title: t('play'), onclick: () => player.toggle() }, icon('pause'))),
        h('button', { class: 'pbtn', title: '+10s', onclick: () => player.seek(10) }, icon('fwd30')),
        h('button', { class: 'pbtn', title: t('nextVideo'), disabled: !after, onclick: () => (upcoming ? player.skip() : nextTitle() && openTitle(nextTitle())) }, icon('next'))),
      h('div', { class: 'wc-now' }, eq(), h('span', { text: d.items.length > 1 ? t('episodeOf', digits(n + 1), digits(d.items.length)) : t('nowPlaying') })),
      after ? h('div', { class: 'wc-next', title: after, onclick: () => (upcoming ? player.skip() : openTitle(nextTitle())) },
        h('span', { text: t('nextVideo') + ':' }), h('b', { text: after })) : h('div', { class: 'wc-next' }),
      speed);
  }
  function renderInfo() {
    const series = d.items.length > 1;
    const channel = d.speaker || d.source_title || d.root;
    const channelImg = d.media === 'youtube' && (!d.speaker || d.speaker === d.source_title) ? d.source_image : d.speaker_image || d.source_image;
    const mark = h('button', {
      class: 'pill' + (d.saved ? ' on' : ''),
      async onclick() {
        d.saved = !d.saved;
        mark.classList.toggle('on', d.saved);
        try { await POST('/api/watchlist', { title_id: d.id, on: d.saved }); refreshState(); toast(d.saved ? t('saveOn') : t('save')); } catch (e) { toast(e.message, true); }
      },
    }, icon('bookmark'), t('save'));
    const yt = ytId(it.url);
    const statLine = [it.views != null && t('nViews', compact(it.views)), since(it.published_at)].filter(Boolean).join('  ·  ');
    const desc = it.description || d.description;
    const descEl = desc ? h('div', { class: 'watch-desc-text', text: desc }) : null;
    const moreBtn = desc && desc.length > 240 ? h('button', { class: 'link', text: t('more'), onclick() { descEl.classList.toggle('open'); this.textContent = descEl.classList.contains('open') ? t('less') : t('more'); } }) : null;
    fill(info,
      h('h1', { class: 'watch-title', text: it.label || d.name }),
      h('div', { class: 'watch-row' },
        h('a', { class: 'watch-channel', href: d.speaker ? '#/speaker/' + encodeURIComponent(d.speaker) : '#/source/' + encodeURIComponent(d.root) },
          avatar(channel, channelImg),
          h('div', {}, h('b', { text: channel }), h('span', { text: d.source_title && d.source_title !== channel ? d.source_title : series ? t('episodeOf', digits(d.items.indexOf(it) + 1), digits(d.items.length)) : t(d.kind === 'film' ? 'filmOne' : 'lecture') }))),
        h('div', { class: 'watch-actions' },
          it.likes != null && h('span', { class: 'pill stat' }, icon('thumbsUp'), compact(it.likes)),
          it.comments != null && h('span', { class: 'pill stat' }, icon('comment'), compact(it.comments)),
          mark,
          yt ? h('button', { class: 'pill', onclick: () => POST('/api/play', { file_id: it.file_id }).then(() => toast(t('openedYouTube'))) }, icon('youtube'), 'YouTube')
            : h('button', { class: 'pill', onclick: () => POST('/api/play', { file_id: it.file_id }).then(() => toast(t('opening', it.label, playerName()))).catch((e) => toast(e.message, true)) }, icon('external'), playerName()),
          yt && h('button', { class: 'pill square', title: t('copyLink'), onclick: () => navigator.clipboard.writeText(it.url).then(() => toast(t('linkCopied'))).catch(() => toast(it.url)) }, icon('link')))),
      h('div', { class: 'watch-desc' },
        statLine && h('div', { class: 'watch-stats', text: statLine }),
        d.topics.length ? h('div', { class: 'genres' }, d.topics.map((k) => h('a', { class: 'chip-link', href: '#/topic/' + encodeURIComponent(k), text: topicName(k) })),
          d.language && h('span', { class: 'chip outline', text: langName(d.language) })) : null,
        descEl, moreBtn));
  }
  function sideRow({ thumb, title, sub, dur, prog, onclick, file, titleId }) {
    return h('div', { class: 'up-row', 'data-file': file, 'data-title': titleId, onclick },
      h('div', { class: 'up-thumb' }, thumb ? h('img', { src: thumb, alt: '', loading: 'lazy' }) : h('div', { class: 'tile', style: `background:${hashColor(title)}` }),
        dur ? h('span', { class: 'duration', text: clock(dur) }) : null,
        prog ? h('div', { class: 'bar' }, h('i', { style: `width:${Math.min(100, Math.round(prog * 100))}%` })) : null,
        nowBadge()),
      h('div', { class: 'up-txt' }, h('b', { text: title, title }), sub.map((s) => s && h('span', { text: s }))));
  }
  function renderSide() {
    const auto = h('label', { class: 'switch' },
      h('input', { type: 'checkbox', checked: player.autoplay(), onchange() { player.setAutoplay(this.checked); } }), h('i'), t('autoplay'));
    const series = d.items.length > 1;
    // A single video shows itself at the top as "Playing"; a series lists
    // all its episodes with the current one marked.
    const queue = h('div', { class: 'up-list' + (series ? ' queue' : '') },
      (series ? d.items : [it]).map((x) => sideRow({
        thumb: x.thumb || d.poster, title: x.label || d.name, file: x.file_id,
        sub: [series ? itemCode(d, x) : d.speaker || d.source_title, x.views != null && t('nViews', compact(x.views))],
        dur: x.duration, prog: x.watched ? 1 : x.position && x.duration ? x.position / x.duration : 0,
        onclick: () => player.start(d, d.items, d.items.indexOf(x), { fromWatch: true }),
      })));
    const others = more.length ? h('div', { class: 'up-list' },
      more.map((c) => sideRow({
        thumb: c.poster, title: c.name, titleId: c.id,
        sub: [c.speaker && c.speaker !== (d.speaker || '') ? c.speaker : '', [c.views != null && t('nViews', compact(c.views)), since(c.added_at)].filter(Boolean).join(' · ')],
        dur: c.file_count === 1 ? c.total_duration : null, prog: c.progress, onclick: () => openTitle(c),
      }))) : null;
    fill(side,
      h('div', { class: 'up-head' }, h('h3', { text: t('upNext') }), auto),
      queue,
      others && h('div', { class: 'up-label', text: t('moreFrom', d.speaker || d.source_title || d.root) }),
      others);
    markPlaying();
    side.querySelector('.up-row.now')?.scrollIntoView({ block: 'nearest' });
  }

  renderBox();
  renderControls();
  renderInfo();
  renderSide();
  player.onChange = (x) => {
    if (x && player.title()?.id === d.id) { it = x; renderBox(); renderControls(); renderInfo(); renderSide(); }
  };
  // Past the last episode, Next (and autoplay) moves on to the speaker's
  // next talk.
  player.onQueueEnd = () => {
    const c = nextTitle();
    if (!c) return false;
    openTitle(c);
    return true;
  };
  return {
    cleanup() { player.detach(box); player.onChange = null; player.onQueueEnd = null; },
    detailId: d.id,
  };
}

// ---------------------------------------------------------- settings
const TABS = [['sources', 'database', 'tabSources'], ['player', 'monitor', 'tabPlayer'], ['youtube', 'youtube', 'tabYouTube'], ['assistant', 'sparkles', 'tabAssistant'], ['language', 'languages', 'tabLanguage'], ['library', 'drive', 'tabLibrary']];
async function settingsPage(tab) {
  if (!TABS.some((x) => x[0] === tab)) tab = 'sources';
  const wrap = h('div', { class: 'wrap' }, h('h1', { class: 'page-title', text: t('settings') }),
    h('div', { class: 'tabs' }, TABS.map(([id, ic, label]) => h('a', { class: 'tab' + (id === tab ? ' on' : ''), href: '#/settings/' + id }, icon(ic), t(label)))));
  page.append(wrap);
  const body = h('div');
  wrap.append(body);
  const s = await GET('/api/settings');
  switch (tab) {
    case 'player': return playerTab(body, s);
    case 'youtube': return youtubeTab(body, s);
    case 'assistant': return assistantTab(body, s);
    case 'language': return languageTab(body);
    case 'library': return libraryTab(body, s);
    default: return sourcesTab(body);
  }
}
function field(label, control, hint, style) {
  return h('div', { class: 'field', style }, h('label', { text: label }), control, hint && h('div', { class: 'hint', text: hint }));
}
function seg(options, value, onchange) {
  const box = h('div', { class: 'seg' });
  const render = () => fill(box, options.map(([v, label, ic]) => h('button', { class: v === value ? 'on' : '', type: 'button', onclick() { value = v; render(); onchange(v); } }, ic && icon(ic), label)));
  render();
  box.set = (v) => { value = v; render(); };
  return box;
}
async function pickPath(kind) {
  try { return (await POST('/api/pick', { kind })).path; } catch (e) { toast(e.message, true); return null; }
}
const KIND_OPTS = () => [['lectures', t('kindLectures'), 'mic'], ['series', t('kindSeries'), 'layers'], ['quran', t('kindQuran'), 'book'], ['films', t('kindFilms'), 'film']];
const msgLine = () => h('div', { class: 'result-line' });
const setMsg = (el, text, ok) => { el.className = 'result-line ' + (ok ? 'ok' : 'err'); el.textContent = text; };

function sourcesTab(body) {
  let kind = 'lectures', origin = null;
  const loc = h('input', { class: 'input mono', placeholder: 'http://server/Waz/   ·   https://www.youtube.com/@channel   ·   https://…/feed.xml', spellcheck: 'false' });
  const name = h('input', { class: 'input', placeholder: 'Tafsir' });
  const speaker = h('input', { class: 'input' });
  const speakerFolders = h('input', { type: 'checkbox' });
  const kindHint = h('div', { class: 'hint', text: t('kindHint').lectures });
  const kindSeg = seg(KIND_OPTS(), kind, (v) => { kind = v; kindHint.textContent = t('kindHint')[v]; });
  loc.addEventListener('input', () => { origin = null; });
  const result = msgLine();
  const browse = h('button', { class: 'btn', title: t('selectFirst'), async onclick() { const p = await pickPath('folder'); if (p) { loc.value = p; origin = 'folder'; } } }, icon('folder'));
  const check = h('button', {
    class: 'btn', text: t('checkLink'), async onclick() {
      if (!loc.value.trim()) return loc.focus();
      check.disabled = true;
      result.className = 'result-line'; result.textContent = t('checking');
      try {
        const r = await POST('/api/sources/check', { location: loc.value });
        origin = r.origin;
        const kindLabel = { lectures: t('kindLectures'), series: t('kindSeries'), quran: t('kindQuran'), films: t('kindFilms') }[r.kind];
        fill(result, `${t('checkOk', t('originNames')[r.origin], t('nItems', r.items), r.folders || null)} ${t('looksLike', kindLabel)}`,
          r.sample?.length ? h('span', { class: 'sample', text: r.sample.join(' · ') }) : null);
        result.className = 'result-line ok';
        kind = r.kind; kindSeg.set(r.kind); kindHint.textContent = t('kindHint')[r.kind];
        if (!name.value.trim() && r.name) name.value = r.name;
        if (!speaker.value.trim() && r.speaker) speaker.value = r.speaker;
      } catch (e) { setMsg(result, e.message, false); }
      check.disabled = false;
    },
  });
  const add = h('button', {
    class: 'btn primary', async onclick() {
      if (!loc.value.trim()) return loc.focus();
      add.disabled = true;
      try {
        const r = await POST('/api/sources', { location: loc.value, name: name.value, kind, origin, speaker: speaker.value.trim() || null, speaker_folders: speakerFolders.checked, sync: true });
        setMsg(result, t('added', r.name), true);
        loc.value = ''; name.value = ''; speaker.value = ''; speakerFolders.checked = false; origin = null;
        await refreshState();
      } catch (e) { setMsg(result, e.message, false); }
      add.disabled = false;
    },
  }, icon('plus'), t('addAndSync'));
  [loc, name].forEach((el) => el.addEventListener('keydown', (e) => { if (e.key === 'Enter') add.click(); }));

  const addPanel = h('div', { class: 'panel' },
    h('h2', { text: t('addLink') }), h('p', { class: 'lead', text: t('addLead') }),
    h('div', { class: 'form-row' },
      field(t('location'), h('div', { class: 'with-btn' }, loc, browse), t('locationHint'), 'flex:1 1 380px'),
      field(t('type'), kindSeg, null),
      field(t('name'), name, t('nameHint'), 'flex:0 1 200px')),
    h('div', { class: 'form-row', style: 'margin-top:14px;align-items:center' },
      field(t('speakerField'), speaker, t('speakerHint'), 'flex:0 1 280px'),
      h('label', { class: 'check-row', style: 'margin-top:8px' }, speakerFolders, t('speakerFolders')),
      h('div', { style: 'flex:1' }),
      h('div', { class: 'actions' }, check, add)),
    kindHint, result);

  const list = h('div', { class: 'sources-list' });
  const countLine = h('div', { class: 'subtitle' });
  const syncAll = h('button', { class: 'btn' });
  const full = h('button', { class: 'btn', text: t('fullRescan'), async onclick() {
    if (!await confirmBox(t('rescanQ'), t('rescanText'), t('rescanOk'))) return;
    try { await POST('/api/sync', { full: true }); refreshState(); } catch (e) { toast(e.message, true); }
  } });
  const logBox = h('pre', { class: 'log' });
  body.append(addPanel,
    h('div', { class: 'panel' }, h('div', { class: 'panel-head' }, h('div', {}, h('h2', { text: t('yourSources') }), countLine), h('div', { class: 'actions' }, full, syncAll)), list),
    h('div', { class: 'panel' }, h('h2', { text: t('activity') }), h('p', { class: 'lead', text: t('activityLead') }), logBox));

  function render(state) {
    const sources = state.sources;
    const busy = state.job.running || state.job.queued_roots.length > 0;
    countLine.textContent = t('nLinks', sources.length);
    syncAll.className = 'btn' + (busy ? ' busy' : ' primary');
    fill(syncAll, busy ? h('span', { class: 'spinner' }) : icon('sync'), busy ? t('syncing') : t('syncAll'));
    syncAll.onclick = async () => {
      if (busy) { await POST('/api/jobs/stop'); toast(t('stopping')); }
      else { try { await POST('/api/sync'); } catch (e) { toast(e.message, true); } }
      refreshState();
    };
    full.disabled = !sources.length;
    logBox.textContent = state.job.log.length ? state.job.log.join('\n') : t('nothing');
    logBox.scrollTop = logBox.scrollHeight;
    if (app.editing) return;
    fill(list, sources.length ? sources.map((s) => sourceRow(s, state)) : h('div', { class: 'subtitle', style: 'padding:14px 0', text: t('noSources') }));
  }
  function sourceRow(s, state) {
    const dotClass = s.syncing ? 'syncing' : s.status === 'ok' ? 'ok' : s.status === 'error' ? 'error' : '';
    let when = h('div', { class: 'when', text: ago(s.last_sync) });
    if (s.syncing) when = h('div', { class: 'when busy', text: `${t('syncing')} ${state.job.detail || ''}` });
    else if (s.queued) when = h('div', { class: 'when busy', text: t('queued') });
    else if (s.status === 'error') when = h('div', { class: 'when err', text: `${ago(s.last_sync)} · ${s.error || 'error'}` });
    else if (s.status === 'stopped') when = h('div', { class: 'when', text: `${ago(s.last_sync)} · ${t('stoppedEarly')}` });
    const enc = encodeURIComponent(s.name);
    const kindLabel = { lectures: t('kindLectures'), series: t('kindSeries'), quran: t('kindQuran'), films: t('kindFilms') }[s.kind];
    const row = h('div', { class: 'source-row' },
      h('div', { class: 'origin ' + s.origin, title: t('originNames')[s.origin] }, icon(ORIGIN_ICON[s.origin] || 'folder')),
      h('div', { class: 'body' },
        h('div', { class: 'name-line' }, h('span', { class: 'dot ' + dotClass }), h('a', { class: 'name', href: '#/source/' + enc, text: s.name }),
          h('span', { class: 'tag', text: kindLabel }), h('span', { class: 'tag', text: t('nItems', s.count) }), s.speaker && h('span', { class: 'tag', text: s.speaker })),
        h('div', { class: 'loc', text: s.location, title: s.location }), when),
      h('div', { class: 'row-actions' },
        h('button', { class: 'btn small', disabled: s.syncing || s.queued, async onclick() { await POST(`/api/sources/${enc}/sync`).catch((e) => toast(e.message, true)); refreshState(); } }, icon('sync'), t('sync')),
        (s.syncing || s.queued) && h('button', { class: 'icon-btn', title: t('stop'), async onclick() { await POST(`/api/sources/${enc}/stop`); refreshState(); } }, icon('stop')),
        h('button', { class: 'icon-btn', title: t('edit'), onclick() { editRow(row, s); } }, icon('pencil')),
        h('button', { class: 'icon-btn danger', title: t('del'), async onclick() {
          if (!await confirmBox(t('deleteQ', s.name), t('deleteText'), t('del'), true)) return;
          try { await DEL(`/api/sources/${enc}`); } catch (e) { toast(e.message, true); }
          refreshState();
        } }, icon('trash'))));
    return row;
  }
  function editRow(row, s) {
    app.editing = s.name;
    let k = s.kind;
    const n = h('input', { class: 'input', value: s.name });
    const l = h('input', { class: 'input mono', value: s.location });
    const sp = h('input', { class: 'input', value: s.speaker || '' });
    const sf = h('input', { type: 'checkbox' });
    sf.checked = s.speaker_folders;
    const m = msgLine();
    row.replaceWith(h('div', { class: 'source-row', style: 'display:block' }, h('div', { class: 'edit-box' },
      h('div', { class: 'form-row' }, field(t('name'), n, null, 'flex:0 1 220px'), field(t('location'), l, null, 'flex:1 1 320px'), field(t('type'), seg(KIND_OPTS(), k, (v) => { k = v; }))),
      h('div', { class: 'form-row', style: 'margin-top:12px;align-items:center' }, field(t('speakerField'), sp, null, 'flex:0 1 260px'), h('label', { class: 'check-row', style: 'margin-top:18px' }, sf, t('speakerFolders'))),
      m,
      h('div', { class: 'actions', style: 'margin-top:12px' },
        h('button', { class: 'btn primary', text: t('saveBtn'), async onclick() {
          try {
            await PUT(`/api/sources/${encodeURIComponent(s.name)}`, { name: n.value, location: l.value, kind: k, origin: s.origin, speaker: sp.value.trim() || null, speaker_folders: sf.checked });
            app.editing = null; toast(t('savedOk')); await refreshState(); render(app.state);
          } catch (e) { setMsg(m, e.message, false); }
        } }),
        h('button', { class: 'btn', text: t('cancel'), onclick() { app.editing = null; render(app.state); } })))));
  }
  render(app.state);
  return { onState: render, cleanup() { app.editing = null; } };
}

function playerTab(body, s) {
  const p = s.player;
  const cmd = h('input', { class: 'input mono', value: p.command });
  const args = h('input', { class: 'input mono', value: p.args.join(' ') });
  const ext_ = h('input', { type: 'checkbox' });
  ext_.checked = p.external_only;
  const m = msgLine();
  body.append(h('div', { class: 'panel' }, h('h2', { text: t('playerTitle') }), h('p', { class: 'lead', text: t('playerLead') }),
    h('div', { class: 'form-row' },
      field(t('program'), h('div', { class: 'with-btn' }, cmd, h('button', { class: 'btn', async onclick() { const x = await pickPath('program'); if (x) cmd.value = x; } }, icon('folder'), t('browse'))), t('programHint'), 'flex:1 1 420px'),
      field(t('args'), args, t('argsHint'), 'flex:1 1 260px')),
    h('label', { class: 'check-row', style: 'margin-top:16px' }, ext_, t('externalOnly')),
    h('div', { class: 'actions', style: 'margin-top:18px' },
      h('button', { class: 'btn primary', text: t('saveBtn'), async onclick() {
        try { await PUT('/api/settings/player', { command: cmd.value, args: args.value.split(/\s+/).filter(Boolean), external_only: ext_.checked }); setMsg(m, t('savedOk'), true); refreshState(); }
        catch (e) { setMsg(m, e.message, false); }
      } }),
      h('button', { class: 'btn ghost', onclick: () => POST('/api/open-url', { url: 'https://www.videolan.org/vlc/' }).catch((e) => toast(e.message, true)) }, icon('external'), t('getVlc'))),
    m));
}
function youtubeTab(body, s) {
  const key = h('input', { class: 'input mono', type: 'password', value: s.youtube.api_key, placeholder: 'AIza…' });
  const show = h('button', { class: 'btn', onclick() { key.type = key.type === 'password' ? 'text' : 'password'; } }, icon('eye'));
  const m = msgLine();
  body.append(h('div', { class: 'panel' }, h('h2', { text: t('ytTitle') }), h('p', { class: 'lead', text: t('ytLead') }),
    h('ol', { class: 'steps' }, t('ytSteps').map((x) => h('li', { text: x }))),
    h('div', { class: 'actions', style: 'margin:10px 0 18px' }, h('button', { class: 'btn ghost', onclick: () => POST('/api/open-url', { url: 'https://console.cloud.google.com/apis/library/youtube.googleapis.com' }).catch((e) => toast(e.message, true)) }, icon('external'), t('openConsole'))),
    field(t('ytKey'), h('div', { class: 'with-btn' }, key, show), null, 'max-width:620px'),
    h('div', { class: 'actions', style: 'margin-top:18px' },
      h('button', { class: 'btn', text: t('testKey'), async onclick() {
        m.className = 'result-line'; m.textContent = t('checking');
        try { await POST('/api/youtube/test', { api_key: key.value }); setMsg(m, t('keyWorks'), true); } catch (e) { setMsg(m, e.message, false); }
      } }),
      h('button', { class: 'btn primary', text: t('saveBtn'), async onclick() {
        try { await PUT('/api/settings/youtube', { api_key: key.value.trim() }); setMsg(m, t('savedOk'), true); refreshState(); } catch (e) { setMsg(m, e.message, false); }
      } })),
    m));
}
function assistantTab(body, s) {
  const a = s.assistant;
  const key = h('input', { class: 'input mono', type: 'password', value: a.api_key, placeholder: 'Gemini API key' });
  const show = h('button', { class: 'btn', onclick() { key.type = key.type === 'password' ? 'text' : 'password'; } }, icon('eye'));
  const model = h('input', { class: 'input mono', value: a.model, list: 'model-list' });
  const datalist = h('datalist', { id: 'model-list' });
  const m = msgLine();
  body.append(h('div', { class: 'panel' }, h('h2', { text: t('asTitle') }), h('p', { class: 'lead', text: t('asLead') }),
    h('div', { class: 'form-row' },
      field(t('geminiKey'), h('div', { class: 'with-btn' }, key, show), null, 'flex:1 1 380px'),
      field(t('model'), h('div', { class: 'with-btn' }, model, h('button', { class: 'btn', text: t('loadList'), async onclick() {
        m.className = 'result-line'; m.textContent = t('checking');
        try { const r = await GET('/api/assistant/models?' + qs({ key: key.value.trim() })); fill(datalist, r.models.map((x) => h('option', { value: x }))); setMsg(m, t('modelsOk', r.models.length), true); }
        catch (e) { setMsg(m, e.message, false); }
      } })), null, 'flex:1 1 260px'), datalist),
    h('div', { class: 'hint', style: 'margin-top:8px' }, t('freeFrom'), ' ', h('button', { class: 'link', text: 'aistudio.google.com/apikey', onclick: () => POST('/api/open-url', { url: 'https://aistudio.google.com/apikey' }).catch((e) => toast(e.message, true)) })),
    h('div', { class: 'actions', style: 'margin-top:18px' },
      h('button', { class: 'btn primary', text: t('saveBtn'), async onclick() {
        try { await PUT('/api/settings/assistant', { api_key: key.value.trim(), model: model.value.trim() }); setMsg(m, t('savedOk'), true); refreshState(); } catch (e) { setMsg(m, e.message, false); }
      } }),
      h('button', { class: 'btn danger', async onclick() {
        if (!await confirmBox(t('clearChatQ'), t('clearChatText'), t('clear'), true)) return;
        await DEL('/api/assistant'); assistant.messages = []; renderAssistant(); toast(t('chatCleared'));
      } }, icon('trash'), t('clearChat'))),
    m));
}
function languageTab(body) {
  body.append(h('div', { class: 'panel' }, h('h2', { text: t('langTitle') }), h('p', { class: 'lead', text: t('langLead') }),
    seg([['bn', 'বাংলা'], ['en', 'English']], lang, (v) => setLang(v))));
}
async function libraryTab(body, s) {
  const lib = s.library;
  const skip = h('input', { class: 'input', value: lib.skip_dirs.join(', ') });
  const exts = h('input', { class: 'input', value: lib.media_extensions.join(', ') });
  const conc = h('input', { class: 'input', type: 'number', min: 1, max: 64, value: lib.concurrency });
  const delay = h('input', { class: 'input', type: 'number', min: 0, value: lib.delay_ms });
  const timeout = h('input', { class: 'input', type: 'number', min: 1, value: lib.timeout_secs });
  const depth = h('input', { class: 'input', type: 'number', min: 1, value: lib.max_depth });
  const m = msgLine();
  const stats = h('div', { class: 'stats' });
  const kv = h('div', { class: 'kv' });
  const list = (v) => v.split(',').map((x) => x.trim()).filter(Boolean);
  const n = (el) => Math.max(0, parseInt(el.value, 10) || 0);
  async function action(name, title, text, ok, danger) {
    if (!await confirmBox(title, text, ok, danger)) return;
    try { await POST('/api/storage/' + name); toast(t('done')); refreshState(); load(); app.gridMemo = {}; } catch (e) { toast(e.message, true); }
  }
  body.append(
    h('div', { class: 'panel' }, h('h2', { text: t('libTitle') }), stats, kv,
      h('div', { class: 'actions', style: 'margin-top:16px' }, h('button', { class: 'btn', onclick: () => POST('/api/storage/open').catch((e) => toast(e.message, true)) }, icon('folder'), t('openData')))),
    h('div', { class: 'panel' }, h('h2', { text: t('scanning') }), h('p', { class: 'lead', text: t('scanningLead') }),
      h('div', { class: 'form-row' }, field(t('skipFolders'), skip, t('commaHint'), 'flex:1 1 300px'), field(t('fileTypes'), exts, null, 'flex:1 1 300px')),
      h('div', { class: 'form-row', style: 'margin-top:14px' }, field(t('parallel'), conc), field(t('pause'), delay), field(t('timeout'), timeout), field(t('depth'), depth)),
      h('div', { class: 'actions', style: 'margin-top:18px' }, h('button', { class: 'btn primary', text: t('saveBtn'), async onclick() {
        try { await PUT('/api/settings/library', { skip_dirs: list(skip.value), media_extensions: list(exts.value), concurrency: n(conc), delay_ms: n(delay), timeout_secs: n(timeout), max_depth: n(depth) }); setMsg(m, t('savedOk'), true); }
        catch (e) { setMsg(m, e.message, false); }
      } })), m),
    h('div', { class: 'panel' }, h('h2', { text: t('maintenance') }), h('p', { class: 'lead', text: t('maintenanceLead') }),
      h('div', { class: 'actions' },
        h('button', { class: 'btn', onclick: () => action('rebuild', t('rebuild'), t('rebuildText'), t('rebuild')) }, icon('sync'), t('rebuild')),
        h('button', { class: 'btn', onclick: () => action('clear-history', t('clearHistory'), t('clearHistoryText'), t('clear'), true) }, icon('history'), t('clearHistory')),
        h('button', { class: 'btn danger-solid', onclick: () => action('reset', t('reset'), t('resetText'), t('resetOk'), true) }, icon('trash'), t('reset')))));
  async function load() {
    const st = await GET('/api/storage');
    const x = st.stats;
    fill(stats, [[x.lectures, 'statLectures'], [x.series, 'statSeries'], [x.quran, 'statQuran'], [x.films, 'statFilms'], [x.speakers, 'statSpeakers'], [x.files, 'statFiles']]
      .map(([v, l]) => h('div', { class: 'stat' }, h('b', { text: num(v) }), h('span', { text: t(l) }))),
    h('div', { class: 'stat' }, h('b', { text: num(Math.round((x.seconds || 0) / 3600)) }), h('span', { text: t('statHours') })),
    h('div', { class: 'stat' }, h('b', { text: size(st.db_bytes) }), h('span', { text: t('statDb') })));
    fill(kv, h('span', { text: t('dataFolder') }), h('code', { text: st.data_dir }), h('span', { text: t('settingsFile') }), h('code', { text: st.config_path }), h('span', { text: t('database') }), h('code', { text: st.db_path }));
  }
  await load();
}

function modal(content) {
  const back = h('div', { class: 'modal-back', onclick(e) { if (e.target === back) close(); } }, h('div', { class: 'modal' }, content));
  const onKey = (e) => { if (e.key === 'Escape') close(); };
  function close() { back.remove(); document.removeEventListener('keydown', onKey); }
  document.addEventListener('keydown', onKey);
  $('#overlay-root').append(back);
  return close;
}
function confirmBox(title, text, okLabel, danger = false) {
  return new Promise((resolve) => {
    const close = modal(h('div', {}, h('h2', { text: title }), h('p', { text }),
      h('div', { class: 'actions' },
        h('button', { class: 'btn', text: t('cancel'), onclick() { close(); resolve(false); } }),
        h('button', { class: 'btn ' + (danger ? 'danger-solid' : 'primary'), text: okLabel, onclick() { close(); resolve(true); } }))));
  });
}

// --------------------------------------------------------- assistant
const assistant = { open: false, messages: null, busy: false };
$('#fab').append(icon('sparkles'));
$('#fab').onclick = () => toggleAssistant();
async function toggleAssistant(force) {
  assistant.open = force ?? !assistant.open;
  $('#assistant').hidden = !assistant.open;
  $('#fab').classList.toggle('open', assistant.open);
  $('#fab').replaceChildren(icon(assistant.open ? 'x' : 'sparkles'));
  if (!assistant.open) return;
  if (assistant.messages === null) {
    assistant.messages = [];
    renderAssistant();
    try { assistant.messages = (await GET('/api/assistant')).messages; } catch { /* empty chat */ }
  }
  renderAssistant();
}
function renderAssistant() {
  const flags = app.state?.flags || {};
  const c = app.state?.counts || {};
  const bodyEl = h('div', { class: 'as-body' });
  const input = h('input', { placeholder: t('asPh'), disabled: assistant.busy || !flags.assistant });
  const send = h('button', { class: 'send', disabled: assistant.busy || !flags.assistant, onclick: () => ask(input.value) }, icon('send'));
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') ask(input.value); });
  if (!flags.assistant) {
    bodyEl.append(h('div', { class: 'as-intro' }, h('h3', { text: t('asSetupTitle') }), h('p', { text: t('asSetupText') }),
      h('a', { class: 'btn primary', href: '#/settings/assistant', onclick: () => toggleAssistant(false) }, icon('sliders'), t('asOpenSettings'))));
  } else if (!assistant.messages?.length) {
    bodyEl.append(h('div', { class: 'as-intro' }, h('h3', { text: t('asHello') }), h('p', { text: t('asHelloText', c.lectures || 0, c.series || 0, c.quran || 0) }),
      t('suggestions').map((x) => h('button', { class: 'suggestion', text: x, onclick: () => ask(x) }))));
  } else {
    for (const m of assistant.messages) {
      if (m.role === 'user') bodyEl.append(h('div', { class: 'msg-user', text: m.text }));
      else if (m.role === 'error') bodyEl.append(h('div', { class: 'msg-err', text: m.text }));
      else bodyEl.append(h('div', { class: 'msg-bot' }, m.text,
        m.picks?.length ? h('div', { class: 'picks' }, m.picks.map((p) => h('div', { class: 'pick', onclick: () => go('#/title/' + p.id) },
          p.poster ? h('img', { src: p.poster, alt: '', loading: 'lazy' }) : h('div', { class: 'ph' }),
          h('div', {}, h('b', { text: p.name }), h('span', { text: [p.speaker, p.total_duration && human(p.total_duration)].filter(Boolean).join(' · ') }))))) : null));
    }
    if (assistant.busy) bodyEl.append(h('div', { class: 'typing' }, h('i'), h('i'), h('i')));
  }
  fill($('#assistant'),
    h('div', { class: 'as-head' }, h('div', { class: 'as-icon' }, icon('sparkles')),
      h('div', { class: 'grow' }, h('b', { text: t('asTitle') }), h('small', { text: t('asSub', flags.model || 'Gemini') })),
      h('button', { class: 'icon-btn', onclick: () => toggleAssistant(false) }, icon('x'))),
    bodyEl, h('div', { class: 'as-foot' }, input, send));
  bodyEl.scrollTop = bodyEl.scrollHeight;
  if (!assistant.busy && flags.assistant) input.focus();
}
async function ask(text) {
  text = (text || '').trim();
  if (!text || assistant.busy) return;
  assistant.messages.push({ role: 'user', text });
  assistant.busy = true;
  renderAssistant();
  try { const r = await POST('/api/assistant', { message: text }); assistant.messages.push({ role: 'assistant', text: r.reply, picks: r.picks }); }
  catch (e) { assistant.messages.push({ role: 'error', text: e.message }); }
  assistant.busy = false;
  renderAssistant();
}

// ------------------------------------------------------- search & keys
(function setupTopbar() {
  $('.search-icon').append(icon('search'));
  $('#top-settings').append(icon('sliders'));
  $('#job-stop').append(icon('stop'));
  $('#add-source').append(icon('plus'));
  $('#add-source').onclick = () => go('#/settings/sources');
  $('#job-stop').onclick = async () => { await POST('/api/jobs/stop').catch(() => {}); toast(t('stopping')); refreshState(); };
  $('#lang-toggle').onclick = () => setLang(lang === 'bn' ? 'en' : 'bn');
  const input = $('#search');
  let timer = 0;
  input.addEventListener('input', () => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      const q = input.value.trim();
      const target = '#/search/' + encodeURIComponent(q);
      if (!q) { if (location.hash.startsWith('#/search/')) history.back(); return; }
      delete app.gridMemo[target];
      if (location.hash.startsWith('#/search/')) { history.replaceState(null, '', target); route(); }
      else location.hash = target;
    }, 220);
  });
  input.addEventListener('keydown', (e) => { if (e.key === 'Escape') { input.value = ''; input.blur(); if (location.hash.startsWith('#/search/')) history.back(); } });
  document.addEventListener('keydown', (e) => {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') { e.preventDefault(); input.focus(); input.select(); }
    else if (e.key === '/' && !/INPUT|TEXTAREA|SELECT/.test(document.activeElement.tagName)) { e.preventDefault(); input.focus(); }
  });
})();

window.addEventListener('hashchange', route);
applyStaticText();
refreshState().then(() => {
  // The saved setting wins over this browser's memory on first load.
  const saved = app.state?.flags?.language;
  if (saved && saved !== lang) { lang = saved; try { localStorage.setItem('noor.lang', lang); } catch { /* ignore */ } applyStaticText(); renderSidebar(); }
  route();
});
