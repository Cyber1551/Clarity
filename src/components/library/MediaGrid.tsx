import { type MediaItem } from "@/types/mediaTypes";
import { Box, type BoxProps, Grid, type GridProps, Spinner, Text } from "@chakra-ui/react";
import { MediaCard } from "./MediaCard";
import { VirtuosoGrid, type VirtuosoGridHandle } from "react-virtuoso";
import { forwardRef, useEffect, useMemo, useRef } from "react";
import { type ViewerMode } from "@/stores/mediaStore";

interface MediaGridProps {
    items: MediaItem[];
    isLoading: boolean;
    error: string | null;
    mode: ViewerMode;
    scrollToMediaId?: number | null;
    onScrolledToMediaId?: (mediaId: number) => void;
}

const GridList = forwardRef<HTMLDivElement, GridProps>(({ style, children, ...props }, ref) => (
    <Grid
        ref={ref}
        {...props}
        templateColumns="repeat(auto-fill, minmax(160px, 1fr))"
        gap={4}
        p={4}
        style={style}
    >
        {children}
    </Grid>
));
GridList.displayName = "GridList";

const GridItem = ({ children, ...props }: BoxProps) => (
    <Box {...props}>
        {children}
    </Box>
);

// Three-tier scroll-to-index timing for VirtuosoGrid: the initial rAF call
// triggers an immediate scroll, the retry compensates for items that hadn't
// measured their final size yet, and the done timer notifies the caller
// after both attempts have settled.
const SCROLL_RETRY_MS = 80;
const SCROLL_DONE_MS = 140;

export const MediaGrid = ({ items, isLoading, error, mode, scrollToMediaId, onScrolledToMediaId }: MediaGridProps) => {
    const virtuosoRef = useRef<VirtuosoGridHandle>(null);
    const indexMap = useMemo(() => {
        const map = new Map<number, number>();
        items.forEach((item, index) => map.set(item.mediaId, index));
        return map;
    }, [items]);

    useEffect(() => {
        if (!scrollToMediaId || isLoading) return;

        const index = indexMap.get(scrollToMediaId);
        if (index === undefined) return;

        const handle = virtuosoRef.current;
        if (!handle) return;

        const scroll = () => handle.scrollToIndex({ index, align: "center", behavior: "smooth" });
        const raf = requestAnimationFrame(scroll);
        const retry = setTimeout(scroll, SCROLL_RETRY_MS);
        const done = setTimeout(() => {
            onScrolledToMediaId?.(scrollToMediaId);
        }, SCROLL_DONE_MS);
        return () => {
            cancelAnimationFrame(raf);
            clearTimeout(retry);
            clearTimeout(done);
        };
    }, [indexMap, isLoading, onScrolledToMediaId, scrollToMediaId]);

    if (isLoading) {
        return (
            <Box
                display="flex"
                alignItems="center"
                justifyContent="center"
                minH="400px"
            >
                <Spinner size="xl" />
            </Box>
        );
    }

    if (error) {
        return (
            <Box
                display="flex"
                alignItems="center"
                justifyContent="center"
                minH="400px"
            >
                <Text color="red.500">{error}</Text>
            </Box>
        );
    }

    if (items.length === 0) {
        return (
            <Box
                display="flex"
                alignItems="center"
                justifyContent="center"
                minH="400px"
                flexDirection="column"
                gap={2}
            >
                <Text fontSize="lg" fontWeight="semibold">
                    No media found
                </Text>
                <Text fontSize="sm" color="gray.500">
                    Add media files to your library to get started
                </Text>
            </Box>
        );
    }

    return (
        <VirtuosoGrid
            ref={virtuosoRef}
            style={{ height: "100%" }}
            data={items}
            totalCount={items.length}
            computeItemKey={(_, item) => item.mediaId}
            components={{
                List: GridList,
                Item: GridItem,
            }}
            itemContent={(_, item) => (
                <MediaCard item={item} mode={mode} allItems={items} />
            )}
        />
    );
};
