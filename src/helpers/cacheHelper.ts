/**
 * Cache helper module
 * Provides functions for managing the media cache database
 */
import { getAllMedia, extractVideoMetadata, extractImageMetadata, deleteMediaItem, updateMediaItemPath } from "@/api/cacheApi";
import { scanDirectory } from "@/api/fsApi";
import { MediaItem } from "@/types/media_item";
import { invoke } from '@tauri-apps/api/tauri';
import { exists } from '@tauri-apps/api/fs';

/**
 * Initialize the database in the specified folder
 * @param {string} folderPath - Path to the folder where the database should be initialized
 */
export async function initializeDatabase(folderPath: string): Promise<void> {
  try {
    await invoke('init_database', { folderPath });
  } catch (error) {
    console.error("Error initializing database:", error);
    throw error;
  }
}

/**
 * Create maps of media items for quick lookup
 * @param {MediaItem[]} items - Array of media items
 * @returns {Object} Maps of media items by path and filename
 */
function createMediaItemMaps(items: MediaItem[]): {
  byPath: Map<string, MediaItem>;
  byFilename: Map<string, MediaItem>;
} {
  const byPath = new Map<string, MediaItem>();
  const byFilename = new Map<string, MediaItem>();

  for (const item of items) {
    byPath.set(item.path, item);
    const filename = item.path.split('\\').pop() || '';
    byFilename.set(filename, item);
  }

  return { byPath, byFilename };
}

/**
 * Handle deleted and renamed files
 * @param {MediaItem[]} existingItems - Array of existing media items
 * @param {Map<string, MediaItem>} existingItemsMap - Map of existing items by path
 * @param {Map<string, MediaItem>} scannedItemsByFilename - Map of scanned items by filename
 * @returns {Object} Counts of deleted and renamed items
 */
async function handleDeletedAndRenamedFiles(
  existingItems: MediaItem[],
  existingItemsMap: Map<string, MediaItem>,
  scannedItemsByFilename: Map<string, MediaItem>
): Promise<{ deletedCount: number; renamedCount: number }> {
  let deletedCount = 0;
  let renamedCount = 0;

  for (const existingItem of existingItems) {
    const fileExists = await exists(existingItem.path);
    if (!fileExists) {
      // Check if this might be a renamed file
      const filename = existingItem.path.split('\\').pop() || '';
      const scannedItem = scannedItemsByFilename.get(filename);

      if (scannedItem && !existingItemsMap.has(scannedItem.path)) {
        // This is likely a renamed file (same filename, different path)
        try {
          await updateMediaItemPath(existingItem.path, scannedItem.path);
          renamedCount++;
        } catch (error) {
          console.error(`Error updating media item path from ${existingItem.path} to ${scannedItem.path}:`, error);
        }
      } else {
        // File was deleted
        try {
          await deleteMediaItem(existingItem.path);
          deletedCount++;
        } catch (error) {
          console.error(`Error deleting media item ${existingItem.path}:`, error);
        }
      }
    }
  }

  return { deletedCount, renamedCount };
}

/**
 * Process media files to extract metadata
 * @param {MediaItem[]} scannedItems - Array of scanned media items
 * @returns {Object} Counts of processed videos and images
 */
async function processMediaFiles(scannedItems: MediaItem[]): Promise<{
  processedVideoCount: number;
  processedImageCount: number;
}> {
  let processedVideoCount = 0;
  let processedImageCount = 0;

  for (const item of scannedItems) {
    if (item.type === 'video') {
      try {
        // Extract metadata and store in database
        await extractVideoMetadata(item.path);
        processedVideoCount++;
      } catch (error) {
        console.error(`Error processing video ${item.path}:`, error);
      }
    } else if (item.type === 'image') {
      try {
        // Extract metadata and store in database
        await extractImageMetadata(item.path);
        processedImageCount++;
      } catch (error) {
        console.error(`Error processing image ${item.path}:`, error);
      }
    }
  }

  return { processedVideoCount, processedImageCount };
}

/**
 * Scans the chosen folder for media files and updates the database.
 * For each media file, extracts metadata and stores it in the database.
 * @param {string} folder - The folder to scan
 * @returns {Promise<MediaItem[]>} The array of MediaItem objects from the database
 */
export async function updateMediaCache(folder: string): Promise<MediaItem[]> {
  // Get all existing media items from the database
  const existingItems = await getAllMedia();

  // Create maps for quick lookup
  const { byPath: existingItemsMap } = createMediaItemMaps(existingItems);

  // Scan the directory for media files
  const scannedItems = await scanDirectory(folder);

  // Create maps for quick lookup
  const { byFilename: scannedItemsByFilename } = createMediaItemMaps(scannedItems);

  // Handle deleted and renamed files
  await handleDeletedAndRenamedFiles(existingItems, existingItemsMap, scannedItemsByFilename);

  // Process media files to extract metadata
  await processMediaFiles(scannedItems);

  // Return all media items from the database
  const mediaItems = await getAllMedia();
  return mediaItems;
}
