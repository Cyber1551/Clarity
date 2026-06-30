import { useMutation, useQueryClient } from "@tanstack/react-query";
import { rename_media_file } from "@/api/libraryApi";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

interface RenameMediaFileVariables {
    fileId: number;
    newFileName: string;
    /** Used to invalidate the right detail entry on success. */
    mediaId: number | null;
}

export function useRenameMediaFile() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: ({ fileId, newFileName }: RenameMediaFileVariables) =>
            rename_media_file(fileId, newFileName),
        onSuccess: (_data, { mediaId }) => {
            if (mediaId != null) {
                void queryClient.invalidateQueries({
                    queryKey: queryKeys.library.detail(mediaId),
                });
            }
            // The catalog row's `fileName` changes too, so refresh the list.
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.items() });
        },
        onError: (e) => notify.error("Couldn't rename file", e),
    });
}
