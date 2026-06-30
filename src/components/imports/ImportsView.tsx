import { useState } from "react";
import { HStack } from "@chakra-ui/react";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { useMediaStore } from "@/stores/mediaStore";
import { type ImportSkippedItem } from "@/types/importTypes";
import { useImportFolders } from "@/queries/imports/useImportFolders";
import { useImportFolderItems } from "@/queries/imports/useImportFolderItems";
import { ImportsSidebar } from "./ImportsSidebar";
import { ImportsContent } from "./ImportsContent";
import { ImportsDuplicatesDialog } from "./ImportsDuplicatesDialog";

export const ImportsView = () => {
    const selectedFolder = useInterfaceStore(s => s.selectedImportFolder);
    const setSelectedFolder = useInterfaceStore(s => s.setSelectedImportFolder);
    const lastImportResult = useInterfaceStore(s => s.lastImportResult);
    const setActiveTab = useInterfaceStore(s => s.setActiveTab);
    const highlightMedia = useMediaStore(s => s.highlightMedia);
    const setScrollTargetMediaId = useMediaStore(s => s.setScrollTargetMediaId);
    const clearHighlight = useMediaStore(s => s.clearHighlight);

    const { folders, isLoading, error } = useImportFolders();
    const { items, isLoading: isLoadingItems } = useImportFolderItems(selectedFolder);

    const [isDuplicatesOpen, setDuplicatesOpen] = useState(false);
    const [importScrollTarget, setImportScrollTarget] = useState<number | null>(null);

    const handleSelectFolder = (folder: string) => {
        clearHighlight();
        setDuplicatesOpen(false);
        setSelectedFolder(folder);
    };

    const handleJump = (item: ImportSkippedItem) => {
        if (item.existingDirPath?.startsWith("Imports/")) {
            const folder = item.existingDirPath.replace(/^Imports\//, "");
            // We're navigating to the existing item's folder; preserve the
            // highlight so the row still pulses after the jump.
            setSelectedFolder(folder);
            setActiveTab("imports");
            setImportScrollTarget(item.mediaId);
        } else {
            setActiveTab("library");
            setScrollTargetMediaId(item.mediaId);
        }
        highlightMedia(item.mediaId);
        setDuplicatesOpen(false);
    };

    return (
        <>
            <HStack h="full" w="full" gap={0} align="stretch">
                <ImportsSidebar
                    folders={folders}
                    selectedFolder={selectedFolder}
                    isLoading={isLoading}
                    error={error}
                    onSelect={handleSelectFolder}
                />
                <ImportsContent
                    selectedFolder={selectedFolder}
                    items={items}
                    isLoadingItems={isLoadingItems}
                    lastImportResult={lastImportResult}
                    scrollTargetMediaId={importScrollTarget}
                    onScrolledToMediaId={() => setImportScrollTarget(null)}
                    onShowDuplicates={() => setDuplicatesOpen(true)}
                />
            </HStack>

            <ImportsDuplicatesDialog
                open={isDuplicatesOpen}
                onOpenChange={setDuplicatesOpen}
                skippedItems={lastImportResult?.skippedItems ?? []}
                onJump={handleJump}
            />
        </>
    );
};
