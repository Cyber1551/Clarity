import { useEffect } from "react";
import { queryOptions, useQuery, useQueryClient } from "@tanstack/react-query";
import { get_media_detail } from "@/api/libraryApi";
import { type MediaDetail, type MediaItem } from "@/types/mediaTypes";
import { formatError } from "@/utils/format";
import { queryKeys } from "@/queries/keys";

interface UseMediaDetailArgs {
    /** The currently focused media id, or `null` when the viewer is closed. */
    mediaId: number | null;
    /** Sibling items used to compute prev/next ids for neighbor prefetch. */
    items: MediaItem[];
}

interface UseMediaDetailResult {
    detail: MediaDetail | null;
    loading: boolean;
    error: string | null;
    /** Re-fetch the current detail (invalidates the cached entry). */
    reload: () => void;
}

/**
 * Options factory for a single media detail.
 */
export const mediaDetailQuery = (mediaId: number) =>
    queryOptions({
        queryKey: queryKeys.library.detail(mediaId),
        queryFn: () => get_media_detail(mediaId),
    });

/**
 * Loads `MediaDetail` for the focused media id and proactively prefetches the prev/next neighbors so arrow-key navigation feels instant.
 */
export function useMediaDetail({ mediaId, items }: UseMediaDetailArgs): UseMediaDetailResult {
    const queryClient = useQueryClient();

    const query = useQuery({
        // The factory needs a real id; gate the actual fetch behind `enabled`.
        ...mediaDetailQuery(mediaId ?? -1),
        enabled: mediaId != null,
    });

    // Warm the cache for the immediate neighbors so left/right arrow nav swaps detail instantly.
    // Prefetch failures are silently swallowed.
    useEffect(() => {
        if (mediaId == null) return;
        const idx = items.findIndex((i) => i.mediaId === mediaId);
        if (idx === -1) return;

        for (const offset of [-1, 1] as const) {
            const neighbor = items[idx + offset];
            if (!neighbor) continue;
            void queryClient.prefetchQuery(mediaDetailQuery(neighbor.mediaId));
        }
    }, [mediaId, items, queryClient]);

    const reload = () => {
        if (mediaId == null) return;
        void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
    };

    return {
        detail: query.data ?? null,
        loading: mediaId != null && query.isPending,
        error: query.error ? formatError(query.error, "Failed to load media detail") : null,
        reload,
    };
}
