import { useEffect, useState } from "react";
import { queryOptions, useQuery } from "@tanstack/react-query";
import { get_thumbnail } from "@/api/libraryApi";
import { logger } from "@/utils/logger";
import { queryKeys } from "@/queries/keys";

interface UseThumbnailUrlOptions {
    enabled?: boolean;
}

/**
 * Options factory for a thumbnail's raw `BlobWithMime`. Thumbnails are immutable (the content hash is part of the key), hence `staleTime: Infinity`.
 */
export const thumbnailQuery = (hash: string) =>
    queryOptions({
        queryKey: queryKeys.thumbnail(hash),
        queryFn: () => get_thumbnail(hash),
        staleTime: Infinity,
    });

/**
 * Returns a stable object URL for a thumbnail keyed by content hash.
 */
export function useThumbnailUrl(
    hash: string | null | undefined,
    options: UseThumbnailUrlOptions = {},
): string | null {
    const { enabled = true } = options;
    const active = enabled && !!hash;

    const query = useQuery({
        ...thumbnailQuery(hash ?? ""),
        enabled: active,
    });

    const [url, setUrl] = useState<string | null>(null);
    const data = query.data;

    useEffect(() => {
        let objectUrl: string | null = null;

        function sync() {
            if (!data) {
                setUrl(null);
                return;
            }
            objectUrl = URL.createObjectURL(new Blob([data.blob], { type: data.mimetype }));
            setUrl(objectUrl);
        }

        sync();

        return () => {
            if (objectUrl) URL.revokeObjectURL(objectUrl);
        };
    }, [data]);

    useEffect(() => {
        if (query.error) {
            logger.debug("thumbnails", "failed to load thumbnail", {
                hash,
                error: query.error,
            });
        }
    }, [hash, query.error]);

    return url;
}
