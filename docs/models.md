# Core Data Models

## Backend Structs (Rust)

### MediaRow (`src-tauri\src\media\model.rs`)
The canonical identity of a piece of content.
- **Identity**: Bound to `content_hash` (Blake3).
- **Deduplication**: One row per unique file content, regardless of filesystem presence.
- **Storage Reference**: Used to derive the path in `.objects/`.
- **Status Tracking**: Manages state for asynchronous processing (`hash`, `metadata`, `thumbnail`).

### MediaFileRow (`src-tauri\src\media_files\model.rs`)
A filesystem projection of a `MediaRow`.
- **Context**: Maps a `MediaRow` to a specific user-visible path (e.g., `Library/ByTag/Beach/video.mp4`).
- **Syncing**: Stores `mtime` and `size_bytes` to detect external modifications without re-hashing.
- **Cleanup**: `last_seen_mtime` helps identify files deleted manually while the app was closed.

### MediaItem (`src-tauri\src\media\model.rs`)
A composite domain object for gallery rendering.
- **Composition**: Combines `MediaRow` metadata with a "representative" `MediaFileRow` path.
- **Usage**: Primary unit returned by `get_media_items`.

### JobRow (`src-tauri\src\jobs\model.rs`)
Persistence for the background task queue.
- **Lifecycle**: `pending` -> `processing` -> `done` (deleted) or `error`.
- **Retries**: Tracks `attempts` and `last_error` for automated recovery.

## Data Transfer Objects (`src-tauri\src\commands\dto.rs`)

### MediaItemDto
Flattened representation of `MediaItem` optimized for frontend JSON consumption.

### MediaDetailDto
Comprehensive data for the `Viewer`. Includes the `MediaRow` attributes and a collection of all associated `MediaFileRow` entries (all paths where this media exists).
