import React from 'react';
import { MediaItem } from "@/types/mediaTypes.ts";
import { formatDurationFromMilliseconds } from "@/utils/timeUtils.ts";

interface MediaItemCardProps {
    mediaItem: MediaItem;
}

const MediaItemCard: React.FC<MediaItemCardProps> = ({ mediaItem }) => {
    const thumbnail = mediaItem.thumbUrl;

    return (
        <div
            className="border border-gray-200 rounded-md overflow-hidden shadow-sm hover:shadow-md transition-shadow max-w-[178px] max-h-[224px]">
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
            <div className="p-3">
                <h3 className="text-sm font-medium truncate" title={mediaItem.fileName}>
                    {mediaItem.fileName}
                </h3>
                <div className="mt-1 flex justify-between items-center">
                    <span className="text-xs text-gray-500 capitalize">
                        {mediaItem.mediaType}
                    </span>
                    {mediaItem.mediaType === 'video' && mediaItem.durationMs && (
                        <span className="text-xs text-gray-500">
                            {formatDurationFromMilliseconds(mediaItem.durationMs)}
                        </span>
                    )}
                </div>
            </div>
        </div>
    );
};

export default MediaItemCard;
