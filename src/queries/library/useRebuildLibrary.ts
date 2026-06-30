import { useMutation, useQueryClient } from "@tanstack/react-query";
import { rebuild_library } from "@/api/syncApi";
import { type SyncReport } from "@/types/mediaTypes";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

function summarize(report: SyncReport): string {
    return `${report.reconciled} item${report.reconciled === 1 ? "" : "s"} reprojected`;
}

/** Wipes the Library tree and reprojects every reviewed item from scratch. */
export function useRebuildLibrary() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: () => rebuild_library(),
        onSuccess: (report) => {
            notify.success("Library rebuilt", summarize(report));
            void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.all() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.imports.all() });
        },
        onError: (e) => notify.error("Rebuild failed", e),
    });
}
