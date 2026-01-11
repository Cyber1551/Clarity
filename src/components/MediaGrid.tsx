import { MediaFeedItem } from "@/types/mediaTypes";
import { Box, Grid, Spinner, Text } from "@chakra-ui/react";
import MediaCard from "./MediaCard";
import { VirtuosoGrid } from "react-virtuoso";
import { forwardRef } from "react";

interface MediaGridProps {
    items: MediaFeedItem[];
    isLoading: boolean;
    error: string | null;
}

const GridList = forwardRef<HTMLDivElement, any>(({ style, children, ...props }, ref) => (
    <Grid
        ref={ref}
        {...props}
        templateColumns="repeat(auto-fill, minmax(200px, 1fr))"
        gap={4}
        p={4}
        style={style}
    >
        {children}
    </Grid>
));

const GridItem = ({ children, ...props }: any) => (
    <Box {...props}>
        {children}
    </Box>
);

const MediaGrid = ({ items, isLoading, error }: MediaGridProps) => {
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
            style={{ height: "100%" }}
            data={items}
            totalCount={items.length}
            components={{
                List: GridList,
                Item: GridItem
            }}
            itemContent={(index, item) => (
                <MediaCard key={item.mediaId} item={item} />
            )}
        />
    );
};

export default MediaGrid;
