import { appConfigDir, join } from "@tauri-apps/api/path";
import {
    createDir,
    readTextFile,
    writeTextFile,
    BaseDirectory,
} from "@tauri-apps/plugin-fs";

/**
 * Returns the full path to our config.json file.
 * e.g. ~/.config/myapp/config.json (Linux/macOS)
 *      C:\Users\<User>\AppData\Roaming\myapp\config.json (Windows)
 */
async function getConfigPath(): Promise<string> {
    const configDir = await appConfigDir();
    return join(configDir, "vidtrack", "config.json");
}

/**
 * Reads the config JSON and returns the stored folderPath.
 * If file doesn't exist or JSON is invalid, return null.
 */
export async function getStoredFolder(): Promise<string | null> {
    try {
        const configFile = await getConfigPath();
        const contents = await readTextFile(configFile);
        const json = JSON.parse(contents);
        return json.folderPath ?? null;
    } catch (err) {
        console.error(err)
        // File likely doesn't exist yet or JSON is invalid
        return null;
    }
}

/**
 * Writes the chosen folder path to our config.json file.
 */
export async function setStoredFolder(folderPath: string): Promise<void> {
    const configFile = await getConfigPath();

    // Make sure the parent directory exists
    // We'll remove "/config.json" from the path to get the parent
    const parentDir = configFile.replace(/[/\\]config\.json$/, "");
    await createDir(parentDir, {
        recursive: true,
        dir: BaseDirectory.AppData,
    });

    const config = { folderPath };
    await writeTextFile(configFile, JSON.stringify(config, null, 2), {
        dir: BaseDirectory.AppData,
    });
}
