import { useCallback } from "react";
import { useMarkAsReviewed } from "@/queries/library/useMarkAsReviewed";
import { useRenameMedia } from "@/queries/library/useRenameMedia";
import { useToggleLoved } from "@/queries/library/useToggleLoved";
import { useUpdateFavoriteRating } from "@/queries/library/useUpdateFavoriteRating";
import { useUpdateQualityRating } from "@/queries/library/useUpdateQualityRating";

interface UseViewerMutationsArgs {
    /** Currently focused media id (null when viewer is closed). */
    mediaId: number | null;
}

interface UseViewerMutationsResult {
    toggleLoved: () => Promise<void>;
    setQuality: (rating: number) => Promise<void>;
    setFavorite: (rating: number) => Promise<void>;
    rename: (newName: string) => Promise<void>;
    markReviewed: () => Promise<void>;
}

/**
 * Stable, mediaId-aware facade over the viewer's mutating calls.
 */
export function useViewerMutations({
    mediaId,
}: UseViewerMutationsArgs): UseViewerMutationsResult {
    const toggleLovedM = useToggleLoved();
    const updateQualityM = useUpdateQualityRating();
    const updateFavoriteM = useUpdateFavoriteRating();
    const renameM = useRenameMedia();
    const reviewM = useMarkAsReviewed();

    const toggleLoved = useCallback(async () => {
        if (mediaId == null) return;
        await toggleLovedM.mutateAsync(mediaId).catch(() => undefined);
    }, [mediaId, toggleLovedM]);

    const setQuality = useCallback(async (rating: number) => {
        if (mediaId == null) return;
        await updateQualityM.mutateAsync({ mediaId, rating }).catch(() => undefined);
    }, [mediaId, updateQualityM]);

    const setFavorite = useCallback(async (rating: number) => {
        if (mediaId == null) return;
        await updateFavoriteM.mutateAsync({ mediaId, rating }).catch(() => undefined);
    }, [mediaId, updateFavoriteM]);

    const rename = useCallback(async (newName: string) => {
        if (mediaId == null) return;
        await renameM.mutateAsync({ mediaId, newName }).catch(() => undefined);
    }, [mediaId, renameM]);

    const markReviewed = useCallback(async () => {
        if (mediaId == null) return;
        await reviewM.mutateAsync(mediaId).catch(() => undefined);
    }, [mediaId, reviewM]);

    return { toggleLoved, setQuality, setFavorite, rename, markReviewed };
}
