/**
 * Header component for the application
 * Contains the app title, folder selection, and cache status
 */
import React from 'react';
import { Loader2 } from 'lucide-react';
import { CacheAction } from '@/hooks/useMediaCache';

interface HeaderProps {
  /**
   * Current folder path
   */
  folderPath: string | null;
  
  /**
   * Current cache action state
   */
  cacheAction: CacheAction;
  
  /**
   * Text descriptions for cache actions
   */
  cacheActionText: { [key: number]: string };
  
  /**
   * Handler for when the user clicks the button to pick a folder
   */
  onPickFolder: () => Promise<void>;
}

/**
 * Header component with app title, folder selection, and cache status
 */
const Header: React.FC<HeaderProps> = ({
  folderPath,
  cacheAction,
  cacheActionText,
  onPickFolder
}) => {
  return (
    <nav className="w-full flex items-center justify-between bg-gray-100 border-b border-gray-300 px-4 py-2 fixed top-0 z-10">
      {/* Left Side: App Name */}
      <div>
        <h2 className="text-lg font-bold">VidTrack</h2>
      </div>

      {/* Right Side: Folder display and Pick/Change button */}
      <div className="flex items-center space-x-4">
        {cacheAction !== CacheAction.Idle && (
          <div className="pr-4 flex items-center space-x-2">
            <Loader2 className="animate-spin h-5 w-5 text-gray-600" />
            <span className="text-sm text-gray-600">
              {cacheActionText[cacheAction]}
            </span>
          </div>
        )}
        <span className="text-sm text-gray-600">
          {folderPath ? folderPath : 'No folder selected'}
        </span>
        <button
          onClick={onPickFolder}
          className="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600 focus:outline-none focus:ring focus:ring-blue-300"
        >
          {folderPath ? 'Change Folder' : 'Pick Folder'}
        </button>
      </div>
    </nav>
  );
};

export default Header;
