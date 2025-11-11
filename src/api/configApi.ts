// src/config.ts
import { BaseDirectory, readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';

export type AppConfig = {
    folderPath: string | null;
};

const CONFIG_FILENAME = 'app_config.json';

/**
 * Loads the configuration file.
 * Returns an object with folderPath or a default config if file is missing.
 */
export async function loadConfig(): Promise<AppConfig> {
    try {
        const text = await readTextFile(CONFIG_FILENAME, { baseDir: BaseDirectory.AppConfig });
        return JSON.parse(text);
    } catch (error) {
        console.error("Error loading config: ", error)
        // If the config file doesn't exist or cannot be read, return default config.
        return { folderPath: null };
    }
}

/**
 * Saves the provided configuration to disk.
 */
export async function saveConfig(config: AppConfig): Promise<void> {
    await writeTextFile(CONFIG_FILENAME, JSON.stringify(config), { dir: BaseDirectory.AppConfig });
}
