# Ingestion Pipeline

How files enter the library and are processed.

### 1. Discovery & Identity
- **Trigger**: User initiates an import via the UI.
- **Hashing**: App calculates the Blake3 hash of the source file.
- **Deduplication Check**: App queries the `media` table for an existing hash.

### 2. Ingestion (Copy to `.objects`)
- **First Time Seen**: File is copied directly from source to `.objects/[hash].[ext]`.
- **Duplicate Seen**: Source file is ignored; the canonical version already exists in `.objects`.

### 3. Projection
- **Path Generation**: Based on attributes (date, tags, quality, favorites), the app determines target paths in `Library` or `Imports`.
- **Hardlinking**: Physical hardlinks are created from the `.objects` file to the target paths.
- **Database Link**: `MediaLinkRow` records are inserted for each new path, pointing to the `MediaRow`.

### 4. Asynchronous Enrichment
- **Metadata Extraction**: A `Metadata` job is enqueued to probe for width, height, and duration.
- **Thumbnail Generation**: A `Thumbnail` job is enqueued to create a WebP preview stored in the DB.
- **UI Update**: Frontend refreshes to show the new item, showing a spinner if jobs are still processing.

### 5. Manual Content Updates
- **Edits/Crops**: If the app detects an `mtime` mismatch during a scan, it triggers a re-import.
- **Hash Update**: The modified file gets a new hash, a new `MediaRow`, and the link is updated to point to the new identity, preserving user metadata (like quality ratings).
