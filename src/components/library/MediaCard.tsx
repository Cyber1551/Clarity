import { type MediaItem, type JobStatus } from "@/types/mediaTypes";
import { Box, Image, Spinner, Text } from "@chakra-ui/react";
import { useMediaStore, type ViewerMode } from "@/stores/mediaStore";
import { memo, type KeyboardEvent } from "react";
import { get_thumbnail } from "@/api/libraryApi";
import { keyframes } from "@emotion/react";
import { useObjectUrlFromBlob } from "@/hooks/useObjectUrlFromBlob";

interface MediaCardProps {
    item: MediaItem;
    mode: ViewerMode;
    allItems: MediaItem[];
}

const pulse = keyframes`
    0% { box-shadow: 0 0 0 0 rgba(66, 153, 225, 0.9); }
    70% { box-shadow: 0 0 0 10px rgba(66, 153, 225, 0); }
    100% { box-shadow: 0 0 0 0 rgba(66, 153, 225, 0); }
`;

const IN_FLIGHT_STATUSES: ReadonlySet<JobStatus> = new Set(["pending", "processing"]);

const MediaCard = ({ item, mode, allItems }: MediaCardProps) => {
    const openViewer = useMediaStore(s => s.openViewer);
    const highlightedMediaId = useMediaStore(s => s.highlightedMediaId);

    const thumbUrl = useObjectUrlFromBlob(
        () => get_thumbnail(item.contentHash),
        [item.contentHash, item.thumbnailStatus],
        {
            enabled: item.thumbnailStatus === "done",
            onError: (e) => console.error("Failed to load thumbnail", e),
        }
    );

    const isHighlighted = highlightedMediaId === item.mediaId;

    const isProcessing =
        IN_FLIGHT_STATUSES.has(item.hashStatus) ||
        IN_FLIGHT_STATUSES.has(item.metadataStatus) ||
        IN_FLIGHT_STATUSES.has(item.thumbnailStatus);

    const hasError =
        item.hashStatus === "error" ||
        item.metadataStatus === "error" ||
        item.thumbnailStatus === "error";

    const handleOpen = () => openViewer(item.mediaId, mode, allItems);
    const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
        if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            handleOpen();
        }
    };

    const label = item.fileName ?? item.contentHash;

    return (
        <Box
            position="relative"
            aspectRatio="4/3"
            borderRadius="md"
            overflow="hidden"
            cursor="pointer"
            bg="gray.100"
            _hover={{ bg: "gray.200" }}
            transition="background 0.2s"
            outline={isHighlighted ? "2px solid" : "none"}
            outlineColor={isHighlighted ? "blue.400" : "transparent"}
            animation={isHighlighted ? `${pulse} 1.6s ease-out infinite` : "none"}
            onClick={handleOpen}
            onKeyDown={handleKeyDown}
            role="button"
            tabIndex={0}
            aria-label={`Open ${label}`}
        >
            <Image
                src={thumbUrl ?? undefined}
                alt={label}
                w="full"
                h="full"
                objectFit="contain"
            />

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
                    {label}
                </Text>
            </Box>
        </Box>
    );
};

export const MemoizedMediaCard = memo(MediaCard);
export { MemoizedMediaCard as MediaCard };
