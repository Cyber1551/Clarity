import { useEffect, useMemo, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import {
    Box,
    Button,
    HStack,
    IconButton,
    Input,
    Dialog,
    Image,
    Portal,
    Spinner,
    Tag,
    Text,
    VStack,
} from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { create_tag, get_media_detail, list_tags, tag_media, untag_media, mark_media_reviewed } from "@/api/libraryApi";
import { MediaDetail, Tag as TagType } from "@/types/mediaTypes";
import { Plus } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";

export function Viewer() {
    const viewer = useMediaStore((s) => s.viewer);
    const closeViewer = useMediaStore((s) => s.closeViewer);
    const isOpen = Boolean(viewer);

    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [detail, setDetail] = useState<MediaDetail | null>(null);
    const [allTags, setAllTags] = useState<TagType[]>([]);
    const [newTagName, setNewTagName] = useState("");
    const [isReviewing, setIsReviewing] = useState(false);

    const loadData = async () => {
        if (!viewer) return;
        setLoading(true);
        setError(null);
        try {
            const [d, tags] = await Promise.all([
                get_media_detail(viewer.mediaId),
                list_tags(),
            ]);
            setDetail(d);
            setAllTags(tags);
        } catch (e: any) {
            setError(e?.toString?.() ?? "Failed to load media detail");
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        let active = true;
        async function init() {
            if (!isOpen || !viewer) {
                setDetail(null);
                setAllTags([]);
                setNewTagName("");
                setError(null);
                return;
            }

            setLoading(true);
            setError(null);
            try {
                const [d, tags] = await Promise.all([
                    get_media_detail(viewer.mediaId),
                    list_tags(),
                ]);
                if (!active) return;
                setDetail(d);
                setAllTags(tags);
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

    const availableTags = useMemo(() => {
        const current = new Set(detail?.tags.map((t) => t.id) ?? []);
        return allTags.filter((t) => !current.has(t.id));
    }, [allTags, detail?.tags]);

    async function handleCreateAndAddTag() {
        const name = newTagName.trim();
        if (!name || !detail) return;
        try {
            const created = await create_tag(name);
            await tag_media(detail.mediaId, created.id);
            setNewTagName("");
            await loadData();
        } catch (e) {
            setError(e?.toString?.() ?? "Failed to create/tag");
        }
    }

    async function handleAddExistingTag(tagId: number) {
        if (!detail) return;
        try {
            await tag_media(detail.mediaId, tagId);
            await loadData();
        } catch (e) {
            setError(e?.toString?.() ?? "Failed to tag media");
        }
    }

    async function handleRemoveTag(tagId: number) {
        if (!detail) return;
        try {
            await untag_media(detail.mediaId, tagId);
            await loadData();
        } catch (e) {
            setError(e?.toString?.() ?? "Failed to untag media");
        }
    }

    async function handleMarkReviewed() {
        if (!detail) return;
        try {
            setIsReviewing(true);
            await mark_media_reviewed(detail.mediaId);
            // Backend emits events; close the viewer here
            closeViewer();
        } catch (e) {
            setError(e?.toString?.() ?? "Failed to mark as reviewed");
        } finally {
            setIsReviewing(false);
        }
    }

    return (
        <Dialog.Root
            placement="center"
            motionPreset="slide-in-bottom"
            open={isOpen}
            onOpenChange={({ open }) => {
                if (!open) closeViewer();
            }}
        >
            <Portal>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content maxW="90vw" h="80vh" overflow="hidden">
                        <Dialog.Header>
                            <Dialog.Title>Viewer</Dialog.Title>
                        </Dialog.Header>
                        <Dialog.Body p={0} display="flex" h="calc(80vh - 64px)">
                            {/* Left: Media preview */}
                            <Box flex="3" bg="black" display="flex" alignItems="center" justifyContent="center" overflow="hidden">
                                {loading && <Spinner color="white" size="xl" />}
                                {!loading && detail && (
                                    <Image
                                        src={convertFileSrc(detail.canonicalPath)}
                                        alt={detail.contentHash}
                                        maxW="100%"
                                        maxH="100%"
                                        objectFit="contain"
                                    />
                                )}
                                {!loading && !detail && !error && (
                                    <Text color="gray.500">No media loaded</Text>
                                )}
                            </Box>

                            {/* Right: Sidebar */}
                            <Box flex="1" p={4} overflowY="auto" borderLeft="1px solid" borderColor="whiteAlpha.200" bg="gray.900">
                                {error && (
                                    <Box mb={3} color="red.400" fontSize="sm">
                                        {error}
                                    </Box>
                                )}
                                
                                {detail && (
                                    <VStack align="stretch" gap={6}>
                                        {/* Metadata Section */}
                                        <Box>
                                            <Text fontWeight="bold" fontSize="xs" mb={2} color="gray.400" textTransform="uppercase" letterSpacing="wider">Info</Text>
                                            <VStack align="stretch" gap={1.5}>
                                                <HStack justify="space-between">
                                                    <Text fontSize="xs" color="gray.400">Type</Text>
                                                    <Text fontSize="xs" fontWeight="medium" color="gray.100">{detail.mediaType}</Text>
                                                </HStack>
                                                <HStack justify="space-between">
                                                    <Text fontSize="xs" color="gray.400">Dimensions</Text>
                                                    <Text fontSize="xs" fontWeight="medium" color="gray.100">{detail.width ?? "?"} × {detail.height ?? "?"}</Text>
                                                </HStack>
                                                <HStack justify="space-between">
                                                    <Text fontSize="xs" color="gray.400">Hash</Text>
                                                    <Tooltip content={detail.contentHash} showArrow portalled={false}>
                                                        <Text fontSize="xs" fontWeight="medium" fontFamily="mono" color="gray.100" cursor="help">
                                                            {detail.contentHash.slice(0, 8)}…
                                                        </Text>
                                                    </Tooltip>
                                                </HStack>
                                                <HStack justify="space-between">
                                                    <Text fontSize="xs" color="gray.400">Files</Text>
                                                    <Text fontSize="xs" fontWeight="medium" color="gray.100">{detail.files.length}</Text>
                                                </HStack>
                                            </VStack>
                                        </Box>

                                        {/* Tags Section */}
                                        <Box>
                                            <Text fontWeight="bold" fontSize="xs" mb={2} color="gray.400" textTransform="uppercase" letterSpacing="wider">Tags</Text>
                                            <HStack gap={1} flexWrap="wrap" mb={3}>
                                                {detail.tags.map((t) => (
                                                    <Tag.Root key={t.id} size="sm" borderRadius="full" variant="subtle" colorPalette="blue">
                                                        <Tag.Label>{t.name}</Tag.Label>
                                                        <Tag.CloseTrigger onClick={() => handleRemoveTag(t.id)} />
                                                    </Tag.Root>
                                                ))}
                                                {detail.tags.length === 0 && (
                                                    <Text color="gray.500" fontSize="xs" fontStyle="italic">No tags yet</Text>
                                                )}
                                            </HStack>

                                            <HStack gap={1} mb={2}>
                                                <Input
                                                    size="sm"
                                                    placeholder="Add tag..."
                                                    value={newTagName}
                                                    onChange={(e) => setNewTagName(e.target.value)}
                                                    onKeyDown={(e) => {
                                                        if (e.key === "Enter") {
                                                            e.preventDefault();
                                                            void handleCreateAndAddTag();
                                                        }
                                                    }}
                                                    bg="whiteAlpha.50"
                                                    borderColor="whiteAlpha.200"
                                                    _focus={{ borderColor: "blue.500" }}
                                                />
                                                <IconButton 
                                                    size="sm" 
                                                    aria-label="Add tag" 
                                                    variant="ghost"
                                                    onClick={() => void handleCreateAndAddTag()}
                                                    disabled={!newTagName.trim()}
                                                    color="gray.400"
                                                >
                                                    <Plus size={14} />
                                                </IconButton>
                                            </HStack>

                                            {availableTags.length > 0 && (
                                                <Box>
                                                    <Text fontSize="2xs" color="gray.500" mb={1.5} textTransform="uppercase">Suggested</Text>
                                                    <HStack gap={1} flexWrap="wrap">
                                                        {availableTags.slice(0, 12).map((t) => (
                                                            <Button 
                                                                key={t.id} 
                                                                size="2xs" 
                                                                variant="outline" 
                                                                onClick={() => void handleAddExistingTag(t.id)}
                                                                borderRadius="full"
                                                                fontWeight="normal"
                                                                fontSize="2xs"
                                                                color="gray.300"
                                                                borderColor="whiteAlpha.300"
                                                                _hover={{ bg: "whiteAlpha.100" }}
                                                            >
                                                                {t.name}
                                                            </Button>
                                                        ))}
                                                    </HStack>
                                                </Box>
                                            )}
                                        </Box>
                                    </VStack>
                                )}
                            </Box>
                        </Dialog.Body>
                        <Dialog.Footer gap={3}>
                            <Button
                                colorPalette="blue"
                                size="sm"
                                loading={isReviewing}
                                onClick={() => void handleMarkReviewed()}
                                disabled={!detail}
                            >
                                Mark as Reviewed
                            </Button>
                            <Dialog.ActionTrigger asChild>
                                <Button variant="outline" size="sm" onClick={closeViewer}>Close</Button>
                            </Dialog.ActionTrigger>
                        </Dialog.Footer>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}
