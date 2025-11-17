export type MediaType = "image" | "video";

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

    thumbUrl: string | null;
    thumbWidth: number | null;
    thumbHeight: number | null;
}

export interface TreeNode {
    dirName: string;
    path: string;
    children: TreeNode[];
}