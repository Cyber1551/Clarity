/**
 * Hook for watching file system changes
 * Provides functionality to watch a directory for changes and trigger callbacks
 */
import { useEffect, useRef, useCallback } from 'react';
import { watch } from 'tauri-plugin-fs-watch-api';
import { debounce } from 'lodash-es';
import { isImageFile, isVideoFile } from '@/helpers/mediaHelper';

/**
 * Options for the file watcher
 */
interface UseFileWatcherOptions {
  /**
   * Whether to watch recursively (include subdirectories)
   */
  recursive?: boolean;
  /**
   * Debounce delay in milliseconds
   */
  debounceDelay?: number;
  /**
   * Paths to ignore (will not trigger the callback)
   */
  ignorePaths?: string[];
}

/**
 * Custom hook for watching file system changes
 * @param {string|null} folderPath - Path to the folder to watch
 * @param {Function} onChange - Callback function to call when relevant changes are detected
 * @param {UseFileWatcherOptions} options - Options for the file watcher
 */
export function useFileWatcher(
  folderPath: string | null,
  onChange: () => Promise<void>,
  options: UseFileWatcherOptions = {}
) {
  // Default options
  const {
    recursive = true,
    debounceDelay = 300,
    ignorePaths = [
      '.mediaCache.json', 
      '.thumbnails', 
      'media_cache.db',
      'cache',
      'thumbnail',
      '.db',
      '.db-journal',
      '.db-shm',
      '.db-wal'
    ]
  } = options;

  // Use a ref to store the cleanup function
  const stopWatchingRef = useRef<(() => void) | null>(null);

  // Use a ref to track if we're currently initializing the cache
  // This helps prevent the file watcher from triggering during initialization
  const isInitializingRef = useRef<boolean>(false);

  // Create a debounced version of the onChange callback
  const debouncedOnChange = useCallback(
    debounce(async () => {
      // Only trigger the onChange callback if we're not currently initializing the cache
      if (!isInitializingRef.current) {
        await onChange();
      }
    }, debounceDelay),
    [onChange, debounceDelay]
  );

  // Method to set the initializing flag
  const setIsInitializing = useCallback((isInitializing: boolean) => {
    isInitializingRef.current = isInitializing;
  }, []);

  // Set up the file watcher when the folder path changes
  useEffect(() => {
    // If no folder path is provided, do nothing
    if (!folderPath) return;

    // Function to check if a file change is relevant (media file or directory)
    const isRelevantChange = (path: string): boolean => {
      // Get the filename from the path
      const fileName = path.split('\\').pop() || '';

      // Check if the file or directory should be ignored
      // Ignore specific file extensions
      if (fileName.endsWith('.db') || 
          fileName.endsWith('.db-journal') || 
          fileName.endsWith('.db-shm') || 
          fileName.endsWith('.db-wal') ||
          fileName === '.mediaCache.json' ||
          fileName === 'media_cache.db') {
        return false;
      }

      // Ignore specific directories
      if (path.includes('\\.thumbnails\\') || 
          path.includes('\\cache\\') ||
          path.includes('\\thumbnails\\')) {
        return false;
      }

      // Check if any of the ignore patterns match
      if (ignorePaths.some(ignorePath => path.includes(ignorePath))) {
        return false;
      }

      // Check if it's an image or video file
      if (isImageFile(fileName) || isVideoFile(fileName)) {
        return true;
      }

      // Check if it's a directory (no file extension)
      const hasExtension = fileName.includes('.');
      return !hasExtension;
    };

    // Set up the watcher
    (async () => {
      try {
        // Start watching the folder
        stopWatchingRef.current = await watch(
          folderPath,
          async (events) => {
            if (events.length === 0) return;

            // Check if any of the events should be ignored
            if (events.some(event => 
              ignorePaths.some(ignorePath => event.path.includes(ignorePath))
            )) {
              return;
            }

            // Check if any of the changed files are relevant
            const relevantChanges = events.filter(event => isRelevantChange(event.path));

            if (relevantChanges.length === 0) {
              return;
            }
            await debouncedOnChange();
          },
          { recursive }
        );
      } catch (error) {
        console.error("Error setting up file watcher:", error);
      }
    })();

    // Clean up the watcher when the component unmounts or the folder path changes
    return () => {
      if (stopWatchingRef.current) {
        stopWatchingRef.current();
        stopWatchingRef.current = null;
      }

      debouncedOnChange.cancel();
    };
  }, [folderPath, debouncedOnChange, recursive, ignorePaths]);

  // Return the setIsInitializing method to allow external components to control the flag
  return { setIsInitializing };
}
