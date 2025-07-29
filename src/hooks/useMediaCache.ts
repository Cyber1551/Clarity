/**
 * Hook for managing media cache
 * Provides functions to update and access media items
 * Uses useMediaDatabase for database operations
 */
import { useState, useCallback, useEffect } from 'react';
import { MediaItem } from '@/types/media_item';
import { useMediaDatabase, DatabaseOperation } from '@/hooks/useMediaDatabase';
import {getAllMedia, updateMediaCache} from '@/api/cacheApi';

/**
 * Cache action states
 */
export enum CacheAction {
  Idle,
  Initializing,
  Updating,
  Error
}

/**
 * Custom hook for managing media cache
 * @returns {Object} Media cache state and functions
 * @property {MediaItem[]} mediaItems - Current media items
 * @property {CacheAction} cacheAction - Current cache action state
 * @property {Function} refreshCache - Function to refresh the media cache
 * @property {Object} cacheActionText - Text descriptions for cache actions
 */
export function useMediaCache() {
  const [mediaItems, setMediaItems] = useState<MediaItem[]>([]);
  const [cacheAction, setCacheAction] = useState<CacheAction>(CacheAction.Idle);

  // Use the database hook for database operations
  const { 
    operationState,
    initializeDatabase,
    getMediaItems
  } = useMediaDatabase();

  // Map database operation state to cache action
  useEffect(() => {
    switch (operationState) {
      case DatabaseOperation.Initializing:
        setCacheAction(CacheAction.Initializing);
        break;
      case DatabaseOperation.Updating:
        setCacheAction(CacheAction.Updating);
        break;
      case DatabaseOperation.Error:
        setCacheAction(CacheAction.Error);
        break;
      case DatabaseOperation.Idle:
        setCacheAction(CacheAction.Idle);
        break;
    }
  }, [operationState]);

  /**
   * Refresh the media cache for a specific folder
   * @param {string} folderPath - Path to the folder to scan
   * @returns {Promise<MediaItem[]>} Updated media items
   */
  const refreshCache = useCallback(async (folderPath: string): Promise<MediaItem[]> => {
    try {
      // Update the media cache using the Rust implementation
      const stats = await updateMediaCache(folderPath);
      console.log("Media cache updated:", stats);
      
      // Get the updated media items
      const items = await getAllMedia();
      setMediaItems(items);
      return items;
    } catch (error) {
      console.error("Error updating media cache:", error);
      throw error;
    }
  }, [setMediaItems]);

  /**
   * Initialize the media cache for a new folder
   * @param {string} folderPath - Path to the folder to initialize
   * @returns {Promise<MediaItem[]>} Initialized media items
   */
  const initializeCache = useCallback(async (folderPath: string): Promise<MediaItem[]> => {
    try {
      // Initialize the database
      await initializeDatabase(folderPath);

      // Update the media cache using the Rust implementation
      //const stats = await updateMediaCache(folderPath);
      //console.log("Media cache initialized:", stats);
      
      // Get the updated media items
      const items = await getMediaItems();
      setMediaItems(items);
      return items;
    } catch (error) {
      console.error("Error initializing media cache:", error);
      throw error;
    }
  }, [initializeDatabase, getMediaItems, setMediaItems]);

  /**
   * Text descriptions for cache action states
   */
  const cacheActionText: { [key: number]: string } = {
    [CacheAction.Initializing]: "Initializing Cache...",
    [CacheAction.Updating]: "Updating Cache...",
    [CacheAction.Error]: "Error generating cache, please restart the program!",
  };

  return { 
    mediaItems, 
    cacheAction, 
    refreshCache, 
    initializeCache, 
    cacheActionText 
  };
}
