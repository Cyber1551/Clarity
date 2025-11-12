import React from 'react';

interface HeaderProps {
  folderPath: string | null;
  cacheActionText: { [key: number]: string };
  onPickFolder: () => Promise<void>;
}

const Header: React.FC<HeaderProps> = ({
  folderPath,
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
