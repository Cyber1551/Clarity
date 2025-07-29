/**
 * API utilities for error handling and common patterns
 */

/**
 * Wraps an API call with consistent error handling
 * @param {Function} apiCall - The API function to call
 * @param {string} errorMessage - Custom error message prefix
 * @param {any[]} args - Arguments to pass to the API function
 * @returns {Promise<T>} The result of the API call
 * @throws {Error} If the API call fails
 */
export async function withErrorHandling<T>(
  apiCall: (...args: any[]) => Promise<T>,
  errorMessage: string,
  ...args: any[]
): Promise<T> {
  try {
    return await apiCall(...args);
  } catch (error) {
    console.error(`${errorMessage}:`, error);
    throw error instanceof Error 
      ? error 
      : new Error(`${errorMessage}: ${error}`);
  }
}

/**
 * Wraps an API call with consistent error handling and returns a default value on error
 * @param {Function} apiCall - The API function to call
 * @param {T} defaultValue - Default value to return on error
 * @param {string} errorMessage - Custom error message prefix
 * @param {any[]} args - Arguments to pass to the API function
 * @returns {Promise<T>} The result of the API call or the default value on error
 */
export async function withErrorHandlingAndDefault<T>(
  apiCall: (...args: any[]) => Promise<T>,
  defaultValue: T,
  errorMessage: string,
  ...args: any[]
): Promise<T> {
  try {
    return await apiCall(...args);
  } catch (error) {
    console.error(`${errorMessage}:`, error);
    return defaultValue;
  }
}

/**
 * Creates a debounced version of a function that delays invoking func until after wait milliseconds
 * @param {Function} func - The function to debounce
 * @param {number} wait - The number of milliseconds to delay
 * @returns {Function} The debounced function
 */
export function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: NodeJS.Timeout | null = null;
  
  return function(...args: Parameters<T>): void {
    const later = () => {
      timeout = null;
      func(...args);
    };
    
    if (timeout !== null) {
      clearTimeout(timeout);
    }
    
    timeout = setTimeout(later, wait);
  };
}
