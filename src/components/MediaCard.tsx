import { MediaItem } from "@/types/mediaTypes";
import { Box, Image, Spinner, Text } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { useEffect, useMemo, useState } from "react";
import { get_thumbnail } from "@/api/libraryApi";
import { keyframes } from "@emotion/react";

interface MediaCardProps {
    item: MediaItem;
}

const MediaCard = ({ item }: MediaCardProps) => {
    const openViewer = useMediaStore(s => s.openViewer);
    const highlightedMediaId = useMediaStore(s => s.highlightedMediaId);
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

    const isHighlighted = highlightedMediaId === item.mediaId;
    const pulse = useMemo(() => keyframes`
        0% { box-shadow: 0 0 0 0 rgba(66, 153, 225, 0.9); }
        70% { box-shadow: 0 0 0 10px rgba(66, 153, 225, 0); }
        100% { box-shadow: 0 0 0 0 rgba(66, 153, 225, 0); }
    `, []);

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
            aspectRatio="4/3"
            borderRadius="md"
            overflow="hidden"
            cursor="pointer"
            bg="gray.100"
            _hover={{ 
                bg: "gray.200"
            }}
            transition="background 0.2s"
            outline={isHighlighted ? "2px solid" : "none"}
            outlineColor={isHighlighted ? "blue.400" : "transparent"}
            animation={isHighlighted ? `${pulse} 1.6s ease-out infinite` : "none"}
            onClick={() => openViewer(item.mediaId)}
        >
            {/* Thumbnail image */}
            <Image
                src={thumbUrl ?? undefined}
                alt={item.fileName ?? item.contentHash}
                w="full"
                h="full"
                objectFit="contain"
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
