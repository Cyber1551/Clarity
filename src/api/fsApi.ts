/**
 * File system API module
 * Provides functions for interacting with the file system
 */
import { open } from "@tauri-apps/api/dialog";
import { withErrorHandling } from "@/utils/apiUtils";

/**
 * Opens a folder picker dialog and returns the selected folder path
 * @returns {Promise<string|null>} The selected folder path or null if cancelled
 */
export async function pickFolder(): Promise<string | null> {
  return withErrorHandling(
    async () => {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (Array.isArray(selected)) {
        // If multiple is false, we shouldn't get an array, but just in case:
        return selected[0] || null;
      }

      return selected as string | null;
    },
    "Error picking folder"
  );
}

/**
 * Checks if a directory should be skipped during scanning
 * @param {string} path - Path to check
 * @returns {boolean} True if the directory should be skipped
 */
// function shouldSkipDirectory(path: string): boolean {
//   const skipPatterns = [
//     ".thumbnails",
//     "node_modules",
//     ".git",
//     ".vscode"
//   ];
//
//   return skipPatterns.some(pattern => path.includes(pattern));
// }

/**
 * Creates a MediaItem object from a file entry
 * @param {FileEntry} entry - File entry from readDir
 * @returns {MediaItem|null} MediaItem object or null if not a media file
 */
// function createMediaItem(entry: FileEntry): MediaItem | null {
//   if (!entry.name) {
//     return null;
//   }
//
//   const isImage = isImageFile(entry.name);
//   const isVideo = isVideoFile(entry.name);
//
//   if (!isImage && !isVideo) {
//     return null;
//   }
//
//   const type = isImage ? 'image' : 'video';
//
//   return {
//     path: entry.path,
//     title: entry.name,
//     type,
//     length: type === 'video' ? 0 : null, // Use 0 as a placeholder for video length
//     thumbnail: null,
//     tags: [],
//     bookmarks: []
//   };
// }

/**
 * Processes a directory entry (file or subdirectory)
 * @param {FileEntry} entry - Directory entry to process
 * @returns {Promise<MediaItem[]>} Array of media items found
 */
// async function processDirectoryEntry(entry: FileEntry): Promise<MediaItem[]> {
//   // If the entry is a directory, recursively scan it
//   if (entry.children) {
//     if (shouldSkipDirectory(entry.path)) {
//       return [];
//     }
//
//     return await scanDirectory(entry.path);
//   }
//
//   // Otherwise, it's a file. Check if it's a media file
//   const mediaItem = createMediaItem(entry);
//   return mediaItem ? [mediaItem] : [];
// }

/**
 * Recursively scans a directory for media files
 * @param {string} directory - The folder path to scan
 * @returns {Promise<MediaItem[]>} Array of MediaItem objects for each recognized media file
 */
// export async function scanDirectory(directory: string): Promise<MediaItem[]> {
//   try {
//     // Read the immediate entries (files and subdirectories) in the directory
//     const entries = await readDir(directory, { recursive: false });
//
//     // Process each entry and collect the results
//     const mediaItemArrays = await Promise.all(
//       entries.map(entry => processDirectoryEntry(entry))
//     );
//
//     // Flatten the array of arrays into a single array
//     const mediaItems = mediaItemArrays.flat();
//
//     return mediaItems;
//   } catch (error) {
//     console.error(`Error scanning directory ${directory}:`, error);
//     return [];
//   }
// }
