import { useMutation, useQueryClient } from "@tanstack/react-query";
import { remove_media_tag } from "@/api/tagsApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

interface RemoveMediaTagVariables {
    mediaId: number;
    tagId: number;
}

export function useRemoveMediaTag() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ mediaId, tagId }: RemoveMediaTagVariables) =>
            remove_media_tag(mediaId, tagId),
        onSuccess: (_data, { mediaId }) => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.tags.forMedia(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
        },
        onError: (e) => notify.error("Couldn't remove tag", e),
    });
}
