export type MediaType = "image" | "video" | "unknown";
export type JobStatus = "pending" | "processing" | "done" | "error";

// Media-level feed item (one row per media)
export interface MediaItem {
    mediaId: number;
    relPath: string | null;
    dirPath: string | null;
    fileName: string | null;
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
    width: number | null;
    height: number | null;
    durationMs: number | null;
    qualityRating: number;
    favoriteRating: number;
    loved: boolean;
    files: MediaFileRef[];
    canonicalPath: string;
}
