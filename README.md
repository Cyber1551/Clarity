# Clarity

**Absolute control over your media library, powered by filesystem-first portability.**

<img width="1392" height="966" alt="Clarity Main UI" src="https://github.com/user-attachments/assets/4187d52d-9783-4805-936c-5181c2926bad" />

Clarity is a high-performance, local-first media management application designed to give you absolute control over your library. Unlike traditional media managers that lock your organization inside a proprietary database, Clarity's core philosophy is **filesystem-first portability**. 

It ensures that your organization is never dependent on the app itself by using hardlinks to mirror your library's structure on the physical disk. Whether you are using the app or browsing via a standard file explorer, your files remain organized, deduplicated, and instantly accessible.

---

## Key Features

### Performance-Driven Ingest
*   **Fast Initial Ingest**: A two-phase pipeline separates filesystem scanning from intensive processing. The UI populates instantly after a scan, while heavy tasks run in the background.
*   **Parallel Background Processing**: A multi-threaded job system leverages your CPU cores to handle hashing, metadata extraction, and WebP thumbnail generation in parallel, significantly accelerating library ingestion.

### Filesystem-Level Organization
*   **Hardlink-Based Management**: Uses content-based hashing (BLAKE3) and hardlinks to ensure that your physical file structure reflects your organization. This means your data remains organized and portable even if you stop using the app.
*   **Intelligent Deduplication**: Maintains a single canonical copy in an internal `.objects` store. Hardlinks allow the same file to exist in multiple virtual folders without consuming extra disk space.

### Real-time Synchronization
*   **Filesystem Sync**: Integrated file watching automatically detects additions, moves, or deletions, keeping the database and the physical filesystem in perfect sync.
*   **Smart Garbage Collection**: Automatically cleans up orphaned media entries and internal object storage when files are deleted, ensuring your storage remains lean.

### Seamless Media Experience
*   **Virtualized Media Grid**: Optimized to handle thousands of items with ease, using virtualization to maintain high frame rates even when scrolling through massive collections.
*   **High-Performance Thumbnail Streaming**: Uses a custom protocol to stream thumbnails directly from the database, bypassing expensive IPC overhead and ensuring instant loading.
*   **Intuitive Tagging**: Organize your collection with a tagging system that works in harmony with your filesystem structure.
*   **High-Fidelity Viewer**: Inspect your media with a dedicated, high-performance viewer designed for detail.

<img width="1392" height="966" alt="Clarity Media Viewer" src="https://github.com/user-attachments/assets/8415f464-738c-4df3-add5-5687e86c8ba0" />
*The work-in-progress viewer interface provides detailed media inspection.*

---

## How It Works

Clarity operates in a two-phase process to ensure the UI remains snappy even with massive libraries:

1.  **Phase 1 (Reconciliation)**: The app scans your "Unsorted Media" directory. It identifies new or modified files based on metadata and updates the database immediately.
2.  **Phase 2 (Background Jobs)**: Multi-threaded parallel workers pick up pending jobs. They hash files, link them to the `.objects` store, extract technical metadata, and generate thumbnails.
3.  **UI Feedback**: The React frontend listens for throttled backend events, updating the grid efficiently as thumbnails and metadata become available.

---

## Tech Stack

*   **Backend**: [Rust](https://www.rust-lang.org/) & [Tauri 2](https://tauri.app/) (for cross-platform desktop integration).
*   **Frontend**: [React 19](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), [Vite](https://vitejs.dev/), [Chakra UI](https://chakra-ui.com/), and [Tailwind CSS](https://tailwindcss.com/).
*   **Database**: [SQLite](https://www.sqlite.org/) with WAL (Write-Ahead Logging) mode and connection pooling.
*   **State Management**: [Zustand](https://github.com/pmndrs/zustand).

---

## Architecture Highlights

Clarity follows a strict layered architecture:
*   **Pure Layers**: Dedicated modules for database operations (SQL-only) and filesystem operations, ensuring a clean separation of concerns.
*   **Orchestration Layers**: Specialized modules that coordinate complex flows like library reconciliation and background job execution.
*   **Parallel Job Workers**: A multi-threaded worker pool that claims and processes jobs (hashing, metadata, thumbnails) concurrently, maximizing hardware utilization while ensuring data integrity.
*   **Asset Protocol**: A custom URI scheme for efficient binary data streaming, offloading heavy media loading from the primary command IPC.

---

## Development

To get started with development:

1.  **Install dependencies**:
    ```bash
    pnpm install
    ```

2.  **Run in development mode**:
    ```bash
    pnpm tauri dev
    ```

3.  **Build the application**:
    ```bash
    pnpm tauri build
    ```
