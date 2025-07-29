/**
 * Hook for watching file system changes
 * Provides functionality to watch a directory for changes and trigger callbacks
 */
import { useEffect, useRef, useCallback } from 'react';
import { watch } from 'tauri-plugin-fs-watch-api';
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
  onChange: (res: string[]) => Promise<void>,
  options: UseFileWatcherOptions = {}
) {
  // Default options
  const {
    recursive = true,
    debounceDelay = 300
  } = options;

  // Use a ref to store the cleanup function
  const stopWatchingRef = useRef<(() => void) | null>(null);

  // Use a ref to track if we're currently initializing the cache
  // This helps prevent the file watcher from triggering during initialization
  const isInitializingRef = useRef<boolean>(false);

  // Create a debounced version of the onChange callback
  const guardedOnChange = useCallback(async (relevantChanges: string[]) => {
      // Only trigger the onChange callback if we're not currently initializing the cache
      if (!isInitializingRef.current) {
        await onChange(relevantChanges);
      }
    }, [onChange]);

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

            // Check if any of the changed files are relevant
            const relevantChanges = events
                .filter(event => isRelevantChange(event.path))
                .map(event => event.path);

            if (relevantChanges.length === 0) {
              return;
            }

            await guardedOnChange(relevantChanges);
          },
          { recursive: true, delayMs: debounceDelay }
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
    };
  }, [debounceDelay, folderPath, guardedOnChange, recursive]);

  // Return the setIsInitializing method to allow external components to control the flag
  return { setIsInitializing };
}
