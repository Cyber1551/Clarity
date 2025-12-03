import { MediaItem } from "@/types/mediaTypes";
import { create } from "zustand";
import { get_all_media } from "@/api/libraryApi";

export type ViewerState = null | { mediaId: number; };

type MediaStoreState = {
    items: MediaItem[];
    isLoading: boolean;
    error: string | null;
    viewer: ViewerState;
    loadAllMedia: () => Promise<void>;
    openViewer: (mediaId: number) => void;
    closeViewer: () => void;
};

export const useMediaStore = create<MediaStoreState>((set, get) => ({
    items: [],
    isLoading: false,
    error: null,
    viewer: null,

    async loadAllMedia() {
        const currentState = get();
        // Only show loading spinner on initial load, not on event-driven refreshes
        const isInitialLoad = currentState.items.length === 0;

        if (isInitialLoad) {
            set({ isLoading: true, error: null });
        }

        try {
            const items = await get_all_media();
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
        const exists = get().items.some((m) => m.mediaId === mediaId);
        if (!exists) {
            console.warn("Tried to open viewer for media not in current items", mediaId);
            return;
        }
        set({ viewer: { mediaId } });
    },

    closeViewer() {
        set({ viewer: null });
    },
}));
