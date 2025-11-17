import { MediaItem } from "@/types/mediaTypes";
import { create } from "zustand";


export type ViewerState = null | { mediaId: number; };

type MediaStoreState = {
    items: MediaItem[];
    currentFolder: string;
    isLoading: boolean;
    error: string | null;
    viewer: ViewerState;
    setCurrentFolder: (folder: string) => void;
    loadFolder: (folder: string) => Promise<void>;
    openViewer: (mediaId: number) => void;
    closeViewer: () => void;
};

export const useMediaStore = create<MediaStoreState>((set, get) => ({
    items: [],
    currentFolder: "",
    isLoading: false,
    error: null,
    viewer: null,
    setCurrentFolder(folder) {
        set({ currentFolder: folder });
    },

    async loadFolder(folder) {
        set({ isLoading: true, error: null, currentFolder: folder });
        try {
            // TODO: fetch media items from backend
            set({
                isLoading: false,
            });
        } catch (err) {
            console.error("Failed to load folder", err);
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
