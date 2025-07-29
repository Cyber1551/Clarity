// src/VideoItem.tsx
import React, { useEffect, useState } from 'react';
import { extractVideoMetadata } from '@/api/cacheApi';
import { MediaMetadata } from '@/hooks/useMediaMetadata';

type Props = {
    videoPath: string;
    title: string;
}

const VideoItem: React.FC<Props> = ({ videoPath, title }) => {
    const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function fetchMetadata() {
            try {
                // Call the function to extract video metadata
                const result = await extractVideoMetadata(videoPath);
                console.log("Video metadata result:", result);
                setMetadata(result);
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
                        src={metadata.thumbnail_base64}
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
