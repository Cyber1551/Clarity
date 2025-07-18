/**
 * MediaItemCard component for displaying individual media items
 * Handles both image and video items with thumbnails
 */
import React, { useState } from 'react';
import { MediaItem } from '@/types/media_item';
import { useThumbnail } from '@/hooks/useThumbnail';

interface MediaItemCardProps {
  /**
   * The media item to display
   */
  item: MediaItem;
}

/**
 * Component for displaying an individual media item card
 */
const MediaItemCard: React.FC<MediaItemCardProps> = ({ item }) => {
  // Default thumbnail size
  const thumbnailSize = 64;

  // Extract the media ID from the item
  // In a real implementation, you would get this from the item object
  // For now, we'll parse it from the path as a workaround
  const [mediaId] = useState<number>(() => {
    // This is a temporary solution - in a real app, the mediaId would be part of the MediaItem
    // For now, we'll use a hash of the path to simulate an ID
    return Math.abs(item.path.split('').reduce((acc, char) => {
      return acc + char.charCodeAt(0);
    }, 0));
  });

  // Use the thumbnail hook to fetch the thumbnail
  const { thumbnail, loading, error } = useThumbnail(mediaId, thumbnailSize);

  return (
    <div className="border border-gray-200 rounded-md overflow-hidden shadow-sm hover:shadow-md transition-shadow">
      {/* Thumbnail/Preview Section */}
      <div className="h-40 relative bg-gray-100">
        {loading && (
          <div className="flex justify-center items-center h-full">
            <p className="text-gray-500 text-sm">Loading...</p>
          </div>
        )}

        {error && (
          <div className="flex justify-center items-center h-full">
            <p className="text-red-500 text-sm">{error}</p>
          </div>
        )}

        {thumbnail && (
          <img
            src={thumbnail}
            alt={"Thumbnail"}
            className="w-full h-full object-cover"
            onError={(e) => console.error("Error displaying thumbnail:", e)}
          />
        )}
      </div>

      {/* Info Section */}
      <div className="p-3">
        <h3 className="text-sm font-medium truncate" title={item.title}>
          {item.title}
        </h3>

        <div className="mt-1 flex justify-between items-center">
          <span className="text-xs text-gray-500 capitalize">
            {item.type}
          </span>

          {item.type === 'video' && item.length && (
            <span className="text-xs text-gray-500">
              {formatDuration(item.length)}
            </span>
          )}
        </div>

        {item.tags && item.tags.length > 0 && (
          <div className="mt-2 flex flex-wrap gap-1">
            {item.tags.map((tag, index) => (
              <span 
                key={index} 
                className="px-1.5 py-0.5 bg-blue-100 text-blue-800 rounded text-xs"
              >
                {tag}
              </span>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

/**
 * Format duration in seconds to a human-readable string (MM:SS)
 * @param {number} seconds - Duration in seconds
 * @returns {string} Formatted duration string
 */
function formatDuration(seconds: number): string {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
}

export default MediaItemCard;
