-- `media` table represents the content itself. Multiple `media_files` can point to the same media.
-- This row is the source of truth for an item's attributes; the `Library/` folder tree is a derived hardlink projection of these columns.
CREATE TABLE IF NOT EXISTS media (
    id                  INTEGER PRIMARY KEY,
    content_hash        TEXT NOT NULL UNIQUE,
    media_type          TEXT NOT NULL,
    display_name        TEXT,
    original_file_name  TEXT,
    width               INTEGER,
    height              INTEGER,
    duration_ms         INTEGER,
    quality_rating      INTEGER NOT NULL DEFAULT 0,
    favorite_rating     INTEGER NOT NULL DEFAULT 0,
    loved               INTEGER NOT NULL DEFAULT 0,
    reviewed_at         INTEGER,
    -- Last time this item's attributes were materialized into the Library projection.
    -- An item is "dirty" when reviewed_at IS NOT NULL AND (projected_at IS NULL OR updated_at > projected_at).
    projected_at        INTEGER,
    hash_status         TEXT NOT NULL DEFAULT 'pending',
    metadata_status     TEXT NOT NULL DEFAULT 'pending',
    thumbnail_status    TEXT NOT NULL DEFAULT 'pending',
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

-- `media_files` table represents each hardlink file reference to a media item on disk.
-- Does not contain files in the .objects folder as those are the source of truth files per content_hash.
-- Rows are created and removed by the projection reconcile step.
CREATE TABLE IF NOT EXISTS media_files (
    id                  INTEGER PRIMARY KEY,
    media_id            INTEGER NOT NULL REFERENCES media(id),
    rel_path            TEXT NOT NULL UNIQUE,
    dir_path            TEXT NOT NULL,
    file_name           TEXT NOT NULL,
    ext                 TEXT NOT NULL,
    size_bytes          INTEGER NOT NULL,
    mtime               INTEGER NOT NULL,
    last_seen_mtime     INTEGER NOT NULL,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

-- `tags` table holds the canonical set of user tags.
-- `name` is the user-facing label; `slug` is a filesystem-safe form used for the By Tag projection.
CREATE TABLE IF NOT EXISTS tags (
    id                  INTEGER PRIMARY KEY,
    name                TEXT NOT NULL UNIQUE,
    slug                TEXT NOT NULL UNIQUE,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

-- `media_tags` junction table assigns tags to media (many-to-many).
CREATE TABLE IF NOT EXISTS media_tags (
    media_id            INTEGER NOT NULL REFERENCES media(id) ON DELETE CASCADE,
    tag_id              INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at          INTEGER NOT NULL,
    PRIMARY KEY (media_id, tag_id)
);

-- `thumbnails` table hold all thumbnail data for each unique piece of media content.
CREATE TABLE IF NOT EXISTS thumbnails (
    content_hash        TEXT PRIMARY KEY REFERENCES media(content_hash) ON DELETE CASCADE,
    thumbnail_blob      BLOB NOT NULL,
    mimetype            TEXT NOT NULL,
    width               INTEGER NOT NULL,
    height              INTEGER NOT NULL,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);

-- `jobs` table holds all queued and in-progress jobs for hashing, image/video probing, and thumbnail generation
CREATE TABLE IF NOT EXISTS jobs (
    id                  INTEGER PRIMARY KEY,
    job_type            TEXT NOT NULL,
    media_id            INTEGER REFERENCES media(id) ON DELETE CASCADE,
    file_id             INTEGER,
    rel_path            TEXT,
    queued_mtime        INTEGER,
    priority            INTEGER NOT NULL DEFAULT 0,
    status              TEXT NOT NULL DEFAULT 'pending',
    attempts            INTEGER NOT NULL DEFAULT 0,
    last_error          TEXT,
    created_at          INTEGER NOT NULL,
    updated_at          INTEGER NOT NULL
);
