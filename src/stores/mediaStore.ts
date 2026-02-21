import { MediaItem } from "@/types/mediaTypes";
import { create } from "zustand";
import { get_media_items, get_media_item_by_rel_path } from "@/api/libraryApi";

export type ViewerState = null | { mediaId: number; };

type MediaStoreState = {
    items: MediaItem[];
    isLoading: boolean;
    error: string | null;
    viewer: ViewerState;
    highlightedMediaId: number | null;
    highlightUntil: number | null;
    scrollTargetMediaId: number | null;
    setIsLoading: (isLoading: boolean) => void;
    loadAllMedia: () => Promise<void>;
    openViewer: (mediaId: number) => void;
    closeViewer: () => void;
    highlightMedia: (mediaId: number) => void;
    refreshItemByRelPath: (relPath: string) => Promise<void>;
    setScrollTargetMediaId: (mediaId: number | null) => void;
    clearHighlight: () => void;
};

export const useMediaStore = create<MediaStoreState>((set, get) => ({
    items: [],
    isLoading: true,
    error: null,
    viewer: null,
    highlightedMediaId: null,
    highlightUntil: null,
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
            console.error("Failed to load media", err);
            set({
                error: err?.toString?.() ?? "Failed to load media.",
                isLoading: false,
            });
        }
    },

    openViewer(mediaId) {
        set({ viewer: { mediaId } });
    },

    closeViewer() {
        set({ viewer: null });
    },

    highlightMedia(mediaId) {
        set({ highlightedMediaId: mediaId, highlightUntil: null });
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
            console.error("Failed to refresh media item", err);
        }
    },

    setScrollTargetMediaId(mediaId) {
        set({ scrollTargetMediaId: mediaId });
    },

    clearHighlight() {
        set({ highlightedMediaId: null, highlightUntil: null });
    },
}));
