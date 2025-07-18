/**
 * Hook for managing thumbnails at different sizes
 * Provides functionality to fetch or generate thumbnails as needed
 */
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

/**
 * Interface for thumbnail data
 */
export interface ThumbnailData {
  base64: string;
  size: number;
}

/**
 * Custom hook for managing thumbnails
 * @param {number} mediaId - ID of the media item
 * @param {number} size - Desired thumbnail size (e.g., 16, 32, 64)
 * @returns {Object} Thumbnail state
 * @property {string|null} thumbnail - Base64 encoded thumbnail image
 * @property {boolean} loading - Whether the thumbnail is currently loading
 * @property {string|null} error - Error message if thumbnail loading failed
 */
export function useThumbnail(mediaId: number, size: number) {
  const [thumbnail, setThumbnail] = useState<string | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Reset state when media ID or size changes
    setLoading(true);
    setError(null);
    setThumbnail(null);

    /**
     * Fetch or generate thumbnail
     */
    async function fetchThumbnail() {
      try {
        // First check if the thumbnail exists at the specified size
        const exists = await invoke<boolean>('check_thumbnail_exists', { 
          mediaId,
          size 
        });

        let thumbnailBase64: string;

        if (exists) {
          // If it exists, just fetch it
          thumbnailBase64 = await invoke<string>('get_thumbnail', { 
            mediaId, 
            size 
          });
        } else {
          // If it doesn't exist, generate it
          thumbnailBase64 = await invoke<string>('generate_thumbnail', { 
            mediaId, 
            size 
          });
        }

        setThumbnail(thumbnailBase64);
      } catch (err) {
        console.error(`Error fetching thumbnail for media ${mediaId}:`, err);
        setError(`Failed to fetch thumbnail.`);
      } finally {
        setLoading(false);
      }
    }

    if (mediaId) {
      fetchThumbnail();
    } else {
      setLoading(false);
    }
  }, [mediaId, size]);

  return { thumbnail, loading, error };
}
