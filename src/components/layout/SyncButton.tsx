import { Button } from "@chakra-ui/react";
import { RefreshCw } from "lucide-react";
import { useSyncStatus } from "@/queries/library/useSyncStatus";
import { useSyncLibrary } from "@/queries/library/useSyncLibrary";

/** Header action that flushes dirty reviewed items into the on-disk Library projection. */
export const SyncButton = () => {
    const { dirtyCount } = useSyncStatus();
    const sync = useSyncLibrary();
    const hasDirty = dirtyCount > 0;

    return (
        <Button
            size="sm"
            variant={hasDirty ? "solid" : "outline"}
            colorPalette={hasDirty ? "blue" : "gray"}
            onClick={() => sync.mutate()}
            disabled={!hasDirty || sync.isPending}
            loading={sync.isPending}
            loadingText="Syncing..."
        >
            <RefreshCw size={16} />
            {hasDirty ? `Sync to disk (${dirtyCount})` : "Synced"}
        </Button>
    );
};
