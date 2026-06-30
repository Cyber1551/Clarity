import { queryOptions, useQuery } from "@tanstack/react-query";
import { list_tags } from "@/api/tagsApi";
import { queryKeys } from "@/queries/keys";

/** Full tag set, used for autocomplete suggestions. */
export const tagsQuery = () =>
    queryOptions({
        queryKey: queryKeys.tags.list(),
        queryFn: () => list_tags(),
    });

export function useTags() {
    const query = useQuery(tagsQuery());
    return {
        tags: query.data ?? [],
        isLoading: query.isPending,
    };
}
