import { useCallback, useEffect, useRef, useState } from "react";
import { get_media_detail } from "@/api/libraryApi";
import { type MediaDetail, type MediaItem } from "@/types/mediaTypes";
import { formatError } from "@/utils/format";
import { PRELOAD_CACHE_MAX } from "../constants";

interface UseMediaDetailArgs {
    /** The currently focused media id, or `null` when the viewer is closed. */
    mediaId: number | null;
    /** Sibling items used to compute prev/next ids for preloading. */
    items: MediaItem[];
}

interface UseMediaDetailResult {
    detail: MediaDetail | null;
    loading: boolean;
    error: string | null;
    /** Re-fetch the current detail from the backend (bypasses cache). */
    reload: () => void;
    /** Drop a media id from the preload cache. */
    invalidate: (mediaId: number) => void;
}

/**
 * Loads `MediaDetail` for the focused media id and proactively preloads
 * the immediate neighbors (prev + next) so navigation feels instant.
 *
 * Caches up to `PRELOAD_CACHE_MAX` entries with simple LRU semantics
 * (newest entries are kept; oldest dropped).
 */
export function useMediaDetail({ mediaId, items }: UseMediaDetailArgs): UseMediaDetailResult {
    const [detail, setDetail] = useState<MediaDetail | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const cacheRef = useRef<Map<number, MediaDetail>>(new Map());

    const cachePut = useCallback((id: number, value: MediaDetail) => {
        const cache = cacheRef.current;
        if (cache.has(id)) cache.delete(id);
        cache.set(id, value);
        while (cache.size > PRELOAD_CACHE_MAX) {
            const oldest = cache.keys().next().value;
            if (oldest === undefined) break;
            cache.delete(oldest);
        }
    }, []);

    const loadDetail = useCallback(async (id: number) => {
        const cached = cacheRef.current.get(id);
        if (cached) {
            setDetail(cached);
            setError(null);
            setLoading(false);
            return;
        }

        setLoading(true);
        setError(null);
        try {
            const next = await get_media_detail(id);
            cachePut(id, next);
            setDetail(next);
        } catch (e: unknown) {
            setError(formatError(e, "Failed to load media detail"));
        } finally {
            setLoading(false);
        }
    }, [cachePut]);

    const preloadAdjacent = useCallback(async (id: number) => {
        const idx = items.findIndex((i) => i.mediaId === id);
        if (idx === -1) return;
        const neighbors = [idx - 1, idx + 1]
            .filter((n) => n >= 0 && n < items.length)
            .map((n) => items[n].mediaId)
            .filter((nid) => !cacheRef.current.has(nid));

        for (const nid of neighbors) {
            try {
                const value = await get_media_detail(nid);
                cachePut(nid, value);
            } catch {
                // Preload failures are non-critical
            }
        }
    }, [items, cachePut]);

    const invalidate = useCallback((id: number) => {
        cacheRef.current.delete(id);
    }, []);

    const reload = useCallback(() => {
        if (mediaId == null) return;
        cacheRef.current.delete(mediaId);
        void loadDetail(mediaId);
    }, [mediaId, loadDetail]);

    useEffect(() => {
        if (mediaId == null) {
            setDetail(null);
            setError(null);
            cacheRef.current.clear();
            return;
        }
        void loadDetail(mediaId);
        void preloadAdjacent(mediaId);
    }, [mediaId, loadDetail, preloadAdjacent]);

    return { detail, loading, error, reload, invalidate };
}
