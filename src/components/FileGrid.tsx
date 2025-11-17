import MediaItemCard from "@/components/MediaItemCard";
import { Grid } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore.ts";

const FileGrid = () => {
    const items = useMediaStore(s => s.items);

    const minCardPx = 160;
    const gapPx = 8; // gap-2
    const paddingPx = 12;
    const count = items.length;

    const maxSingleRowWidth = count > 0
        ? count * minCardPx + (count - 1) * gapPx + 2 * paddingPx
        : 0;

    return (
        <Grid
            gap={2}
            maxWidth={maxSingleRowWidth ? `${maxSingleRowWidth}px` : undefined}
            w={"100%"}
            marginLeft={0}
            marginRight={"auto"}
            alignItems={"start"}
            padding={`${paddingPx}px`}
            className={`[grid-template-columns:repeat(auto-fit,minmax(var(--min-card),1fr))]`}
            style={{
                ["--min-card" as never]: `${minCardPx}px`
            }}
        >
            {items.map((item) => (
                <MediaItemCard key={item.mediaId} mediaItem={item} />
            ))}
        </Grid>
    );
};

export default FileGrid;
