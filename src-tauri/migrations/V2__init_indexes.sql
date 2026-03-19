-- `media`
CREATE INDEX IF NOT EXISTS idx_media_created_at ON media(created_at);
CREATE INDEX IF NOT EXISTS idx_media_hash_status ON media(hash_status) WHERE hash_status IN ('pending', 'error');
CREATE INDEX IF NOT EXISTS idx_media_metadata_status ON media(metadata_status) WHERE metadata_status IN ('pending', 'error');
CREATE INDEX IF NOT EXISTS idx_media_thumbnail_status ON media(thumbnail_status) WHERE thumbnail_status IN ('pending', 'error');

-- `media_links`
CREATE INDEX IF NOT EXISTS idx_media_links_media_id ON media_links(media_id);
CREATE INDEX IF NOT EXISTS idx_media_links_dir_path ON media_links(dir_path);
CREATE INDEX IF NOT EXISTS idx_media_links_ext ON media_links(ext);
CREATE INDEX IF NOT EXISTS idx_media_links_created_at ON media_links(created_at);
CREATE UNIQUE INDEX IF NOT EXISTS uniq_media_links_media_dir ON media_links(media_id, dir_path);

-- `jobs`
CREATE INDEX IF NOT EXISTS idx_jobs_status_priority_created ON jobs(status, priority, created_at);
CREATE INDEX IF NOT EXISTS idx_jobs_media_id ON jobs(media_id);
CREATE INDEX IF NOT EXISTS idx_jobs_file_id ON jobs(file_id);
CREATE INDEX IF NOT EXISTS idx_jobs_cleanup ON jobs(attempts, status);