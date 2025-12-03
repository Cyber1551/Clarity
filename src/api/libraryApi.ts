import { invoke } from "@tauri-apps/api/core";
import { MediaItem } from "@/types/mediaTypes";

export async function initialize_library(): Promise<void> {
    await invoke("initialize_library");
}

export async function get_all_media(): Promise<MediaItem[]> {
    return await invoke("get_all_media");
}
