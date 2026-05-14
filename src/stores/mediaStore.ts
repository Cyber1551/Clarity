import { type MediaItem } from "@/types/mediaTypes";
import { create } from "zustand";
import { get_media_items, get_media_item_by_rel_path } from "@/api/libraryApi";
import { formatError } from "@/utils/format";

export type ViewerMode = "import" | "library";

export type ViewerState = null | {
    mediaId: number;
    mode: ViewerMode;
    items: MediaItem[];
};

interface MediaStoreState {
    items: MediaItem[];
    isLoading: boolean;
    error: string | null;
    viewer: ViewerState;
    highlightedMediaId: number | null;
    scrollTargetMediaId: number | null;
    setIsLoading: (isLoading: boolean) => void;
    loadAllMedia: () => Promise<void>;
    openViewer: (mediaId: number, mode: ViewerMode, items: MediaItem[]) => void;
    closeViewer: () => void;
    navigateViewer: (direction: "prev" | "next") => void;
    removeCurrentViewerItem: () => void;
    highlightMedia: (mediaId: number) => void;
    refreshItemByRelPath: (relPath: string) => Promise<void>;
    setScrollTargetMediaId: (mediaId: number | null) => void;
    clearHighlight: () => void;
}

export const useMediaStore = create<MediaStoreState>((set, get) => ({
    items: [],
    isLoading: true,
    error: null,
    viewer: null,
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
            console.error("Failed to load media", err);
            set({
                error: formatError(err, "Failed to load media."),
                isLoading: false,
            });
        }
    },

    openViewer(mediaId, mode, viewerItems) {
        set({ viewer: { mediaId, mode, items: viewerItems } });
    },

    closeViewer() {
        set({ viewer: null });
    },

    navigateViewer(direction) {
        const viewer = get().viewer;
        if (!viewer) return;
        const idx = viewer.items.findIndex(i => i.mediaId === viewer.mediaId);
        if (idx === -1) return;
        const nextIdx = direction === "prev" ? idx - 1 : idx + 1;
        const nextItem = viewer.items[nextIdx];
        if (!nextItem) return;
        set({ viewer: { ...viewer, mediaId: nextItem.mediaId } });
    },

    removeCurrentViewerItem() {
        const viewer = get().viewer;
        if (!viewer) return;
        
        const idx = viewer.items.findIndex(i => i.mediaId === viewer.mediaId);
        if (idx === -1) return;
        
        const remaining = viewer.items.filter(i => i.mediaId !== viewer.mediaId);
        if (remaining.length === 0) {
            set({ viewer: null });
            return;
        }
        
        const nextIdx = Math.min(idx, remaining.length - 1);
        const nextItem = remaining[nextIdx];
        if (!nextItem) return;
        set({ viewer: { ...viewer, items: remaining, mediaId: nextItem.mediaId } });
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
            console.error("Failed to refresh media item", err);
        }
    },

    setScrollTargetMediaId(mediaId) {
        set({ scrollTargetMediaId: mediaId });
    },

    clearHighlight() {
        set({ highlightedMediaId: null });
    },
}));
