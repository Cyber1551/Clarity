import { queryOptions, useQuery } from "@tanstack/react-query";
import { get_media_items } from "@/api/libraryApi";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { formatError } from "@/utils/format";
import { queryKeys } from "@/queries/keys";

/**
 * Options factory for the library catalog.
 */
export const mediaItemsQuery = () =>
    queryOptions({
        queryKey: queryKeys.library.items(),
        queryFn: () => get_media_items(),
    });

/**
 * The full library catalog.
 */
export function useMediaItems() {
    const isLibraryReady = useInterfaceStore((s) => s.isLibraryReady);

    const query = useQuery({
        ...mediaItemsQuery(),
        enabled: isLibraryReady,
    });

    return {
        items: query.data ?? [],
        // Treat "waiting for library-initialized" as loading so the grid shows its spinner instead of an empty state during cold start.
        isLoading: !isLibraryReady || query.isPending,
        error: query.error ? formatError(query.error, "Failed to load media.") : null,
    };
}
