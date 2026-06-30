import { queryOptions, useQuery } from "@tanstack/react-query";
import { get_media_tags } from "@/api/tagsApi";
import { queryKeys } from "@/queries/keys";

export const mediaTagsQuery = (mediaId: number) =>
    queryOptions({
        queryKey: queryKeys.tags.forMedia(mediaId),
        queryFn: () => get_media_tags(mediaId),
    });

export function useMediaTags(mediaId: number) {
    const query = useQuery(mediaTagsQuery(mediaId));
    return {
        tags: query.data ?? [],
        isLoading: query.isPending,
    };
}
