/**
 * MainContent component for displaying the main content area
 */
import React from 'react';
import { MediaItem } from '@/types/media_item';
import MediaGrid from '@/components/MediaGrid';

interface MainContentProps {
  /**
   * Current folder path
   */
  folderPath: string | null;
  
  /**
   * Array of media items to display
   */
  mediaItems: MediaItem[];
}

/**
 * Component for displaying the main content area
 */
const MainContent: React.FC<MainContentProps> = ({ folderPath, mediaItems }) => {
  return (
    <main className="flex-grow pt-16 p-4">
      {!folderPath ? (
        <div className="flex flex-col items-center justify-center h-full text-center">
          <p className="text-gray-500 mb-4">
            No folder selected. Please click "Pick Folder" in the top menu.
          </p>
        </div>
      ) : (
        <div>
          <h3 className="text-xl font-semibold mb-4">Media Items</h3>
          {mediaItems.length === 0 ? (
            <p className="text-gray-500 text-center">
              No media items found in the selected folder.
            </p>
          ) : (
            <MediaGrid mediaItems={mediaItems} />
          )}
        </div>
      )}
    </main>
  );
};

export default MainContent;
