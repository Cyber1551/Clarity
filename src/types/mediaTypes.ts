export type MediaType = "image" | "video" | "unknown";
export type JobStatus = "pending" | "processing" | "done" | "error";

// Media-level feed item (one row per media)
export interface MediaItem {
    mediaId: number;
    relPath: string | null;
    dirPath: string | null;
    fileName: string | null;
    displayName: string | null;
    ext: string | null;
    mediaType: MediaType;
    width: number | null;
    height: number | null;
    durationMs: number | null;
    qualityRating: number;
    favoriteRating: number;
    loved: boolean;
    hashStatus: JobStatus;
    metadataStatus: JobStatus;
    thumbnailStatus: JobStatus;
    contentHash: string;
    reviewedAt: number | null;
}

export interface Tag {
    id: number;
    name: string;
    slug: string;
}

/** Result of a projection sync/rebuild pass. */
export interface SyncReport {
    reconciled: number;
    created: number;
    removed: number;
}

/** How many reviewed items are dirty (pending projection to disk). */
export interface SyncStatus {
    dirtyCount: number;
}

export interface TreeNode {
    dirName: string;
    path: string;
    children: TreeNode[];
}

export interface MediaFileRef {
    id: number;
    relPath: string;
    dirPath: string;
    fileName: string;
    ext: string;
}

export interface MediaDetail {
    mediaId: number;
    contentHash: string;
    mediaType: MediaType;
    displayName: string | null;
    originalFileName: string | null;
    width: number | null;
    height: number | null;
    durationMs: number | null;
    qualityRating: number;
    favoriteRating: number;
    loved: boolean;
    sizeBytes: number;
    createdAt: number;
    reviewedAt: number | null;
    tags: Tag[];
    files: MediaFileRef[];
    canonicalPath: string;
}
