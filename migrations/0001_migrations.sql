-- Sources (plugins/providers)
CREATE TABLE IF NOT EXISTS sources (
  id            TEXT PRIMARY KEY,          -- e.g., "mangadex_plugin"
  version       TEXT NOT NULL,
  created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Media (generic: manga, anime, etc.)
CREATE TABLE IF NOT EXISTS media (
  id            TEXT PRIMARY KEY,          -- canonical local id (uuid/ulid)
  mediatype     TEXT NOT NULL CHECK (mediatype IN ('paged', 'audio', 'video', 'other')),
  title         TEXT NOT NULL,
  alt_titles    TEXT,                      -- JSON array of strings
  description   TEXT,
  cover_url     TEXT,
  tags          TEXT,                      -- JSON array of strings
  status        TEXT,                      -- e.g., ongoing/completed
  created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Mapping Media <-> Source-specific IDs
CREATE TABLE IF NOT EXISTS media_sources (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  media_id       TEXT NOT NULL,
  source_id      TEXT NOT NULL,
  external_id    TEXT NOT NULL,
  last_synced_at DATETIME,
  UNIQUE(media_id, source_id, external_id),
  FOREIGN KEY(media_id) REFERENCES media(id) ON DELETE CASCADE,
  FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
);

-- Units (e.g., chapters, episodes, sections)
CREATE TABLE IF NOT EXISTS units (
  id            TEXT PRIMARY KEY,
  media_id      TEXT NOT NULL,
  source_id     TEXT NOT NULL,
  external_id   TEXT NOT NULL,
  number_text   TEXT,                      -- raw unit number representation
  number_num    REAL,                      -- parsed float for sorting if available
  title         TEXT,
  lang          TEXT,
  group_label   TEXT,                      -- e.g., volume/season/part
  published_at  DATETIME,
  kind          TEXT NOT NULL CHECK (kind IN ('chapter', 'episode', 'section', 'other')),
  created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(media_id, source_id, external_id),
  FOREIGN KEY(media_id) REFERENCES media(id) ON DELETE CASCADE,
  FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE
);

-- Unit assets (e.g., pages, images, streams, files)
CREATE TABLE IF NOT EXISTS unit_assets (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  unit_id    TEXT NOT NULL,
  idx        INTEGER NOT NULL,             -- asset index starting at 1
  url        TEXT NOT NULL,
  mime       TEXT,
  width      INTEGER,
  height     INTEGER,
  kind       TEXT NOT NULL CHECK (kind IN ('page', 'image', 'audio', 'video', 'subtitle', 'file', 'other')),
  UNIQUE(unit_id, idx),
  FOREIGN KEY(unit_id) REFERENCES units(id) ON DELETE CASCADE
);

-- Search cache (persisted subset)
CREATE TABLE IF NOT EXISTS search_cache (
  key         TEXT PRIMARY KEY,            -- source|mediatype|query normalized
  payload     TEXT NOT NULL,               -- JSON (array of media summaries)
  created_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  expires_at  DATETIME NOT NULL
);

-- Media preferences/settings
CREATE TABLE IF NOT EXISTS media_prefs (
  media_id      TEXT PRIMARY KEY,
  download_path TEXT,
  created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY(media_id) REFERENCES media(id) ON DELETE CASCADE
);

-- Track per-unit progress (e.g., page position cache)
CREATE TABLE IF NOT EXISTS unit_progress (
  unit_id      TEXT PRIMARY KEY,
  media_id     TEXT NOT NULL,
  progress_idx INTEGER NOT NULL,           -- progress index (e.g., page number)
  total_count  INTEGER,                    -- total pages/assets
  updated_at   INTEGER NOT NULL DEFAULT (unixepoch()),
  FOREIGN KEY(unit_id) REFERENCES units(id) ON DELETE CASCADE,
  FOREIGN KEY(media_id) REFERENCES media(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_media_mediatype_title ON media(mediatype, title);
CREATE INDEX IF NOT EXISTS idx_media_sources_external ON media_sources(external_id, source_id);
CREATE INDEX IF NOT EXISTS idx_units_media_number ON units(media_id, number_num);
CREATE INDEX IF NOT EXISTS idx_units_source_external ON units(source_id, external_id);
CREATE INDEX IF NOT EXISTS idx_search_cache_exp ON search_cache(expires_at);
CREATE INDEX IF NOT EXISTS idx_media_prefs_path ON media_prefs(download_path);
CREATE INDEX IF NOT EXISTS idx_unit_progress_media ON unit_progress(media_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_unit_assets_unit_idx ON unit_assets(unit_id, idx);
CREATE UNIQUE INDEX IF NOT EXISTS idx_media_sources_source_external
ON media_sources(source_id, external_id);