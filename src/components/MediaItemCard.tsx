/**
 * MediaItemCard component for displaying individual media items
 * Handles both image and video items with thumbnails
 */
import React from 'react';
import { MediaItem } from '@/types/media_item';

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
  // Thumbnails are now included directly in the media item
  // No need to fetch them separately
  const thumbnail = item.thumbnail_base64;

  return (
    <div className="border border-gray-200 rounded-md overflow-hidden shadow-sm hover:shadow-md transition-shadow max-w-[178px] max-h-[224px]">
      {/* Thumbnail/Preview Section */}
      <div className="h-40 relative bg-gray-100">
        {!thumbnail && (
          <div className="flex justify-center items-center h-full">
            <p className="text-red-500 text-sm">Error displaying thumbnail</p>
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
