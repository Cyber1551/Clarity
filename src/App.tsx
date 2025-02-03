import { useEffect, useState } from 'react';
import { loadConfig, saveConfig, AppConfig } from './api/configApi.ts';
import { pickFolder } from "@/api/fsApi.ts";
import {MediaItem} from "@/types/media_item.ts";
import {updateMediaCache} from "@/helpers/cacheHelper.ts";
import MediaGrid from "@/components/MediaGrid.tsx";
import {watch} from "tauri-plugin-fs-watch-api";
import {debounce} from "lodash-es";

function App() {
    // State to keep track of the selected folder path.
    const [folderPath, setFolderPath] = useState<string | null>(null);
    const [mediaItems, setMediaItems] = useState<MediaItem[]>([]);

    // Load the configuration on component mount.
    useEffect(() => {
        async function initConfig() {
            const config: AppConfig = await loadConfig();
            if (config.folderPath) {
                setFolderPath(config.folderPath);
                const items = await updateMediaCache(config.folderPath);
                setMediaItems(items);
            }
        }
        initConfig();
    }, []);

    // Handler for when the user clicks the button to pick a folder.
    const handlePickFolder = async () => {
        const selected = await pickFolder();
        if (selected) {
            setFolderPath(selected);
            // Save the new folder path to the config.
            await saveConfig({ folderPath: selected });

            const items = await updateMediaCache(selected);
            setMediaItems(items);
        }
    };

    // Set up the file watcher using tauri-plugin-fs-watch-api when a folder is selected.
    useEffect(() => {
        if (!folderPath) return;

        let stopWatching: () => void;

        // Debounced function for updating media cache and setting media items
        const debouncedUpdateMediaCache = debounce(async (folderPath: string) => {
            try {
                const items = await updateMediaCache(folderPath);
                setMediaItems(items);
            } catch (error) {
                console.error("Error updating media cache:", error);
            }
        }, 300); // Debounce delay of 300ms (adjustable)

        (async () => {
            try {
                stopWatching = await watch(
                    folderPath,
                    async (event) => {
                        if (event.length === 0) return;
                        if (event[0].path.endsWith('.mediaCache.json')) return;

                        console.log("File system event received:", event);

                        debouncedUpdateMediaCache(folderPath);
                    },
                    { recursive: true }
                );
            } catch (error) {
                console.error("Error setting up watcher:", error);
            }
        })();

        return () => {
            if (stopWatching) {
                stopWatching();
            }
        };
    }, [folderPath]);

    return (
        <div className="min-h-screen w-screen flex flex-col">
            {/* Top Menu Bar */}
            <nav className="w-full flex items-center justify-between bg-gray-100 border-b border-gray-300 px-4 py-2 fixed top-0 z-10">
                {/* Left Side: App Name */}
                <div>
                    <h2 className="text-lg font-bold">My Tauri App</h2>
                </div>

                {/* Right Side: Folder display and Pick/Change button */}
                <div className="flex items-center space-x-4">
                    <span className="text-sm text-gray-600">
                        {folderPath ? folderPath : 'No folder selected'}
                    </span>
                    <button
                        onClick={handlePickFolder}
                        className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring focus:ring-blue-300"
                    >
                        {folderPath ? 'Change Folder' : 'Pick Folder'}
                    </button>
                </div>
            </nav>

            {/* Main Content Area */}
            <main className="flex-grow pt-16 p-4 text-center">
                {!folderPath ? (
                    <p>No folder selected. Please click "Pick Folder" in the top menu.</p>
                ) : (
                    <>
                        <h3>Media Items</h3>
                        <MediaGrid mediaItems={mediaItems} />
                    </>
                )}
            </main>
        </div>
    );
}

export default App;