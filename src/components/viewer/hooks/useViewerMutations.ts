import { useCallback } from "react";
import { useRenameMediaFile } from "@/queries/library/useRenameMediaFile";
import { useReviewAndPromote } from "@/queries/library/useReviewAndPromote";
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
    rename: (fileId: number, newName: string) => Promise<void>;
    /** Marks the current item reviewed and removes it from the viewer queue. */
    markReviewed: () => Promise<void>;
}

/**
 * Bundles the viewer's mutating backend calls behind a stable, mediaId-aware facade.
 * Each underlying mutation lives in `@/queries/library/` and handles its own invalidation + error toast
 */
export function useViewerMutations({
    mediaId,
}: UseViewerMutationsArgs): UseViewerMutationsResult {
    const toggleLovedM = useToggleLoved();
    const updateQualityM = useUpdateQualityRating();
    const updateFavoriteM = useUpdateFavoriteRating();
    const renameM = useRenameMediaFile();
    const promoteM = useReviewAndPromote();

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

    const rename = useCallback(async (fileId: number, newFileName: string) => {
        await renameM
            .mutateAsync({ fileId, newFileName, mediaId })
            .catch(() => undefined);
    }, [mediaId, renameM]);

    const markReviewed = useCallback(async () => {
        if (mediaId == null) return;
        await promoteM.mutateAsync(mediaId).catch(() => undefined);
    }, [mediaId, promoteM]);

    return { toggleLoved, setQuality, setFavorite, rename, markReviewed };
}
