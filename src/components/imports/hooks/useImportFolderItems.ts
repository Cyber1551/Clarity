import { useCallback, useEffect, useState } from "react";
import { get_items_in_import_folder } from "@/api/importApi";
import { get_media_item_by_rel_path } from "@/api/libraryApi";
import { type MediaItem } from "@/types/mediaTypes";
import { type JobCompletedPayload } from "@/types/eventTypes";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { notify } from "@/utils/notify";

interface UseImportFolderItemsResult {
    items: MediaItem[];
    isLoading: boolean;
}

/**
 * Loads media items for the currently selected import folder and keeps them
 * in sync with `library-changed` and `job-completed` events.
 *
 * Pass `null` to clear the items list.
 */
export function useImportFolderItems(folderName: string | null): UseImportFolderItemsResult {
    const [items, setItems] = useState<MediaItem[]>([]);
    const [isLoading, setIsLoading] = useState(false);

    const load = useCallback(async (name: string) => {
        setIsLoading(true);
        try {
            const next = await get_items_in_import_folder(name);
            setItems(next);
        } catch (e) {
            notify.error("Couldn't load import items", e);
        } finally {
            setIsLoading(false);
        }
    }, []);

    const refreshItem = useCallback(async (relPath: string) => {
        try {
            const item = await get_media_item_by_rel_path(relPath);
            if (!item) return;
            setItems(existing =>
                existing.map(current => (current.relPath === item.relPath ? item : current))
            );
        } catch (e) {
            notify.error("Couldn't refresh import item", e);
        }
    }, []);

    useEffect(() => {
        if (folderName) {
            void load(folderName);
        } else {
            setItems([]);
        }
    }, [folderName, load]);

    useTauriEvent("library-changed", () => {
        if (folderName) void load(folderName);
    });

    useTauriEvent<JobCompletedPayload>("job-completed", (event) => {
        if (useInterfaceStore.getState().activeTab !== "imports") return;
        if (!folderName) return;
        const relPath = event.payload?.relPath;
        if (!relPath) return;
        const prefix = `Imports/${folderName}/`;
        if (!relPath.startsWith(prefix)) return;
        void refreshItem(relPath);
    });

    return { items, isLoading };
}
