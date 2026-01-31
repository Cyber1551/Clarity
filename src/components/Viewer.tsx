import { useEffect, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
    Box,
    HStack,
    IconButton,
    Dialog,
    Image,
    Portal,
    Spinner,
    Text,
    VStack,
} from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { get_media_detail } from "@/api/libraryApi";
import { MediaDetail } from "@/types/mediaTypes";
import { Heart, Star } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";

export function Viewer() {
    const viewer = useMediaStore((s) => s.viewer);
    const closeViewer = useMediaStore((s) => s.closeViewer);
    const isOpen = Boolean(viewer);

    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [detail, setDetail] = useState<MediaDetail | null>(null);

    useEffect(() => {
        let active = true;
        async function init() {
            if (!isOpen || !viewer) {
                setDetail(null);
                setError(null);
                return;
            }

            setLoading(true);
            setError(null);
            try {
                const d = await get_media_detail(viewer.mediaId);
                if (!active) return;
                setDetail(d);
            } catch (e: any) {
                if (!active) return;
                setError(e?.toString?.() ?? "Failed to load media detail");
            } finally {
                if (active) setLoading(false);
            }
        }
        
        void init();
        
        return () => {
            active = false;
        };
    }, [isOpen, viewer]);

    return (
        <Dialog.Root open={isOpen} onOpenChange={(e) => !e.open && closeViewer()} size="full">
            <Portal>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content bg="black" color="white" h="100vh" w="100vw" m={0} p={0} border="none">
                        <Dialog.CloseTrigger pos="absolute" top={4} right={4} color="white" />
                        
                        <HStack h="full" w="full" spacing={0}>
                            {/* Main Content Area */}
                            <Box flex={1} position="relative" display="flex" alignItems="center" justifyContent="center" bg="black" overflow="hidden">
                                {loading && <Spinner size="xl" />}
                                {error && <Text color="red.400">{error}</Text>}
                                
                                {detail && !loading && (
                                    <Box maxH="full" maxW="full">
                                        {detail.mediaType === "image" ? (
                                            <Image
                                                src={convertFileSrc(detail.canonicalPath)}
                                                alt={detail.contentHash}
                                                maxH="100vh"
                                                objectFit="contain"
                                            />
                                        ) : (
                                            <video
                                                src={convertFileSrc(detail.canonicalPath)}
                                                controls
                                                autoPlay
                                                style={{ maxHeight: "100vh", maxWidth: "100%" }}
                                            />
                                        )}
                                    </Box>
                                )}
                            </Box>

                            {/* Sidebar Info */}
                            <VStack w="320px" bg="gray.900" h="full" p={6} align="stretch" spacing={6} borderLeftWidth="1px" borderColor="gray.800">
                                <Dialog.Header p={0}>
                                    <Dialog.Title fontSize="xl">Media Detail</Dialog.Title>
                                </Dialog.Header>

                                {detail && (
                                    <>
                                        <VStack align="stretch" spacing={2}>
                                            <Text fontWeight="bold" color="gray.400" fontSize="sm">INFO</Text>
                                            <Text fontSize="sm">Type: {detail.mediaType}</Text>
                                            {detail.width && <Text fontSize="sm">Resolution: {detail.width} x {detail.height}</Text>}
                                            {detail.durationMs && <Text fontSize="sm">Duration: {(detail.durationMs / 1000).toFixed(1)}s</Text>}
                                            <Text fontSize="xs" color="gray.500" wordBreak="break-all">Hash: {detail.contentHash}</Text>
                                        </VStack>

                                        <VStack align="stretch" spacing={4}>
                                            <Text fontWeight="bold" color="gray.400" fontSize="sm">RATINGS</Text>
                                            <HStack>
                                                <IconButton 
                                                    aria-label="Love" 
                                                    variant={detail.loved ? "solid" : "ghost"} 
                                                    colorPalette={detail.loved ? "red" : "gray"}
                                                >
                                                    <Heart fill={detail.loved ? "currentColor" : "none"} />
                                                </IconButton>

                                                <HStack spacing={1}>
                                                    {[1, 2, 3].map((star) => (
                                                        <IconButton
                                                            key={star}
                                                            aria-label={`Rate ${star}`}
                                                            variant="ghost"
                                                            size="sm"
                                                            color={detail.rating >= star ? "yellow.400" : "gray.600"}
                                                        >
                                                            <Star fill={detail.rating >= star ? "currentColor" : "none"} />
                                                        </IconButton>
                                                    ))}
                                                </HStack>
                                            </HStack>
                                        </VStack>

                                        <VStack align="stretch" spacing={2} flex={1} overflowY="auto">
                                            <Text fontWeight="bold" color="gray.400" fontSize="sm">PATHS ({detail.files.length})</Text>
                                            {detail.files.map((f) => (
                                                <Tooltip key={f.id} content={f.relPath} showArrow>
                                                    <Box p={2} bg="gray.800" borderRadius="md" fontSize="xs">
                                                        <Text isTruncated>{f.fileName}{f.ext ? `.${f.ext}` : ""}</Text>
                                                        <Text fontSize="2xs" color="gray.500" isTruncated>{f.dirPath}</Text>
                                                    </Box>
                                                </Tooltip>
                                            ))}
                                        </VStack>
                                    </>
                                )}
                            </VStack>
                        </HStack>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}
