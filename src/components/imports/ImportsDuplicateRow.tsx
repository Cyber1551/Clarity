import { type ImportSkippedItem } from "@/types/importTypes.ts";
import { Box, Button, HStack, VStack, Text, Image } from "@chakra-ui/react";
import { useThumbnailUrl } from "@/queries/thumbnails/useThumbnailUrl";

export interface ImportsDuplicateRowProps {
    item: ImportSkippedItem;
    onJump: (item: ImportSkippedItem) => void;
}

export const ImportsDuplicateRow = ({ item, onJump }: ImportsDuplicateRowProps) => {
    const thumbUrl = useThumbnailUrl(item.contentHash);

    const location = item.existingDirPath ?? item.originalImportFolder ?? "Library";

    return (
        <HStack justify="space-between" gap={4} flexWrap="wrap">
            <HStack gap={3} minW="240px" flex="1">
                <Box w="56px" h="42px" borderRadius="md" overflow="hidden" bg="gray.100" flexShrink={0}>
                    <Image src={thumbUrl ?? undefined} alt={item.fileName} w="full" h="full" objectFit="cover" />
                </Box>
                <VStack align="start" gap={0} minW={0}>
                    <Text fontSize="sm" color="gray.800" truncate>{item.fileName}</Text>
                    <Text fontSize="xs" color="gray.500" truncate>{location}</Text>
                </VStack>
            </HStack>
            <Button size="xs" variant="outline" onClick={() => onJump(item)}>
                Jump
            </Button>
        </HStack>
    );
};
