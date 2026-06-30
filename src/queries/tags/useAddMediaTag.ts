import { useMutation, useQueryClient } from "@tanstack/react-query";
import { add_media_tag } from "@/api/tagsApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

interface AddMediaTagVariables {
    mediaId: number;
    name: string;
}

export function useAddMediaTag() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ mediaId, name }: AddMediaTagVariables) => add_media_tag(mediaId, name),
        onSuccess: (_tag, { mediaId }) => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.tags.forMedia(mediaId) });
            // add may have created a brand-new tag, so refresh the global list too
            void queryClient.invalidateQueries({ queryKey: queryKeys.tags.list() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
        },
        onError: (e) => notify.error("Couldn't add tag", e),
    });
}
