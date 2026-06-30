import { useMutation, useQueryClient } from "@tanstack/react-query";
import { rename_media } from "@/api/libraryApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

interface RenameMediaVariables {
    mediaId: number;
    newName: string;
}

/** Logical rename via display_name; the on-disk hardlinks are renamed on the next sync. */
export function useRenameMedia() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ mediaId, newName }: RenameMediaVariables) => rename_media(mediaId, newName),
        onSuccess: (_data, { mediaId }) => {
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.detail(mediaId) });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.items() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
        },
        onError: (e) => notify.error("Couldn't rename", e),
    });
}
