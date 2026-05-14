import { useCallback } from "react";
import {
    rename_media_file,
    review_and_promote,
    toggle_loved,
    update_favorite_rating,
    update_quality_rating,
} from "@/api/libraryApi";
import { useMediaStore } from "@/stores/mediaStore";
import { notify } from "@/utils/notify";

interface UseViewerMutationsArgs {
    /** Currently focused media id (null when viewer is closed). */
    mediaId: number | null;
    /** Called after a successful mutation so the cache can be invalidated and refetched. */
    onMutated: () => void;
}

interface UseViewerMutationsResult {
    toggleLoved: () => Promise<void>;
    setQuality: (rating: number) => Promise<void>;
    setFavorite: (rating: number) => Promise<void>;
    rename: (fileId: number, newName: string) => Promise<void>;
    /** Marks the current item reviewed and removes it from the viewer queue. */
    markReviewed: () => Promise<void>;
}

/**
 * Wraps the viewer's mutating backend calls. 
 * Every call refetches the current detail via `onMutated` so the UI stays consistent without callers having to remember to invalidate.
 *
 * Each catch block surfaces the failure via `notify.error`; the underlying `_invoke` chokepoint already logs the rejection once, so the only thing left to do here is tell the user.
 */
export function useViewerMutations({
    mediaId,
    onMutated,
}: UseViewerMutationsArgs): UseViewerMutationsResult {
    const removeCurrentViewerItem = useMediaStore((s) => s.removeCurrentViewerItem);

    const toggleLoved = useCallback(async () => {
        if (mediaId == null) return;
        try {
            await toggle_loved(mediaId);
            onMutated();
        } catch (e) {
            notify.error("Couldn't update loved", e);
        }
    }, [mediaId, onMutated]);

    const setQuality = useCallback(async (rating: number) => {
        if (mediaId == null) return;
        try {
            await update_quality_rating(mediaId, rating);
            onMutated();
        } catch (e) {
            notify.error("Couldn't update quality rating", e);
        }
    }, [mediaId, onMutated]);

    const setFavorite = useCallback(async (rating: number) => {
        if (mediaId == null) return;
        try {
            await update_favorite_rating(mediaId, rating);
            onMutated();
        } catch (e) {
            notify.error("Couldn't update favorite rating", e);
        }
    }, [mediaId, onMutated]);

    const rename = useCallback(async (fileId: number, newName: string) => {
        try {
            await rename_media_file(fileId, newName);
            onMutated();
        } catch (e) {
            notify.error("Couldn't rename file", e);
        }
    }, [onMutated]);

    const markReviewed = useCallback(async () => {
        if (mediaId == null) return;
        try {
            await review_and_promote(mediaId);
            removeCurrentViewerItem();
        } catch (e) {
            notify.error("Couldn't promote media", e);
        }
    }, [mediaId, removeCurrentViewerItem]);

    return { toggleLoved, setQuality, setFavorite, rename, markReviewed };
}
