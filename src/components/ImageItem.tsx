import React, { useEffect, useState } from 'react';
import { extractImageMetadata } from '@/api/cacheApi';

// Interface matching the Rust VideoMetadata struct (used for both videos and images)
interface ImageMetadata {
    id: number;           // Database ID of the media item
    thumbnail_id: number; // Database ID of the thumbnail
    duration: number;     // Duration is always 0 for images
}

// Extended metadata with thumbnail URL
interface ExtendedImageMetadata extends ImageMetadata {
    thumbnailUrl: string; // URL using our custom protocol
}

type Props = {
    imagePath: string;
    title: string;
}

const ImageItem: React.FC<Props> = ({ imagePath, title }) => {
    const [metadata, setMetadata] = useState<ExtendedImageMetadata | null>(null);
    const [loading, setLoading] = useState<boolean>(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        async function fetchMetadata() {
            try {
                // Call the function to extract image metadata
                const result = await extractImageMetadata(imagePath);
                console.log("Image metadata result:", result);

                // Create a thumbnail URL using our custom protocol with the thumbnail_id
                const thumbnailUrl = `thumbnail://${result.thumbnail_id}`;

                // Create a new metadata object with the thumbnail URL
                const newMeta = {
                    ...result,
                    thumbnailUrl
                };

                console.log("Image with thumbnail URL:", newMeta);
                setMetadata(newMeta);
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
                    src={metadata.thumbnailUrl}
                    alt={title}
                    style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                />
            )}
        </div>
    );
};

export default ImageItem;
