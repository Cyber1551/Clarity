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
    thumbnailDataUrl: string;
}

export interface TreeNode {
    dirName: string;
    path: string;
    children: TreeNode[];
}