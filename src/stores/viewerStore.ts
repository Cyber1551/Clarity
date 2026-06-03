import { create } from "zustand";
import { type MediaItem } from "@/types/mediaTypes";

export type ViewerMode = "import" | "library";

export type ViewerState = null | {
    mediaId: number;
    mode: ViewerMode;
    items: MediaItem[];
};

interface ViewerStoreState {
    viewer: ViewerState;
    openViewer: (mediaId: number, mode: ViewerMode, items: MediaItem[]) => void;
    closeViewer: () => void;
    navigateViewer: (direction: "prev" | "next") => void;
    /**
     * Drop the focused item from the viewer queue. Used by `markReviewed` to advance the import-mode queue after promoting an item to the library.
     * Closes the viewer if the queue empties.
     */
    removeCurrentViewerItem: () => void;
}

export const useViewerStore = create<ViewerStoreState>((set, get) => ({
    viewer: null,

    openViewer(mediaId, mode, items) {
        set({ viewer: { mediaId, mode, items } });
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
}));
