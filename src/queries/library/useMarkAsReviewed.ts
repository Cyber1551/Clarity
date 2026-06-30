import { useMutation, useQueryClient } from "@tanstack/react-query";
import { mark_as_reviewed } from "@/api/libraryApi";
import { useViewerStore } from "@/stores/viewerStore";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

/**
 * Lazy review: flips the DB flag now (the move into the Library tree happens on the next sync) and drops the item from the viewer queue so the next one slides in.
 */
export function useMarkAsReviewed() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: (mediaId: number) => mark_as_reviewed(mediaId),
        onSuccess: (_data, mediaId) => {
            useViewerStore.getState().removeCurrentViewerItem();
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.items() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.imports.all() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
        },
        onError: (e) => notify.error("Couldn't mark as reviewed", e),
    });
}
