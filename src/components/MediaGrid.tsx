/**
 * MediaGrid component for displaying a grid of media items
 */
import React from 'react';
import { MediaItem } from "@/types/media_item";
import MediaItemCard from "@/components/MediaItemCard";

interface MediaGridProps {
  /**
   * Array of media items to display in the grid
   */
  mediaItems: MediaItem[];
}

/**
 * Component for displaying a grid of media items
 */
const MediaGrid: React.FC<MediaGridProps> = ({ mediaItems }) => {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4">
      {mediaItems.map((item) => (
        <MediaItemCard key={item.path} item={item} />
      ))}
    </div>
  );
};

export default MediaGrid;
