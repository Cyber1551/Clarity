import { useEffect, useMemo, useState } from "react";
import {
    Box,
    Button,
    HStack,
    IconButton,
    Input,
    Kbd,
    Dialog,
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

    useEffect(() => {
        let mounted = true;
        async function load() {
            if (!viewer) return;
            setLoading(true);
            setError(null);
            try {
                const [d, tags] = await Promise.all([
                    get_media_detail(viewer.mediaId),
                    list_tags(),
                ]);
                if (!mounted) return;
                setDetail(d);
                setAllTags(tags);
            } catch (e: any) {
                setError(e?.toString?.() ?? "Failed to load media detail");
            } finally {
                setLoading(false);
            }
        }
        if (isOpen) {
            void load();
        } else {
            setDetail(null);
            setAllTags([]);
            setNewTagName("");
            setError(null);
        }
        return () => {
            mounted = false;
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
            setDetail({ ...detail, tags: [...detail.tags, created] });
            setAllTags((prev) => {
                if (prev.find((t) => t.id === created.id)) return prev;
                return [...prev, created];
            });
            setNewTagName("");
        } catch (e) {
            setError(e?.toString?.() ?? "Failed to create/tag");
        }
    }

    async function handleAddExistingTag(tagId: number) {
        if (!detail) return;
        const tag = allTags.find((t) => t.id === tagId);
        if (!tag) return;
        try {
            await tag_media(detail.mediaId, tagId);
            setDetail({ ...detail, tags: [...detail.tags, tag] });
        } catch (e) {
            setError(e?.toString?.() ?? "Failed to tag media");
        }
    }

    async function handleRemoveTag(tagId: number) {
        if (!detail) return;
        try {
            await untag_media(detail.mediaId, tagId);
            setDetail({ ...detail, tags: detail.tags.filter((t) => t.id !== tagId) });
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
                            {/* Left: Media preview placeholder (uses thumbnail for now) */}
                            <Box flex="2" bg="gray.900" display="flex" alignItems="center" justifyContent="center">
                                {loading && <Spinner color="white" />}
                                {!loading && detail && (
                                    <VStack gap={2} color="white">
                                        <Text fontSize="sm" color="gray.300">Media ID: {detail.mediaId}</Text>
                                        <Text fontSize="sm" color="gray.300">Type: {detail.mediaType}</Text>
                                        <Text fontSize="xs" color="gray.500">Hash: {detail.contentHash.slice(0, 10)}…</Text>
                                        <Text fontSize="xs" color="gray.500">
                                            Files: {detail.files.length}
                                        </Text>
                                        <Text fontSize="xs" color="gray.500">
                                            Dimensions: {detail.width ?? "?"} x {detail.height ?? "?"}
                                        </Text>
                                    </VStack>
                                )}
                            </Box>

                            {/* Right: Tag editor */}
                            <Box flex="1" p={4} overflowY="auto" borderLeft="1px solid" borderColor="gray.200">
                                {error && (
                                    <Box mb={3} color="red.500">
                                        {error}
                                    </Box>
                                )}
                                <VStack align="stretch" gap={4}>
                                    <Box>
                                        <Text fontWeight="bold" mb={2}>Tags</Text>
                                        <HStack gap={2} flexWrap="wrap">
                                            {detail?.tags.map((t) => (
                                                <Tag.Root key={t.id} size="md" borderRadius="full" variant="subtle" colorPalette="blue">
                                                    <Tag.Label>{t.name}</Tag.Label>
                                                    <Tag.CloseTrigger onClick={() => handleRemoveTag(t.id)} />
                                                </Tag.Root>
                                            ))}
                                            {detail && detail.tags.length === 0 && (
                                                <Text color="gray.500" fontSize="sm">No tags yet</Text>
                                            )}
                                        </HStack>
                                    </Box>

                                    <Box>
                                        <Text fontWeight="bold" mb={2}>Add Tag</Text>
                                        <HStack>
                                            <Input
                                                placeholder="Create new tag"
                                                value={newTagName}
                                                onChange={(e) => setNewTagName(e.target.value)}
                                                onKeyDown={(e) => {
                                                    if (e.key === "Enter") {
                                                        e.preventDefault();
                                                        void handleCreateAndAddTag();
                                                    }
                                                }}
                                            />
                                            <IconButton aria-label="Create tag" onClick={() => void handleCreateAndAddTag()}>
                                                <Plus size={16} />
                                            </IconButton>
                                        </HStack>
                                        {availableTags.length > 0 && (
                                            <HStack mt={3} gap={2} flexWrap="wrap">
                                                {availableTags.map((t) => (
                                                    <Button key={t.id} size="sm" onClick={() => void handleAddExistingTag(t.id)}>
                                                        + {t.name}
                                                    </Button>
                                                ))}
                                            </HStack>
                                        )}
                                        {availableTags.length === 0 && allTags.length > 0 && (
                                            <Text mt={2} fontSize="xs" color="gray.500">All existing tags already applied</Text>
                                        )}
                                    </Box>

                                    <Box color="gray.500" fontSize="xs">
                                        <Text>Tips:</Text>
                                        <Text>
                                            • Press <Kbd>Enter</Kbd> to create a new tag quick.
                                        </Text>
                                    </Box>
                                </VStack>
                            </Box>
                        </Dialog.Body>
                        <Dialog.Footer>
                            <Button onClick={() => void handleMarkReviewed()} disabled={isReviewing}>
                                {isReviewing ? (
                                    <HStack gap={2}>
                                        <Spinner size="sm" />
                                        <Text>Marking…</Text>
                                    </HStack>
                                ) : (
                                    "Mark as reviewed"
                                )}
                            </Button>
                            <Dialog.ActionTrigger asChild>
                                <Button variant="outline" onClick={closeViewer}>Close</Button>
                            </Dialog.ActionTrigger>
                        </Dialog.Footer>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}
