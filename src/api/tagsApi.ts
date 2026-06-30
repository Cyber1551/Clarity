import { type Tag } from "@/types/mediaTypes";
import { _invoke } from "./_invoke";

export async function list_tags(): Promise<Tag[]> {
    return await _invoke("list_tags");
}

export async function get_media_tags(mediaId: number): Promise<Tag[]> {
    return await _invoke("get_media_tags", { mediaId });
}

export async function add_media_tag(mediaId: number, name: string): Promise<Tag> {
    return await _invoke("add_media_tag", { mediaId, name });
}

export async function remove_media_tag(mediaId: number, tagId: number): Promise<void> {
    await _invoke("remove_media_tag", { mediaId, tagId });
}
