import { FormatByte, HStack, IconButton, Text, VStack } from "@chakra-ui/react";
import { type MediaDetail } from "@/types/mediaTypes";
import { type ViewerMode } from "@/stores/viewerStore";
import { Star, Gem, ChevronDown, ChevronUp, Copy, Check, Users, Bookmark } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";
import { useState } from "react";
import {
    update_favorite_rating,
    update_quality_rating,
} from "@/api/libraryApi";
import {
    ComingSoonRow,
    FormatDuration,
    FormatRelativeTime,
    MetadataRow,
    RatingRow,
} from "@/components/common";
import { useCopyToClipboard } from "@/hooks/useCopyToClipboard";

interface InfoTabProps {
    detail: MediaDetail;
    mode: ViewerMode;
    onDetailChanged: () => void;
}

export function InfoTab({ detail, mode, onDetailChanged }: InfoTabProps) {
    const [advancedOpen, setAdvancedOpen] = useState(false);
    const [copyHash, hashCopied] = useCopyToClipboard();
    const originalFileName = detail.files?.[0]?.originalFileName ?? null;

    const handleQualityChange = async (newRating: number) => {
        await update_quality_rating(detail.mediaId, newRating);
        onDetailChanged();
    };

    const handleFavoriteChange = async (newRating: number) => {
        await update_favorite_rating(detail.mediaId, newRating);
        onDetailChanged();
    };

    return (
        <VStack align="stretch" gap={5} flex={1} overflowY="auto" px={1}>
            <VStack align="stretch" gap={1.5}>
                {detail.width && detail.height && (
                    <MetadataRow label="Resolution" value={`${detail.width} x ${detail.height}`} />
                )}
                {detail.durationMs != null && (
                    <MetadataRow label="Duration" value={<FormatDuration value={detail.durationMs} />} />
                )}
                {detail.sizeBytes > 0 && (
                    <MetadataRow label="Size" value={<FormatByte value={detail.sizeBytes} />} />
                )}
                <MetadataRow
                    label="Imported"
                    value={<FormatRelativeTime value={detail.createdAt} />}
                    valueWrapper={(node) => (
                        <Tooltip content={new Date(detail.createdAt).toLocaleString()}>
                            {node}
                        </Tooltip>
                    )}
                />
            </VStack>

            <VStack align="stretch" gap={2}>
                <Text fontSize="xs" color="gray.400" fontWeight="medium">Ratings</Text>
                <RatingRow
                    label="Favorite"
                    icon={Star}
                    value={detail.favoriteRating}
                    colorActive="purple.400"
                    colorHover="purple.300"
                    onChange={(v) => void handleFavoriteChange(v)}
                />
                <RatingRow
                    label="Quality"
                    icon={Gem}
                    value={detail.qualityRating}
                    colorActive="yellow.400"
                    colorHover="yellow.300"
                    onChange={(v) => void handleQualityChange(v)}
                />
            </VStack>

            <ComingSoonRow heading="People" emptyText="No people tagged" icon={Users} actionLabel="Tag someone" />

            {detail.mediaType === "video" && (
                <ComingSoonRow heading="Markers" emptyText="No bookmarks or moments" icon={Bookmark} actionLabel="Add bookmark" />
            )}

            <VStack align="stretch" gap={1}>
                <HStack
                    cursor="pointer"
                    onClick={() => setAdvancedOpen(!advancedOpen)}
                    _hover={{ color: "gray.300" }}
                    color="gray.500"
                    transition="color 0.15s"
                >
                    <Text fontSize="xs" fontWeight="medium">Advanced</Text>
                    {advancedOpen ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
                </HStack>
                {advancedOpen && (
                    <VStack align="stretch" gap={1.5} pl={1}>
                        {originalFileName && (
                            <MetadataRow size="2xs" label="Original" value={originalFileName} />
                        )}
                        <HStack gap={1}>
                            <Text fontSize="2xs" color="gray.500" wordBreak="break-all" flex={1}>
                                {detail.contentHash.slice(0, 16)}...
                            </Text>
                            <Tooltip content={hashCopied ? "Copied!" : "Copy hash"}>
                                <IconButton
                                    aria-label="Copy hash"
                                    variant="ghost"
                                    size="xs"
                                    color="gray.500"
                                    onClick={() => void copyHash(detail.contentHash)}
                                >
                                    {hashCopied ? <Check size={12} /> : <Copy size={12} />}
                                </IconButton>
                            </Tooltip>
                        </HStack>
                        {mode === "library" && (
                            <MetadataRow size="2xs" label="Hardlinks" value={detail.files.length} />
                        )}
                    </VStack>
                )}
            </VStack>
        </VStack>
    );
}
