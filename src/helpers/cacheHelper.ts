import {mergeWithExistingCache} from "@/api/cacheApi.ts";
import {scanDirectory} from "@/api/fsApi";
import {MediaItem} from "@/types/media_item.ts";

/**
 * Scans the chosen folder for media files and updates (or creates) the .mediaCache.json file.
 * @param folder The folder to scan.
 * @returns The merged array of MediaItem objects.
 */
export async function updateMediaCache(folder: string): Promise<MediaItem[]> {
    const scannedItems = await scanDirectory(folder);
    return await mergeWithExistingCache(folder, scannedItems);
}
