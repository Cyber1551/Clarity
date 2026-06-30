import { type MediaItem } from "@/types/mediaTypes";
import { type SearchQuery } from "@/types/searchTypes";
import { _invoke } from "./_invoke";

export async function search_media(query: SearchQuery): Promise<MediaItem[]> {
    return await _invoke("search_media", { query });
}
