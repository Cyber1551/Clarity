/**
 * Main application component
 * Orchestrates the application's functionality and layout
 */
import { useEffect } from 'react';
import { pickFolder } from "@/api/fsApi";
import { useConfig } from "@/hooks/useConfig";
import { useMediaCache, CacheAction } from "@/hooks/useMediaCache";
import { useFileWatcher } from "@/hooks/useFileWatcher";
import Header from "@/components/Header";
import MainContent from "@/components/MainContent";

/**
 * Main application component
 */
function App() {
  // Use custom hooks for configuration, media cache, and file watching
  const { config, updateConfig } = useConfig();
  const { mediaItems, cacheAction, initializeCache, refreshCache, cacheActionText } = useMediaCache();

  // Set up file watcher to refresh cache when files change
  const { setIsInitializing } = useFileWatcher(
    config.folderPath,
    async () => {
      if (config.folderPath) {
        try {
          // Set the flag to prevent file watcher from triggering during refresh
          setIsInitializing(true);

          // Refresh the cache
          await refreshCache(config.folderPath);
        } catch (error) {
          console.error("Error refreshing media cache:", error);
        } finally {
          // Always reset the flag, even if refresh fails
          setIsInitializing(false);
        }
      }
    }
  );

  // Load media items when the folder path changes
  useEffect(() => {
    async function loadInitialMedia() {
      if (config.folderPath) {
        try {
          // Set the flag to prevent file watcher from triggering during initialization
          setIsInitializing(true);

          // Initialize the cache
          await initializeCache(config.folderPath);
        } catch (error) {
          console.error("Error initializing media cache:", error);
        } finally {
          // Always reset the flag, even if initialization fails
          setIsInitializing(false);
        }
      }
    }

    loadInitialMedia();
  }, [config.folderPath, initializeCache, setIsInitializing]);

  /**
   * Handler for when the user clicks the button to pick a folder
   */
  const handlePickFolder = async () => {
    const selected = await pickFolder();
    if (selected) {
      try {
        // Save the new folder path to the config
        await updateConfig({ folderPath: selected });
      } catch (error) {
        console.error("Error updating configuration:", error);
      }
    }
  };

  return (
    <div className="min-h-screen w-screen flex flex-col">
      {/* Header with app title, folder selection, and cache status */}
      <Header
        folderPath={config.folderPath}
        cacheAction={cacheAction}
        cacheActionText={cacheActionText}
        onPickFolder={handlePickFolder}
      />

      {/* Main content area with media grid */}
      <MainContent
        folderPath={config.folderPath}
        mediaItems={mediaItems}
      />
    </div>
  );
}

export default App;
