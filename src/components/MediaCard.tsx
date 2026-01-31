import { MediaItem } from "@/types/mediaTypes";
import { Box, Image, Spinner, Text } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { useEffect, useState } from "react";
import { get_thumbnail } from "@/api/libraryApi";

interface MediaCardProps {
    item: MediaItem;
}

const MediaCard = ({ item }: MediaCardProps) => {
    const openViewer = useMediaStore(s => s.openViewer);
    const [thumbUrl, setThumbUrl] = useState<string | null>(null);

    useEffect(() => {
        let active = true;
        let objectUrl: string | null = null;

        async function loadThumb() {
            try {
                const blob = await get_thumbnail(item.contentHash);
                if (!active) return;
                
                const newUrl = URL.createObjectURL(new Blob([blob], { type: "image/webp" }));
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

        if (item.thumbnailStatus === "done") {
            void loadThumb();
        }

        return () => {
            active = false;
            if (objectUrl) {
                URL.revokeObjectURL(objectUrl);
            }
        };
    }, [item.contentHash, item.thumbnailStatus]);

    const isProcessing =
        item.hashStatus === "pending" ||
        item.metadataStatus === "pending" ||
        item.thumbnailStatus === "pending" ||
        item.hashStatus === "processing" ||
        item.metadataStatus === "processing" ||
        item.thumbnailStatus === "processing";

    const hasError =
        item.hashStatus === "error" ||
        item.metadataStatus === "error" ||
        item.thumbnailStatus === "error";

    return (
        <Box
            position="relative"
            aspectRatio="1"
            borderRadius="md"
            overflow="hidden"
            cursor="pointer"
            bg="gray.100"
            _hover={{ 
                bg: "gray.200"
            }}
            transition="background 0.2s"
            onClick={() => openViewer(item.mediaId)}
        >
            {/* Thumbnail image */}
            <Image
                src={thumbUrl ?? ""}
                alt={item.fileName ?? item.contentHash}
                w="full"
                h="full"
                objectFit="cover"
                fallback={<Box w="full" h="full" bg="gray.200" />}
            />

            {/* Processing overlay */}
            {isProcessing && (
                <Box
                    position="absolute"
                    inset="0"
                    bg="blackAlpha.600"
                    display="flex"
                    alignItems="center"
                    justifyContent="center"
                    flexDirection="column"
                    gap={2}
                >
                    <Spinner color="white" />
                    <Text color="white" fontSize="xs">
                        Processing...
                    </Text>
                </Box>
            )}

            {/* Error overlay */}
            {hasError && !isProcessing && (
                <Box
                    position="absolute"
                    inset="0"
                    bg="red.500/20"
                    display="flex"
                    alignItems="center"
                    justifyContent="center"
                >
                    <Text color="red.500" fontSize="xs" fontWeight="bold">
                        Error
                    </Text>
                </Box>
            )}

            {/* File name */}
            <Box
                position="absolute"
                bottom="0"
                left="0"
                right="0"
                bg="blackAlpha.700"
                px={2}
                py={1}
            >
                <Text
                    color="white"
                    fontSize="xs"
                    fontWeight="medium"
                    truncate
                >
                    {item.fileName ?? item.contentHash}
                </Text>
            </Box>
        </Box>
    );
};

export default MediaCard;
