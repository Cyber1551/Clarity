import { queryOptions, useQuery } from "@tanstack/react-query";
import { get_sync_status } from "@/api/syncApi";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { queryKeys } from "@/queries/keys";

/** Projection dirty count: how many reviewed items await sync to disk. */
export const syncStatusQuery = () =>
    queryOptions({
        queryKey: queryKeys.sync.status(),
        queryFn: () => get_sync_status(),
    });

export function useSyncStatus() {
    const isLibraryReady = useInterfaceStore((s) => s.isLibraryReady);
    const query = useQuery({
        ...syncStatusQuery(),
        enabled: isLibraryReady,
    });
    return {
        dirtyCount: query.data?.dirtyCount ?? 0,
        isLoading: query.isPending,
    };
}
