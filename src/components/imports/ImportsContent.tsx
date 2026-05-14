import { Box, Button, HStack, Text } from "@chakra-ui/react";
import { type MediaItem } from "@/types/mediaTypes";
import { type ImportResult } from "@/types/importTypes";
import { MediaGrid } from "@/components/library";
import { useMediaStore } from "@/stores/mediaStore";

interface ImportsContentProps {
    selectedFolder: string | null;
    items: MediaItem[];
    isLoadingItems: boolean;
    lastImportResult: ImportResult | null;
    scrollTargetMediaId: number | null;
    onScrolledToMediaId: () => void;
    onShowDuplicates: () => void;
}

export function ImportsContent({
    selectedFolder,
    items,
    isLoadingItems,
    lastImportResult,
    scrollTargetMediaId,
    onScrolledToMediaId,
    onShowDuplicates,
}: ImportsContentProps) {
    const clearHighlight = useMediaStore(s => s.clearHighlight);

    if (!selectedFolder) {
        return (
            <Box flex={1} minH={0} display="flex" flexDirection="column">
                <Box display="flex" alignItems="center" justifyContent="center" h="full" color="gray.500">
                    Select an import session from the sidebar
                </Box>
            </Box>
        );
    }

    const summary = lastImportResult?.folderName === selectedFolder ? lastImportResult : null;

    return (
        <Box flex={1} minH={0} display="flex" flexDirection="column">
            <Box p={4} borderBottomWidth="1px" borderColor="gray.100">
                <HStack gap={3} align="center" flexWrap="wrap">
                    <Text fontSize="lg" fontWeight="bold">{selectedFolder}</Text>
                    {summary && (
                        <Text fontSize="sm" color="gray.600">
                            Imported {summary.importedCount}
                            {" - "}
                            Skipped {summary.skippedCount} duplicates
                        </Text>
                    )}
                    {summary && summary.skippedCount > 0 && (
                        <Button size="xs" variant="outline" onClick={onShowDuplicates}>
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
                    mode="import"
                    scrollToMediaId={scrollTargetMediaId}
                    onScrolledToMediaId={onScrolledToMediaId}
                />
            </Box>
        </Box>
    );
}
