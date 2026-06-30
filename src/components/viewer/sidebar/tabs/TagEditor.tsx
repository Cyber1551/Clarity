import { Box, Flex, IconButton, Input, Text } from "@chakra-ui/react";
import { X } from "lucide-react";
import { useMemo, useState, type KeyboardEvent } from "react";
import { useMediaTags } from "@/queries/tags/useMediaTags";
import { useTags } from "@/queries/tags/useTags";
import { useAddMediaTag } from "@/queries/tags/useAddMediaTag";
import { useRemoveMediaTag } from "@/queries/tags/useRemoveMediaTag";

interface TagEditorProps {
    mediaId: number;
}

/** Tag chips + add input for a single media item. */
export function TagEditor({ mediaId }: TagEditorProps) {
    const { tags } = useMediaTags(mediaId);
    const { tags: allTags } = useTags();
    const addTag = useAddMediaTag();
    const removeTag = useRemoveMediaTag();
    const [value, setValue] = useState("");

    const assignedIds = useMemo(() => new Set(tags.map((t) => t.id)), [tags]);
    const query = value.trim().toLowerCase();
    const suggestions = useMemo(
        () =>
            query.length === 0
                ? []
                : allTags
                      .filter(
                          (t) =>
                              !assignedIds.has(t.id) && t.name.toLowerCase().includes(query),
                      )
                      .slice(0, 5),
        [allTags, assignedIds, query],
    );

    const submit = (name: string) => {
        const trimmed = name.trim();
        if (!trimmed) return;
        addTag.mutate({ mediaId, name: trimmed });
        setValue("");
    };

    const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter") {
            e.preventDefault();
            submit(value);
        }
    };

    return (
        <Box>
            <Text fontSize="xs" color="gray.400" fontWeight="medium" mb={2}>
                Tags
            </Text>

            <Flex wrap="wrap" gap={1.5} mb={2}>
                {tags.map((tag) => (
                    <Flex
                        key={tag.id}
                        align="center"
                        gap={1}
                        pl={2}
                        pr={1}
                        py={0.5}
                        bg="whiteAlpha.150"
                        borderRadius="full"
                    >
                        <Text fontSize="2xs" color="gray.200">
                            {tag.name}
                        </Text>
                        <IconButton
                            aria-label={`Remove ${tag.name}`}
                            size="2xs"
                            variant="ghost"
                            color="gray.400"
                            _hover={{ color: "red.300" }}
                            onClick={() => removeTag.mutate({ mediaId, tagId: tag.id })}
                        >
                            <X size={10} />
                        </IconButton>
                    </Flex>
                ))}
                {tags.length === 0 && (
                    <Text fontSize="2xs" color="gray.600">
                        No tags yet
                    </Text>
                )}
            </Flex>

            <Input
                value={value}
                onChange={(e) => setValue(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="Add a tag"
                size="xs"
                variant="subtle"
            />

            {suggestions.length > 0 && (
                <Flex wrap="wrap" gap={1.5} mt={1.5}>
                    {suggestions.map((tag) => (
                        <Box
                            key={tag.id}
                            as="button"
                            px={2}
                            py={0.5}
                            bg="whiteAlpha.100"
                            borderRadius="full"
                            _hover={{ bg: "whiteAlpha.200" }}
                            onClick={() => submit(tag.name)}
                        >
                            <Text fontSize="2xs" color="gray.300">
                                {tag.name}
                            </Text>
                        </Box>
                    ))}
                </Flex>
            )}
        </Box>
    );
}
