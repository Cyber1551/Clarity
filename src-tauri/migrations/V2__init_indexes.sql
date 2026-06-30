-- `media`
-- Powers ORDER BY m.created_at DESC in get_media_items / get_media_items_in_dir (gallery sort).
CREATE INDEX IF NOT EXISTS idx_media_created_at ON media(created_at);

-- Powers the library gallery filter (get_media_items WHERE reviewed_at IS NOT NULL) and the
-- imports queue filter (get_media_items_in_dir WHERE reviewed_at IS NULL).
CREATE INDEX IF NOT EXISTS idx_media_reviewed_at ON media(reviewed_at);

-- `media_files`
-- Powers WHERE dir_path = ? (get_media_items_in_dir) and WHERE dir_path LIKE 'prefix%'
-- (remove_deleted_files_in_dir_like, the Library%-scoped subquery in get_media_items).
CREATE INDEX IF NOT EXISTS idx_media_files_dir_path ON media_files(dir_path);

-- Enforces "one hardlink per (media, directory)" invariant. Also serves as the
-- composite index for any media_id-only lookup via leftmost-prefix (list_by_media_id,
-- list_by_media_in_dir_like, the orphan NOT EXISTS check on media_files.media_id, etc.).
CREATE UNIQUE INDEX IF NOT EXISTS uniq_media_files_media_dir ON media_files(media_id, dir_path);

-- `jobs`
-- Powers claim_next_pending: filters status IN ('pending','error') and walks
-- the index backwards for ORDER BY priority DESC, created_at DESC.
CREATE INDEX IF NOT EXISTS idx_jobs_status_priority_created ON jobs(status, priority, created_at);

-- Required for the FK `jobs.media_id REFERENCES media(id) ON DELETE CASCADE`.
-- Without this, every media row deletion does a full jobs scan to find children.
CREATE INDEX IF NOT EXISTS idx_jobs_media_id ON jobs(media_id);

-- Powers cleanup_failed_jobs: WHERE status = 'error' AND attempts >= ?.
-- Equality column (status) leads, range column (attempts) follows for an efficient seek.
CREATE INDEX IF NOT EXISTS idx_jobs_cleanup ON jobs(status, attempts);

-- `media_tags`
-- Reverse lookup (all media for a tag) and the FK `media_tags.tag_id REFERENCES tags(id)`.
-- The (media_id, tag_id) PRIMARY KEY already covers media_id-leading lookups (tags for a media).
CREATE INDEX IF NOT EXISTS idx_media_tags_tag_id ON media_tags(tag_id);
