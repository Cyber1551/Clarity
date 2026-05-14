import { type MediaDetail, type MediaItem } from "@/types/mediaTypes";
import { type ThumbnailResponse } from "@/types/dtoTypes";
import { type BlobWithMime } from "@/types/binaryTypes.ts";

import { _invoke } from "./_invoke";

export async function initialize_library(): Promise<void> {
    await _invoke("initialize_library");
}

export async function get_media_items(): Promise<MediaItem[]> {
    return await _invoke("get_media_items");
}

export async function get_media_detail(mediaId: number): Promise<MediaDetail> {
    return await _invoke("get_media_detail", { mediaId });
}

export async function get_media_item_by_rel_path(relPath: string): Promise<MediaItem | null> {
    return await _invoke("get_media_item_by_rel_path", { relPath });
}

export async function get_thumbnail(hash: string): Promise<BlobWithMime> {
    const res = await _invoke<ThumbnailResponse>("get_thumbnail", { hash });
    return {
        blob: new Uint8Array(res.blob),
        mimetype: res.mimetype
    };
}

export async function mark_as_reviewed(mediaId: number): Promise<void> {
    await _invoke("mark_as_reviewed", { mediaId });
}

export async function review_and_promote(mediaId: number): Promise<void> {
    await _invoke("review_and_promote", { mediaId });
}

export async function update_quality_rating(mediaId: number, rating: number): Promise<void> {
    await _invoke("update_quality_rating", { mediaId, rating });
}

export async function update_favorite_rating(mediaId: number, rating: number): Promise<void> {
    await _invoke("update_favorite_rating", { mediaId, rating });
}

export async function toggle_loved(mediaId: number): Promise<boolean> {
    return await _invoke("toggle_loved", { mediaId });
}

export async function rename_media_file(fileId: number, newFileName: string): Promise<void> {
    await _invoke("rename_media_file", { fileId, newFileName });
}

export async function restart_workers(): Promise<void> {
    await _invoke("restart_workers");
}
