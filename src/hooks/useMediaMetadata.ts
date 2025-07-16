/**
 * Hook for fetching and managing media metadata
 * Provides functionality to load metadata for both images and videos
 */
import { useState, useEffect } from 'react';
import { extractVideoMetadata, extractImageMetadata } from '@/api/cacheApi';

/**
 * Common interface for media metadata
 */
export interface MediaMetadata {
  id: number;
  thumbnail_id: number;
  duration: number; // For videos, this is the duration in seconds; for images, it's 0
  thumbnailUrl: string; // URL using our custom protocol
}

/**
 * Custom hook for fetching and managing media metadata
 * @param {string} mediaPath - Path to the media file
 * @param {string} mediaType - Type of media ('image' or 'video')
 * @returns {Object} Media metadata state
 * @property {MediaMetadata|null} metadata - The media metadata
 * @property {boolean} loading - Whether the metadata is currently loading
 * @property {string|null} error - Error message if metadata loading failed
 */
export function useMediaMetadata(mediaPath: string, mediaType: 'image' | 'video') {
  const [metadata, setMetadata] = useState<MediaMetadata | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Reset state when media path changes
    setLoading(true);
    setError(null);
    setMetadata(null);

    /**
     * Fetch metadata for the media file
     */
    async function fetchMetadata() {
      try {
        let result;

        // Extract metadata based on media type
        if (mediaType === 'video') {
          result = await extractVideoMetadata(mediaPath);
        } else {
          result = await extractImageMetadata(mediaPath);
        }

        // Create a thumbnail URL using our custom protocol with the thumbnail_id
        const thumbnailUrl = `thumbnail://${result.thumbnail_id}`;

        // Create a new metadata object with the thumbnail URL
        const newMetadata: MediaMetadata = {
          ...result,
          thumbnailUrl
        };
        setMetadata(newMetadata);
      } catch (err) {
        console.error(`Error extracting ${mediaType} metadata:`, err);
        setError(`Failed to extract ${mediaType} metadata.`);
      } finally {
        setLoading(false);
      }
    }

    fetchMetadata();
  }, [mediaPath, mediaType]);

  return { metadata, loading, error };
}
