import { useMutation, useQueryClient } from "@tanstack/react-query";
import { toggle_loved } from "@/api/libraryApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

export function useToggleLoved() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: (mediaId: number) => toggle_loved(mediaId),
        onSuccess: (_data, mediaId) => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
        },
        onError: (e) => notify.error("Couldn't update loved", e),
    });
}
