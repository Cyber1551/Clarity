import { useEffect, useMemo, useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { search_media } from "@/api/searchApi";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { serializeFilters, toSearchQuery } from "@/components/search/searchQuery";
import { type SearchFilters } from "@/types/searchTypes";
import { queryKeys } from "@/queries/keys";

const DEBOUNCE_MS = 200;
const RESULT_LIMIT = 200;

function useDebouncedValue<T>(value: T, delayMs: number): T {
    const [debounced, setDebounced] = useState(value);
    useEffect(() => {
        const handle = setTimeout(() => setDebounced(value), delayMs);
        return () => clearTimeout(handle);
    }, [value, delayMs]);
    return debounced;
}

interface UseSearchMediaOptions {
    enabled: boolean;
}

export function useSearchMedia(filters: SearchFilters, { enabled }: UseSearchMediaOptions) {
    const isLibraryReady = useInterfaceStore((s) => s.isLibraryReady);
    const debouncedFilters = useDebouncedValue(filters, DEBOUNCE_MS);

    const serialized = useMemo(() => serializeFilters(debouncedFilters), [debouncedFilters]);
    const query = useMemo(() => toSearchQuery(debouncedFilters, RESULT_LIMIT), [debouncedFilters]);

    const result = useQuery({
        queryKey: queryKeys.search.results(serialized),
        queryFn: () => search_media(query),
        placeholderData: keepPreviousData,
        enabled: enabled && isLibraryReady,
    });

    return {
        results: result.data ?? [],
        isLoading: result.isLoading,
        isFetching: result.isFetching,
    };
}
