import {
    Box,
    type BoxProps,
    Dialog,
    Flex,
    Grid,
    type GridProps,
    Input,
    Portal,
    Text,
} from "@chakra-ui/react";
import { Search } from "lucide-react";
import { forwardRef, useEffect, useMemo, useRef, useState } from "react";
import { VirtuosoGrid } from "react-virtuoso";
import { MediaCard } from "@/components/library/MediaCard";
import { useSearchMedia } from "@/queries/search/useSearchMedia";
import { useTags } from "@/queries/tags/useTags";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { useViewerStore } from "@/stores/viewerStore";
import { applySuggestion, getSuggestions, parse, type Suggestion } from "./searchQuery";

const GridList = forwardRef<HTMLDivElement, GridProps>(({ style, children, ...props }, ref) => (
    <Grid
        ref={ref}
        {...props}
        templateColumns="repeat(auto-fill, minmax(140px, 1fr))"
        gap={3}
        p={3}
        style={style}
    >
        {children}
    </Grid>
));
GridList.displayName = "SearchGridList";

const GridItem = ({ children, ...props }: BoxProps) => <Box {...props}>{children}</Box>;

export function CommandPalette() {
    const isOpen = useInterfaceStore((s) => s.isSearchOpen);
    const setSearchOpen = useInterfaceStore((s) => s.setSearchOpen);

    return (
        <Dialog.Root
            open={isOpen}
            onOpenChange={(e) => setSearchOpen(e.open)}
            placement="top"
            motionPreset="slide-in-top"
        >
            <Portal>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content maxW="3xl" mt="8vh" overflow="hidden">
                        {/* Mount only while open so input state resets on each open (no reset effect). */}
                        {isOpen && <PaletteBody onClose={() => setSearchOpen(false)} />}
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}

function PaletteBody({ onClose }: { onClose: () => void }) {
    const inputRef = useRef<HTMLInputElement>(null);
    const [input, setInput] = useState("");
    const { tags } = useTags();
    const viewer = useViewerStore((s) => s.viewer);
    const hadViewerRef = useRef(viewer != null);

    const filters = useMemo(() => parse(input), [input]);
    const { results, isFetching } = useSearchMedia(filters, { enabled: true });
    const suggestions = useMemo(() => getSuggestions(input, tags), [input, tags]);

    useEffect(() => {
        inputRef.current?.focus();
    }, []);

    // Opening a result opens the viewer; close the palette so it isn't stranded behind it.
    useEffect(() => {
        if (!hadViewerRef.current && viewer) onClose();
        hadViewerRef.current = viewer != null;
    }, [viewer, onClose]);

    const onSuggestion = (suggestion: Suggestion) => {
        setInput(applySuggestion(input, suggestion));
        inputRef.current?.focus();
    };

    return (
        <>
            <Box px={4} pt={4}>
                <Flex align="center" gap={2}>
                    <Box color="gray.400" flexShrink={0}>
                        <Search size={18} />
                    </Box>
                    <Input
                        ref={inputRef}
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        placeholder="Search title… or tag:summer type:photo favorite:2"
                        variant="flushed"
                        size="lg"
                        autoFocus
                    />
                </Flex>

                {suggestions.length > 0 && (
                    <Flex wrap="wrap" gap={1.5} mt={2}>
                        {suggestions.map((s) => (
                            <Box
                                key={`${s.token}:${s.label}`}
                                as="button"
                                px={2}
                                py={0.5}
                                bg="whiteAlpha.100"
                                borderRadius="full"
                                _hover={{ bg: "whiteAlpha.200" }}
                                onClick={() => onSuggestion(s)}
                            >
                                <Text fontSize="xs" color="gray.300">
                                    {s.label}
                                    {s.hint ? (
                                        <Text as="span" color="gray.600" ml={1}>
                                            {s.hint}
                                        </Text>
                                    ) : null}
                                </Text>
                            </Box>
                        ))}
                    </Flex>
                )}
            </Box>

            <Box h="60vh" mt={3}>
                {results.length === 0 ? (
                    <Flex h="full" align="center" justify="center">
                        <Text fontSize="sm" color="gray.500">
                            {isFetching ? "Searching…" : "No results"}
                        </Text>
                    </Flex>
                ) : (
                    <VirtuosoGrid
                        style={{ height: "100%" }}
                        data={results}
                        totalCount={results.length}
                        computeItemKey={(_, item) => item.mediaId}
                        components={{ List: GridList, Item: GridItem }}
                        itemContent={(_, item) => (
                            <MediaCard
                                item={item}
                                mode={item.reviewedAt == null ? "import" : "library"}
                                allItems={results}
                            />
                        )}
                    />
                )}
            </Box>
        </>
    );
}
