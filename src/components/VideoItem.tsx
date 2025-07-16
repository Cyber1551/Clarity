// src/VideoItem.tsx
import React, { useEffect, useState } from 'react';
import { extractVideoMetadata } from '@/api/cacheApi';

// Interface matching the Rust VideoMetadata struct
interface VideoMetadata {
    id: number;           // Database ID of the media item
    thumbnail_id: number; // Database ID of the thumbnail
    duration: number;     // Duration in seconds
}

// Extended metadata with thumbnail URL
interface ExtendedVideoMetadata extends VideoMetadata {
    thumbnailUrl: string; // URL using our custom protocol
}

type Props = {
    videoPath: string;
    title: string;
}

const VideoItem: React.FC<Props> = ({ videoPath, title }) => {
    const [metadata, setMetadata] = useState<ExtendedVideoMetadata | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function fetchMetadata() {
            try {
                // Call the function to extract video metadata
                const result = await extractVideoMetadata(videoPath);
                console.log("RESULT: ", result);

                // Create a thumbnail URL using our custom protocol with the thumbnail_id
                const thumbnailUrl = `thumbnail://${result.thumbnail_id}`;

                // Create a new metadata object with the thumbnail URL
                const newMeta = {
                    ...result,
                    thumbnailUrl
                };

                console.log("NEW: ", newMeta);
                setMetadata(newMeta);
            } catch (err) {
                console.error('Error extracting video metadata:', err);
                setError('Failed to extract metadata.');
            } finally {
                setLoading(false);
            }
        }
        fetchMetadata();
    }, [videoPath]);

    return (
        <div style={{ border: '1px solid #ccc', padding: '0.5rem', borderRadius: '4px' }}>
            <h4>{title}</h4>
            {loading && <p>Loading metadata...</p>}
            {error && <p style={{ color: 'red' }}>{error}</p>}
            {metadata && (
                <div>
                    <img
                        src={metadata.thumbnailUrl}
                        alt={`${title} thumbnail`}
                        style={{ width: '100%', height: 'auto', objectFit: 'cover' }}
                    />
                    <p>Duration: {metadata.duration} sec</p>
                </div>
            )}
        </div>
    );
};

export default VideoItem;
