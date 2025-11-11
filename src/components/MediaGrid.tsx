import React from 'react';
import { MediaItem } from "@/types/media_item";
import MediaItemCard from "@/components/MediaItemCard";

interface MediaGridProps {
  mediaItems: MediaItem[];
}

/**
 * Component for displaying a grid of media items
 */
const MediaGrid: React.FC<MediaGridProps> = ({ mediaItems }) => {
    const minCard = 160; // px
    const gap = 8; // px (gap-2)
    const count = mediaItems.length;
    const maxSingleRowWidth = count > 0
        ? count * minCard + (count - 1) * gap
        : 0;

    return (
        <div
            className={`grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(var(--min-card),1fr))]`}
            style={{
                maxWidth: maxSingleRowWidth ? `${maxSingleRowWidth}px` : undefined,
                width: "100%",
                marginLeft: 0,
                marginRight: "auto",
                alignItems: "start",
                ["--min-card" as never]: `${minCard}px`
            }}
        >
            {mediaItems.map((item) => (
                <MediaItemCard key={item.path} item={item} />
            ))}
        </div>
    );
};

export default MediaGrid;
