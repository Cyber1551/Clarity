import { MediaFeedItem } from "@/types/mediaTypes";
import { Box, Image, Spinner, Text, HStack, Badge } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { Tooltip } from "@/components/ui/tooltip";

interface MediaCardProps {
    item: MediaFeedItem;
}

const MediaCard = ({ item }: MediaCardProps) => {
    const openViewer = useMediaStore(s => s.openViewer);
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

    const displayTags = item.tags.slice(0, 2);
    const remainingTags = item.tags.slice(2);

    return (
        <Box
            position="relative"
            aspectRatio="1"
            borderRadius="md"
            overflow="hidden"
            cursor="pointer"
            bg="gray.100"
            _hover={{ 
                bg: "gray.200",
                "& .tag-overlay": { opacity: 1 }
            }}
            transition="background 0.2s"
            onClick={() => openViewer(item.mediaId)}
        >
            {/* Thumbnail image */}
            <Image
                src={item.thumbnailDataUrl}
                alt={item.fileName ?? item.contentHash}
                w="full"
                h="full"
                objectFit="cover"
            />

            {/* Tag Overlay - Top right */}
            {item.tags.length > 0 && (
                <Box
                    className="tag-overlay"
                    position="absolute"
                    top={2}
                    right={2}
                    display="flex"
                    flexDirection="column"
                    alignItems="flex-end"
                    gap={1}
                    opacity={0.8}
                    transition="opacity 0.2s"
                >
                    <HStack gap={1}>
                        {displayTags.map(tag => (
                            <Badge 
                                key={tag.id} 
                                variant="solid" 
                                colorPalette="blue"
                                size="sm"
                                fontSize="2xs"
                                px={1.5}
                                borderRadius="full"
                                textTransform="none"
                            >
                                {tag.name}
                            </Badge>
                        ))}
                        {remainingTags.length > 0 && (
                            <Tooltip 
                                content={remainingTags.map(t => t.name).join(", ")}
                                showArrow
                            >
                                <Badge 
                                    variant="solid" 
                                    colorPalette="gray"
                                    size="sm"
                                    fontSize="2xs"
                                    px={1.5}
                                    borderRadius="full"
                                >
                                    +{remainingTags.length}
                                </Badge>
                            </Tooltip>
                        )}
                    </HStack>
                </Box>
            )}

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
