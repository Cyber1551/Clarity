/**
 * Hook for database operations related to media
 * Provides functions for initializing, refreshing, and querying the media database
 */
import {useCallback, useState} from 'react';
import { MediaItem } from '@/types/media_item';
import { getAllMedia } from '@/api/cacheApi';
import { invoke } from '@tauri-apps/api/core';

/**
 * Database operation states
 */
export enum DatabaseOperation {
  Idle,
  Initializing,
  Updating,
  Error
}

/**
 * Custom hook for database operations related to media
 * @returns {Object} Database operations and state
 */
export function useMediaDatabase() {
  const [operationState, setOperationState] = useState<DatabaseOperation>(DatabaseOperation.Idle);
  const [error, setError] = useState<string | null>(null);

  /**
   * Initialize the database for a folder
   * @param {string} folderPath - Path to the folder
   * @returns {Promise<void>}
   */
  const initializeDatabase = useCallback(async (folderPath: string): Promise<void> => {
    try {
      setOperationState(DatabaseOperation.Initializing);
      await invoke('initialize_database', { folderPath });
      setOperationState(DatabaseOperation.Idle);
    } catch (error) {
      console.error("Error initializing database:", error);
      setOperationState(DatabaseOperation.Error);
      setError(`Failed to initialize database: ${error}`);
      throw error;
    }
  }, []);

  /**
   * Scan a directory and process all media files
   * @param {string} folderPath - Path to the folder to scan
   * @returns {Promise<MediaItem[]>} Updated media items
   */
  const scanDirectory = async (folderPath: string): Promise<MediaItem[]> => {
    try {
      setOperationState(DatabaseOperation.Updating);
      // This will be implemented in the cacheHelper.ts file
      // We'll call the existing updateMediaCache function
      await invoke('scan_directory', { folderPath });
      setOperationState(DatabaseOperation.Idle);
      return await getMediaItems();
    } catch (error) {
      console.error("Error scanning directory:", error);
      setOperationState(DatabaseOperation.Error);
      setError(`Failed to scan directory: ${error}`);
      throw error;
    }
  };

  /**
   * Get all media items from the database
   * @returns {Promise<MediaItem[]>} Media items
   */
  const getMediaItems = useCallback(async (): Promise<MediaItem[]> => {
    try {
      return await getAllMedia();
    } catch (error) {
      console.error("Error getting media items:", error);
      setError(`Failed to get media items: ${error}`);
      throw error;
    }
  }, []);

  /**
   * Text descriptions for database operation states
   */
  const operationStateText: { [key: number]: string } = {
    [DatabaseOperation.Initializing]: "Initializing Database...",
    [DatabaseOperation.Updating]: "Updating Database...",
    [DatabaseOperation.Error]: "Database Error! Please restart the application.",
  };

  return {
    operationState,
    error,
    initializeDatabase,
    scanDirectory,
    getMediaItems,
    operationStateText
  };
}
