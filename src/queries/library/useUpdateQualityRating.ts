import { useMutation, useQueryClient } from "@tanstack/react-query";
import { update_quality_rating } from "@/api/libraryApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

export function useUpdateQualityRating() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ mediaId, rating }: { mediaId: number; rating: number }) =>
            update_quality_rating(mediaId, rating),
        onSuccess: (_data, { mediaId }) => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
        },
        onError: (e) => notify.error("Couldn't update quality rating", e),
    });
}
