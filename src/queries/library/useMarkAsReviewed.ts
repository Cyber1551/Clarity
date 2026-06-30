import { useMutation, useQueryClient } from "@tanstack/react-query";
import { mark_as_reviewed } from "@/api/libraryApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

export function useMarkAsReviewed() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: (mediaId: number) => mark_as_reviewed(mediaId),
        onSuccess: (_data, mediaId) => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.items() });
        },
        onError: (e) => notify.error("Couldn't mark as reviewed", e),
    });
}
