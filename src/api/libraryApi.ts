import { invoke } from "@tauri-apps/api/core";
import { MediaDetail, MediaItem } from "@/types/mediaTypes";

export async function initialize_library(): Promise<void> {
    await invoke("initialize_library");
}

export async function get_media_items(): Promise<MediaItem[]> {
    return await invoke("get_media_items");
}

export async function get_media_detail(mediaId: number): Promise<MediaDetail> {
    return await invoke("get_media_detail", { mediaId });
}

export async function get_thumbnail(hash: string): Promise<Uint8Array> {
    const res = await invoke<number[]>("get_thumbnail", { hash });
    return new Uint8Array(res);
}
