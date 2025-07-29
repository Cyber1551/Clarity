import React, { useEffect, useState } from 'react';
import { extractImageMetadata } from '@/api/cacheApi';
import { MediaMetadata } from '@/hooks/useMediaMetadata';

type Props = {
    imagePath: string;
    title: string;
}

const ImageItem: React.FC<Props> = ({ imagePath, title }) => {
    const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function fetchMetadata() {
            try {
                // Call the function to extract image metadata
                const result = await extractImageMetadata(imagePath);
                console.log("Image metadata result:", result);
                setMetadata(result);
            } catch (err) {
                console.error('Error extracting image metadata:', err);
                setError('Failed to extract metadata.');
            } finally {
                setLoading(false);
            }
        }
        fetchMetadata();
    }, [imagePath]);

    return (
        <div style={{ height: '100%', width: '100%', position: 'relative' }}>
            {loading && <p>Loading image...</p>}
            {error && <p style={{ color: 'red' }}>{error}</p>}
            {metadata && (
                <img
                    src={metadata.thumbnail_base64}
                    alt={title}
                    style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                />
            )}
        </div>
    );
};

export default ImageItem;
