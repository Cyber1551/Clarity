import { invoke } from '@tauri-apps/api/tauri';
import { MediaItem } from "@/types/media_item.ts";
import { Bookmark } from "@/types/bookmark.ts";

// Interface matching the Rust VideoMetadata struct
interface VideoMetadata {
    id: number;
    thumbnail_id: number;
    duration: number;
}

// Interface matching the Rust MediaItemResponse struct
interface MediaItemResponse {
    id: number;
    path: string;
    title: string;
    media_type: string;
    length: number | null;
    thumbnail_id: number | null; // ID of the thumbnail in the thumbnails table
    tags: string[];
    bookmarks: Bookmark[];
}

/**
 * Retrieves all media items from the database.
 * @returns Promise resolving to an array of MediaItem objects.
 */
export async function getAllMedia(): Promise<MediaItem[]> {
    try {
        const response = await invoke<MediaItemResponse[]>('get_all_media');

        // Convert the response to MediaItem objects
        return response.map(item => ({
            path: item.path,
            title: item.title,
            type: item.media_type as "image" | "video",
            thumbnail: item.thumbnail_id ? `thumbnail://${item.thumbnail_id}` : null, // Use the thumbnail ID for the thumbnail URL
            length: item.length,
            tags: item.tags,
            bookmarks: item.bookmarks
        }));
    } catch (error) {
        console.error('Error retrieving media from database:', error);
        return [];
    }
}

/**
 * Adds a tag to a media item.
 * @param mediaId The ID of the media item.
 * @param tagName The name of the tag to add.
 * @returns Promise resolving when the tag is added.
 */
export async function addTag(mediaId: number, tagName: string): Promise<void> {
    try {
        await invoke('add_tag', { mediaId, tagName });
    } catch (error) {
        console.error(`Error adding tag "${tagName}" to media ${mediaId}:`, error);
        throw error;
    }
}

/**
 * Adds a bookmark to a media item.
 * @param mediaId The ID of the media item.
 * @param description The description of the bookmark.
 * @param timestamp The timestamp of the bookmark in seconds.
 * @returns Promise resolving to the ID of the new bookmark.
 */
export async function addBookmark(mediaId: number, description: string, timestamp: number): Promise<number> {
    try {
        return await invoke<number>('add_bookmark', { mediaId, description, timestamp });
    } catch (error) {
        console.error(`Error adding bookmark to media ${mediaId}:`, error);
        throw error;
    }
}

/**
 * Extracts metadata from a video file, including generating a thumbnail.
 * @param path The path to the video file.
 * @returns Promise resolving to the video metadata.
 */
export async function extractVideoMetadata(path: string): Promise<VideoMetadata> {
    try {
        return await invoke<VideoMetadata>('extract_video_metadata', { path });
    } catch (error) {
        console.error(`Error extracting metadata from video ${path}:`, error);
        throw error;
    }
}

/**
 * Extracts metadata from an image file, including generating a thumbnail.
 * @param path The path to the image file.
 * @returns Promise resolving to the image metadata.
 */
export async function extractImageMetadata(path: string): Promise<VideoMetadata> {
    try {
        return await invoke<VideoMetadata>('extract_image_metadata', { path });
    } catch (error) {
        console.error(`Error extracting metadata from image ${path}:`, error);
        throw error;
    }
}

/**
 * Deletes a media item and all its associated data from the database.
 * @param path The path of the media item to delete.
 * @returns Promise resolving to true if the item was deleted, false if it wasn't found.
 */
export async function deleteMediaItem(path: string): Promise<boolean> {
    try {
        return await invoke<boolean>('delete_media_item_by_path', { path });
    } catch (error) {
        console.error(`Error deleting media item ${path}:`, error);
        throw error;
    }
}

/**
 * Updates the path of a media item in the database.
 * @param oldPath The current path of the media item.
 * @param newPath The new path of the media item.
 * @returns Promise resolving to true if the item was updated, false if it wasn't found.
 */
export async function updateMediaItemPath(oldPath: string, newPath: string): Promise<boolean> {
    try {
        return await invoke<boolean>('update_media_item_path', { oldPath, newPath });
    } catch (error) {
        console.error(`Error updating media item path from ${oldPath} to ${newPath}:`, error);
        throw error;
    }
}

/**
 * Gets a thumbnail by its ID.
 * @param thumbnailId The ID of the thumbnail.
 * @returns Promise resolving to a data URL containing the thumbnail image.
 */
export async function getThumbnailById(thumbnailId: number): Promise<string> {
    try {
        console.log(`Getting thumbnail by ID: ${thumbnailId}`);
        return await invoke<string>('get_thumbnail_by_id', { thumbnailId });
    } catch (error) {
        console.error(`Error getting thumbnail by ID ${thumbnailId}:`, error);
        throw error;
    }
}
