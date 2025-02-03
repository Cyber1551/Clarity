import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';
import {MediaItem} from "@/types/media_item.ts";

/**
 * Merges new media items with an existing .mediaCache.json file in the chosen folder.
 * Preserves user-added tags and bookmarks from previous sessions.
 * @param folder The folder in which the .mediaCache.json file resides.
 * @param newItems The array of newly scanned MediaItem objects.
 * @returns The merged array of MediaItem objects.
 */
export async function mergeWithExistingCache(
    folder: string,
    newItems: MediaItem[]
): Promise<MediaItem[]> {
    const cacheFilePath = `${folder}/.mediaCache.json`;
    let mergedItems: MediaItem[] = newItems;

    try {
        // Attempt to read the existing cache file.
        const cacheContent = await readTextFile(cacheFilePath);
        const existingItems: MediaItem[] = JSON.parse(cacheContent);

        // Create a map keyed by the file path from the old cache.
        const existingMap = new Map<string, MediaItem>();
        existingItems.forEach(item => {
            existingMap.set(item.path, item);
        });

        // For each new item, if it exists in the old cache, merge tags and bookmarks.
        mergedItems = newItems.map(item => {
            if (existingMap.has(item.path)) {
                const existingItem = existingMap.get(item.path)!;
                return { ...item, tags: existingItem.tags, bookmarks: existingItem.bookmarks };
            }
            return item;
        });

        // Files that are no longer present in the directory are removed from the cache.
    } catch (error) {
        // If the cache file doesn't exist or there was an error reading it, simply use the new items.
        console.log('No existing cache found or error reading cache; using new scan results.', error);
    }

    // Write the merged data back to the .mediaCache.json file.
    await writeTextFile(cacheFilePath, JSON.stringify(mergedItems, null, 2));
    return mergedItems;
}
