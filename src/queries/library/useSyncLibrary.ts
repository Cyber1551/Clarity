import { useMutation, useQueryClient } from "@tanstack/react-query";
import { sync_library } from "@/api/syncApi";
import { type SyncReport } from "@/types/mediaTypes";
import { queryKeys } from "@/queries/keys";
import { notify } from "@/utils/notify";

function summarize(report: SyncReport): string {
    return `${report.reconciled} item${report.reconciled === 1 ? "" : "s"} updated`;
}

/** Materializes all dirty reviewed items into the Library hardlink tree. */
export function useSyncLibrary() {
    const queryClient = useQueryClient();
    return useMutation({
        mutationFn: () => sync_library(),
        onSuccess: (report) => {
            notify.success("Synced to disk", summarize(report));
            void queryClient.invalidateQueries({ queryKey: queryKeys.sync.status() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.library.all() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.imports.all() });
        },
        onError: (e) => notify.error("Sync failed", e),
    });
}
