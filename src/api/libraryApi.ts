import { invoke } from "@tauri-apps/api/core";
import { MediaDetail, MediaItem, MediaFeedItem, Tag } from "@/types/mediaTypes";

export async function initialize_library(): Promise<void> {
    await invoke("initialize_library");
}

export async function get_all_media(): Promise<MediaItem[]> {
    return await invoke("get_all_media");
}

export async function get_media_feed(): Promise<MediaFeedItem[]> {
    return await invoke("get_media_feed");
}

export async function get_media_detail(mediaId: number): Promise<MediaDetail> {
    return await invoke("get_media_detail", { mediaId });
}

export async function get_thumbnail(hash: string): Promise<Uint8Array> {
    const res = await invoke<number[]>("get_thumbnail", { hash });
    return new Uint8Array(res);
}

export async function list_tags(): Promise<Tag[]> {
    return await invoke("list_tags");
}

export async function create_tag(name: string): Promise<Tag> {
    return await invoke("create_tag", { req: { name } });
}

export async function tag_media(mediaId: number, tagId: number): Promise<void> {
    await invoke("tag_media", { req: { mediaId, tagId } });
}

export async function untag_media(mediaId: number, tagId: number): Promise<void> {
    await invoke("untag_media", { req: { mediaId, tagId } });
}

export async function mark_media_reviewed(mediaId: number): Promise<void> {
    await invoke("mark_media_reviewed", { mediaId });
}
