import { type MediaItem } from "@/types/mediaTypes";
import { create } from "zustand";
import { get_media_items, get_media_item_by_rel_path } from "@/api/libraryApi";
import { formatError } from "@/utils/format";
import { notify } from "@/utils/notify";

interface MediaStoreState {
    items: MediaItem[];
    isLoading: boolean;
    error: string | null;
    highlightedMediaId: number | null;
    scrollTargetMediaId: number | null;
    setIsLoading: (isLoading: boolean) => void;
    loadAllMedia: () => Promise<void>;
    highlightMedia: (mediaId: number) => void;
    refreshItemByRelPath: (relPath: string) => Promise<void>;
    setScrollTargetMediaId: (mediaId: number | null) => void;
    clearHighlight: () => void;
}

export const useMediaStore = create<MediaStoreState>((set, get) => ({
    items: [],
    isLoading: true,
    error: null,
    highlightedMediaId: null,
    scrollTargetMediaId: null,

    setIsLoading(isLoading) {
        set({ isLoading });
    },

    async loadAllMedia() {
        const currentState = get();
        // Only show loading spinner on initial load, not on event-driven refreshes
        const isInitialLoad = currentState.items.length === 0;

        if (isInitialLoad) {
            set({ isLoading: true, error: null });
        }

        try {
            const items = await get_media_items();
            set({
                items,
                isLoading: false,
            });
        } catch (err) {
            // `_invoke` already logged the failure; `MediaGrid` reads this `error` field and renders the inline empty/error state.
            set({
                error: formatError(err, "Failed to load media."),
                isLoading: false,
            });
        }
    },

    highlightMedia(mediaId) {
        set({ highlightedMediaId: mediaId });
    },

    async refreshItemByRelPath(relPath) {
        try {
            const item = await get_media_item_by_rel_path(relPath);
            if (!item) return;
            set(state => ({
                items: state.items.map(existing =>
                    existing.relPath === item.relPath ? item : existing
                ),
            }));
        } catch (err) {
            // Background refresh; no inline UI to surface the failure, so a toast is the only place the user finds out.
            notify.error("Couldn't refresh media item", err);
        }
    },

    setScrollTargetMediaId(mediaId) {
        set({ scrollTargetMediaId: mediaId });
    },

    clearHighlight() {
        set({ highlightedMediaId: null });
    },
}));
