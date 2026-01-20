export type MediaType = "image" | "video" | "unknown";
export type JobStatus = "pending" | "processing" | "done" | "error";

export interface MediaItem {
    mediaId: number;
    fileId: number;

    relPath: string;
    dirPath: string;
    fileName: string;
    ext: string;

    mediaType: MediaType;
    width: number | null;
    height: number | null;
    durationMs: number | null;

    hashStatus: JobStatus;
    metadataStatus: JobStatus;
    thumbnailStatus: JobStatus;
    contentHash: string;
}

// Media-level feed item (one row per media)
export interface MediaFeedItem {
    mediaId: number;
    relPath: string | null;
    dirPath: string | null;
    fileName: string | null;
    ext: string | null;
    mediaType: MediaType;
    width: number | null;
    height: number | null;
    durationMs: number | null;
    hashStatus: JobStatus;
    metadataStatus: JobStatus;
    thumbnailStatus: JobStatus;
    contentHash: string;
    reviewedAt: number | null;
    tags: Tag[];
}

export interface TreeNode {
    dirName: string;
    path: string;
    children: TreeNode[];
}

export interface Tag {
    id: number;
    name: string;
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
    files: MediaFileRef[];
    tags: Tag[];
    canonicalPath: string;
}
