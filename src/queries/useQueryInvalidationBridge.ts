import { useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { get_media_item_by_rel_path } from "@/api/libraryApi";
import { type JobCompletedPayload } from "@/types/eventTypes";
import { type MediaItem } from "@/types/mediaTypes";
import { logger } from "@/utils/logger";
import { queryKeys } from "./keys";

const IMPORTS_FOLDER_RE = /^Imports\/([^/]+)\//;

/**
 * Single Tauri-event-to-query-invalidation bridge, mounted once from `App`. 
 * The `job-completed` handler splices the one changed item into cached lists rather than invalidating, so heavy import runs don't trigger full grid refetches.
 */
export function useQueryInvalidationBridge() {
    const queryClient = useQueryClient();
    const setLibraryReady = useInterfaceStore((s) => s.setLibraryReady);

    const spliceItemIntoList = useCallback(
        (queryKey: readonly unknown[], next: MediaItem) => {
            queryClient.setQueryData<MediaItem[]>(queryKey, (current) => {
                if (!current) return current;
                let touched = false;
                const updated = current.map((existing) => {
                    if (existing.relPath !== next.relPath) return existing;
                    touched = true;
                    return next;
                });
                return touched ? updated : current;
            });
        },
        [queryClient],
    );

    useTauriEvent("library-initialized", () => {
        logger.debug("query-bridge", "library initialized");
        setLibraryReady(true);
        void queryClient.invalidateQueries({ queryKey: queryKeys.library.items() });
    });

    useTauriEvent("library-changed", () => {
        logger.debug("query-bridge", "library changed - invalidating library + imports + sync domains");
        void queryClient.invalidateQueries({ queryKey: queryKeys.library.all() });
        void queryClient.invalidateQueries({ queryKey: queryKeys.imports.all() });
        void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
    });

    useTauriEvent<JobCompletedPayload>("job-completed", (event) => {
        const relPath = event.payload?.relPath;
        if (!relPath) return;

        const importMatch = IMPORTS_FOLDER_RE.exec(relPath);
        const importFolder = importMatch?.[1] ?? null;
        
        void (async () => {
            try {
                const item = await get_media_item_by_rel_path(relPath);
                if (!item) return;
                spliceItemIntoList(queryKeys.library.items(), item);
                if (importFolder) {
                    spliceItemIntoList(queryKeys.imports.folderItems(importFolder), item);
                }
            } catch {
                // `_invoke` already logged
            }
        })();
    });
}
