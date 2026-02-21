import { invoke } from "@tauri-apps/api/core";
import { MediaDetail, MediaItem } from "@/types/mediaTypes";
import { ThumbnailResponse } from "@/types/dtoTypes";

export async function initialize_library(): Promise<void> {
    await invoke("initialize_library");
}

export async function get_media_items(): Promise<MediaItem[]> {
    return await invoke("get_media_items");
}

export async function get_media_detail(mediaId: number): Promise<MediaDetail> {
    return await invoke("get_media_detail", { mediaId });
}

export async function get_media_item_by_rel_path(relPath: string): Promise<MediaItem | null> {
    return await invoke("get_media_item_by_rel_path", { relPath });
}

export async function get_thumbnail(hash: string): Promise<{ blob: Uint8Array; mimetype: string }> {
    const res = await invoke<ThumbnailResponse>("get_thumbnail", { hash });
    return {
        blob: new Uint8Array(res.blob),
        mimetype: res.mimetype
    };
}
