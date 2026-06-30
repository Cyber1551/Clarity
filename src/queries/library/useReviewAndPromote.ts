import { useMutation, useQueryClient } from "@tanstack/react-query";
import { review_and_promote } from "@/api/libraryApi";
import { useViewerStore } from "@/stores/viewerStore";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

/**
 * Reviews + promotes a media item out of Imports into the Library. 
 * Drops the promoted item from the viewer queue so the next item slides in.
 */
export function useReviewAndPromote() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: (mediaId: number) => review_and_promote(mediaId),
        onSuccess: (_data, mediaId) => {
            useViewerStore.getState().removeCurrentViewerItem();
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.items() });
            // The item moves between folders, so import lists can change too.
            void queryClient.invalidateQueries({ queryKey: queryKeys.imports.all() });
        },
        onError: (e) => notify.error("Couldn't promote media", e),
    });
}
