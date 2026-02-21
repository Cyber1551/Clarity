import { useEffect, useState } from "react";
import { ImportSkippedItem } from "@/types/importTypes.ts";
import { get_thumbnail } from "@/api/libraryApi.ts";
import { Box, Button, HStack, VStack, Text, Image } from "@chakra-ui/react";

export interface ImportsDuplicateRowProps {
    item: ImportSkippedItem;
    onJump: (item: ImportSkippedItem) => void;
}

const ImportsDuplicateRow = ({ item, onJump, }: ImportsDuplicateRowProps) => {
    const [thumbUrl, setThumbUrl] = useState<string | null>(null);

    useEffect(() => {
        let active = true;
        let objectUrl: string | null = null;

        async function loadThumb() {
            try {
                const { blob, mimetype } = await get_thumbnail(item.contentHash);
                if (!active) return;
                const newUrl = URL.createObjectURL(new Blob([blob] as BlobPart[], { type: mimetype }));
                if (!active) {
                    URL.revokeObjectURL(newUrl);
                    return;
                }
                objectUrl = newUrl;
                setThumbUrl(objectUrl);
            } catch (e) {
                console.error("Failed to load thumbnail", e);
            }
        }

        void loadThumb();

        return () => {
            active = false;
            if (objectUrl) {
                URL.revokeObjectURL(objectUrl);
            }
        };
    }, [item.contentHash]);

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

export default ImportsDuplicateRow;
