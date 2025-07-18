/**
 * Hook for managing media cache
 * Provides functions to update and access media items
 * Uses useMediaDatabase for database operations
 */
import { useState, useCallback, useEffect } from 'react';
import { MediaItem } from '@/types/media_item';
import { useMediaDatabase, DatabaseOperation } from '@/hooks/useMediaDatabase';
import { updateMediaCache, initializeDatabase as initDb } from '@/helpers/cacheHelper';

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
    operationState
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
      // Use the existing updateMediaCache function for now
      // In a future update, this could be replaced with scanDirectory
      const items = await updateMediaCache(folderPath);
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
      await initDb(folderPath);

      // Use the existing updateMediaCache function for now
      // In a future update, this could be replaced with scanDirectory
      const items = await updateMediaCache(folderPath);
      setMediaItems(items);
      return items;
    } catch (error) {
      console.error("Error initializing media cache:", error);
      throw error;
    }
  }, [setMediaItems]);

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
