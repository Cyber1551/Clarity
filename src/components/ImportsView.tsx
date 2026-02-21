import { useEffect, useRef, useState } from "react";
import { Box, Button, Dialog, HStack, Portal, Text, VStack, Spinner } from "@chakra-ui/react";
import { LuFolder } from "react-icons/lu";
import { get_import_folders, get_items_in_import_folder } from "@/api/importApi";
import { MediaItem } from "@/types/mediaTypes";
import MediaGrid from "./MediaGrid";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { listen } from "@tauri-apps/api/event";
import { useMediaStore } from "@/stores/mediaStore";
import { get_media_item_by_rel_path } from "@/api/libraryApi";
import { ImportSkippedItem } from "@/types/importTypes";
import ImportsDuplicateRow from "@/components/ImportsDuplicateRow.tsx";

type JobCompletedPayload = {
    jobType: string;
    mediaId: number | null;
    fileId: number | null;
    relPath: string | null;
    status: string;
};

const ImportsView = () => {
    const [folders, setFolders] = useState<string[]>([]);
    const selectedFolder = useInterfaceStore(s => s.selectedImportFolder);
    const setSelectedFolder = useInterfaceStore(s => s.setSelectedImportFolder);
    const lastImportResult = useInterfaceStore(s => s.lastImportResult);
    const setActiveTab = useInterfaceStore(s => s.setActiveTab);
    const activeTab = useInterfaceStore(s => s.activeTab);
    const highlightMedia = useMediaStore(s => s.highlightMedia);
    const setScrollTargetMediaId = useMediaStore(s => s.setScrollTargetMediaId);
    const clearHighlight = useMediaStore(s => s.clearHighlight);
    const [items, setItems] = useState<MediaItem[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [isLoadingItems, setIsLoadingItems] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [isDuplicatesOpen, setDuplicatesOpen] = useState(false);
    const [importScrollTarget, setImportScrollTarget] = useState<number | null>(null);
    const suppressClearOnFolderChange = useRef(false);

    useEffect(() => {
        void loadFolders();

        const unsubscribeLibrary = listen("library-changed", () => {
            void loadFolders();
            if (selectedFolder) {
                void loadItems(selectedFolder);
            }
        });

        const unsubscribeJob = listen<JobCompletedPayload>("job-completed", (event) => {
            if (activeTab !== "imports") return;
            if (!selectedFolder) return;
            const relPath = event.payload?.relPath;
            if (!relPath) return;
            const prefix = `Imports/${selectedFolder}/`;
            if (!relPath.startsWith(prefix)) return;
            void refreshItemByRelPath(relPath);
        });

        return () => {
            unsubscribeLibrary.then(fn => fn());
            unsubscribeJob.then(fn => fn());
        };
    }, [activeTab, selectedFolder]);

    useEffect(() => {
        if (selectedFolder) {
            void loadItems(selectedFolder);
        } else {
            setItems([]);
        }
    }, [selectedFolder]);

    useEffect(() => {
        if (suppressClearOnFolderChange.current) {
            suppressClearOnFolderChange.current = false;
            return;
        }
        clearHighlight();
    }, [clearHighlight, selectedFolder]);

    useEffect(() => {
        setDuplicatesOpen(false);
    }, [selectedFolder, lastImportResult?.folderName]);

    const loadFolders = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const f = await get_import_folders();
            setFolders(f);
        } catch (e: unknown) {
            setError(e instanceof Error ? e.message : String(e));
        } finally {
            setIsLoading(false);
        }
    };

    const loadItems = async (folderName: string) => {
        setIsLoadingItems(true);
        try {
            const i = await get_items_in_import_folder(folderName);
            setItems(i);
        } catch (e: unknown) {
            console.error(e);
        } finally {
            setIsLoadingItems(false);
        }
    };

    const refreshItemByRelPath = async (relPath: string) => {
        try {
            const item = await get_media_item_by_rel_path(relPath);
            if (!item) return;
            setItems(existing =>
                existing.map(current =>
                    current.relPath === item.relPath ? item : current
                )
            );
        } catch (e) {
            console.error("Failed to refresh import item", e);
        }
    };

    const handleJump = (item: ImportSkippedItem) => {
        if (item.existingDirPath?.startsWith("Imports/")) {
            const folder = item.existingDirPath.replace(/^Imports\//, "");
            suppressClearOnFolderChange.current = true;
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
                {/* Sidebar */}
                <VStack 
                    w="250px" 
                    borderRightWidth="1px" 
                    borderColor="gray.100" 
                    align="stretch" 
                    p={4} 
                    gap={2}
                    overflowY="auto"
                >
                    <Text fontWeight="bold" fontSize="sm" color="gray.500" mb={2}>IMPORT SESSIONS</Text>
                    {isLoading && folders.length === 0 && <Spinner size="sm" />}
                        {folders.map(folder => (
                            <HStack 
                                key={folder}
                                p={2}
                                borderRadius="md"
                                cursor="pointer"
                                bg={selectedFolder === folder ? "blue.50" : "transparent"}
                                color={selectedFolder === folder ? "blue.600" : "inherit"}
                                _hover={{ bg: selectedFolder === folder ? "blue.50" : "gray.700" }}
                                onClick={() => {
                                    clearHighlight();
                                    setSelectedFolder(folder);
                                }}
                            >
                            <LuFolder />
                            <Text fontSize="sm" fontWeight={selectedFolder === folder ? "bold" : "normal"}>
                                {folder}
                            </Text>
                        </HStack>
                    ))}
                    {folders.length === 0 && !isLoading && (
                        <Text fontSize="xs" color="gray.400">No imports found</Text>
                    )}
                    {error && <Text fontSize="xs" color="red.500">{error}</Text>}
                </VStack>

                {/* Main Content */}
                <Box flex={1} minH={0} display="flex" flexDirection="column">
                    {selectedFolder ? (
                        <>
                            <Box p={4} borderBottomWidth="1px" borderColor="gray.100">
                                <HStack gap={3} align="center" flexWrap="wrap">
                                    <Text fontSize="lg" fontWeight="bold">{selectedFolder}</Text>
                                    {lastImportResult?.folderName === selectedFolder && (
                                        <Text fontSize="sm" color="gray.600">
                                            Imported {lastImportResult.importedCount}
                                            {" - "}
                                            Skipped {lastImportResult.skippedCount} duplicates
                                        </Text>
                                    )}
                                    {lastImportResult?.folderName === selectedFolder && lastImportResult.skippedCount > 0 && (
                                        <Button
                                            size="xs"
                                            variant="outline"
                                            onClick={() => setDuplicatesOpen(true)}
                                        >
                                            View duplicates
                                        </Button>
                                    )}
                            </HStack>
                        </Box>
                        <Box flex={1} minH={0} paddingTop={5} onClick={clearHighlight}>
                            <MediaGrid
                                items={items}
                                isLoading={isLoadingItems}
                                error={null}
                                scrollToMediaId={importScrollTarget}
                                onScrolledToMediaId={() => setImportScrollTarget(null)}
                            />
                        </Box>
                    </>
                ) : (
                        <Box display="flex" alignItems="center" justifyContent="center" h="full" color="gray.500">
                            Select an import session from the sidebar
                        </Box>
                    )}
                </Box>
            </HStack>

            <Dialog.Root
                placement="center"
                motionPreset="slide-in-bottom"
                open={isDuplicatesOpen}
                onOpenChange={({ open }) => setDuplicatesOpen(open)}
            >
                <Portal>
                    <Dialog.Backdrop />
                    <Dialog.Positioner>
                        <Dialog.Content maxW="lg" maxH="90vh">
                            <Dialog.Header>
                                <Dialog.Title>Duplicates skipped</Dialog.Title>
                            </Dialog.Header>
                            <Dialog.Body style={{ overflowY: 'auto' }}>
                                {lastImportResult?.skippedItems.length ? (
                                    <VStack align="stretch" gap={4}>
                                        {lastImportResult.skippedItems.map(item => (
                                            <ImportsDuplicateRow
                                                key={`${item.mediaId}-${item.contentHash}`}
                                                item={item}
                                                onJump={handleJump}
                                            />
                                        ))}
                                    </VStack>
                                ) : (
                                    <Text fontSize="sm" color="gray.500">No duplicates to show.</Text>
                                )}
                            </Dialog.Body>
                            <Dialog.Footer>
                                <Dialog.ActionTrigger asChild>
                                    <Button variant="outline">Close</Button>
                                </Dialog.ActionTrigger>
                            </Dialog.Footer>
                        </Dialog.Content>
                    </Dialog.Positioner>
                </Portal>
            </Dialog.Root>
        </>
    );
};

export default ImportsView;
