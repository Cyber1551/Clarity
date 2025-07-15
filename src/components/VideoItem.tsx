// src/VideoItem.tsx
import React, { useEffect, useState } from 'react';
import {convertFileSrc, invoke} from '@tauri-apps/api/tauri';

interface VideoMetadata {
    thumbnail_path: string; // Path to the thumbnail file
    duration: number;  // Duration in seconds
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
                // Call the Tauri command to extract video metadata
                const result = await invoke<VideoMetadata>('extract_video_metadata', { path: videoPath });
                console.log("RESULT: ", result);

                // Create a thumbnail URL using our custom protocol
                const thumbnailUrl = `thumbnail://${result.thumbnail_path}`;

                // Create a new metadata object with the thumbnail URL
                const newMeta = {
                    thumbnail_path: result.thumbnail_path,
                    thumbnailUrl: thumbnailUrl,
                    duration: result.duration
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
