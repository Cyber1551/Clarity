import { useEffect, useMemo, useState } from "react";
import { Box, HStack, Text, VStack, Spinner } from "@chakra-ui/react";
import { LuFolder } from "react-icons/lu";
import { get_import_folders, get_items_in_import_folder } from "@/api/importApi";
import { MediaItem } from "@/types/mediaTypes";
import MediaGrid from "./MediaGrid";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { listen } from "@tauri-apps/api/event";
import { throttle } from "lodash-es";

const ImportsView = () => {
    const [folders, setFolders] = useState<string[]>([]);
    const selectedFolder = useInterfaceStore(s => s.selectedImportFolder);
    const setSelectedFolder = useInterfaceStore(s => s.setSelectedImportFolder);
    const [items, setItems] = useState<MediaItem[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [isLoadingItems, setIsLoadingItems] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const throttledLoadItems = useMemo(
        () => throttle((folder: string) => void loadItems(folder), 2000, { leading: true, trailing: true }),
        []
    );

    useEffect(() => {
        void loadFolders();

        const unsubscribeLibrary = listen("library-changed", () => {
            void loadFolders();
            if (selectedFolder) {
                void loadItems(selectedFolder);
            }
        });

        const unsubscribeJob = listen("job-completed", () => {
            if (selectedFolder) {
                void throttledLoadItems(selectedFolder);
            }
        });

        return () => {
            unsubscribeLibrary.then(fn => fn());
            unsubscribeJob.then(fn => fn());
            throttledLoadItems.cancel();
        };
    }, [selectedFolder, throttledLoadItems]);

    useEffect(() => {
        if (selectedFolder) {
            void loadItems(selectedFolder);
        } else {
            setItems([]);
        }
    }, [selectedFolder]);

    const loadFolders = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const f = await get_import_folders();
            setFolders(f);
        } catch (e: any) {
            setError(e.toString());
        } finally {
            setIsLoading(false);
        }
    };

    const loadItems = async (folderName: string) => {
        setIsLoadingItems(true);
        try {
            const i = await get_items_in_import_folder(folderName);
            setItems(i);
        } catch (e: any) {
            console.error(e);
        } finally {
            setIsLoadingItems(false);
        }
    };

    return (
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
                        onClick={() => setSelectedFolder(folder)}
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
                            <Text fontSize="lg" fontWeight="bold">{selectedFolder}</Text>
                        </Box>
                        <Box flex={1} minH={0} paddingTop={5}>
                            <MediaGrid items={items} isLoading={isLoadingItems} error={null} />
                        </Box>
                    </>
                ) : (
                    <Box display="flex" alignItems="center" justifyContent="center" h="full" color="gray.500">
                        Select an import session from the sidebar
                    </Box>
                )}
            </Box>
        </HStack>
    );
};

export default ImportsView;
