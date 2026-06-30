import { create } from "zustand";

interface MediaStoreState {
    highlightedMediaId: number | null;
    scrollTargetMediaId: number | null;
    highlightMedia: (mediaId: number) => void;
    setScrollTargetMediaId: (mediaId: number | null) => void;
    clearHighlight: () => void;
}

export const useMediaStore = create<MediaStoreState>((set) => ({
    highlightedMediaId: null,
    scrollTargetMediaId: null,

    highlightMedia(mediaId) {
        set({ highlightedMediaId: mediaId });
    },

    setScrollTargetMediaId(mediaId) {
        set({ scrollTargetMediaId: mediaId });
    },

    clearHighlight() {
        set({ highlightedMediaId: null });
    },
}));
