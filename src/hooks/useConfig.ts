/**
 * Hook for managing application configuration
 * Provides functions to load and save configuration settings
 */
import { useState, useEffect } from 'react';
import { loadConfig, saveConfig, AppConfig } from '@/api/configApi';

/**
 * Custom hook for managing application configuration
 * @returns {Object} Configuration state and functions
 * @property {AppConfig} config - Current configuration state
 * @property {boolean} isLoading - Loading state
 * @property {Function} updateConfig - Function to update configuration
 * @property {Error|null} error - Error state
 */
export function useConfig() {
  const [config, setConfig] = useState<AppConfig>({ folderPath: null });
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  // Load configuration on component mount
  useEffect(() => {
    async function initConfig() {
      try {
        setIsLoading(true);
        const loadedConfig = await loadConfig();
        setConfig(loadedConfig);
      } catch (err) {
        setError(err instanceof Error ? err : new Error('Failed to load configuration'));
        console.error('Error loading configuration:', err);
      } finally {
        setIsLoading(false);
      }
    }

    initConfig();
  }, []);

  /**
   * Update configuration and save to disk
   * @param {AppConfig} newConfig - New configuration to save
   */
  const updateConfig = async (newConfig: AppConfig) => {
    try {
      setIsLoading(true);
      await saveConfig(newConfig);
      setConfig(newConfig);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to save configuration'));
      console.error('Error saving configuration:', err);
    } finally {
      setIsLoading(false);
    }
  };

  return { config, isLoading, updateConfig, error };
}
