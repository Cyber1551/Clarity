import { open } from "@tauri-apps/api/dialog";
import { readDir } from '@tauri-apps/api/fs';
import {MediaItem} from "@/types/media_item.ts";
import {isImageFile, isVideoFile} from "@/helpers/mediaHelper.ts";

export async function pickFolder(): Promise<string | null> {
    const selected = await open({
        directory: true,
        multiple: false,
    });
    if (Array.isArray(selected)) {
        // If multiple is false, we shouldn't get an array, but just in case:
        return selected[0] || null;
    }
    return selected as string | null;
}

/**
 * Recursively scans a directory for media files.
 * @param directory The folder path to scan.
 * @returns An array of MediaItem objects for each recognized media file.
 */
export async function scanDirectory(directory: string): Promise<MediaItem[]> {
    let mediaItems: MediaItem[] = [];

    // Read the immediate entries (files and subdirectories) in the directory.
    const entries = await readDir(directory, { recursive: false });

    for (const entry of entries) {
        // If the entry is a directory, recursively scan it.
        if (entry.children) {
            if (entry.path.endsWith(".thumbnails")) continue;
            // Some versions of Tauri return a `children` array for directories.
            // We assume here that if `children` exists, we perform an extra recursion.
            mediaItems = mediaItems.concat(await scanDirectory(entry.path));
        } else {
            // Otherwise, it’s a file. Check if it has a valid media extension.
            if (entry.name && (isImageFile(entry.name) || isVideoFile(entry.name))) {
                const type = isImageFile(entry.name) ? 'image' : 'video';
                const mediaItem: MediaItem = {
                    path: entry.path,
                    title: entry.name,
                    type,
                    length: type === 'video' ? 0 : null, // Use 0 as a placeholder for video length.
                    thumbnail: null,
                    tags: [],
                    bookmarks: []
                };
                mediaItems.push(mediaItem);
            }
        }
    }

    return mediaItems;
}
