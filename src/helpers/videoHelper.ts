import { promisify } from "util";
import { exec } from "child_process";

const execPromise = promisify(exec);

// Generate a thumbnail for videos (use `ffmpeg` or any library you like)
export async function generateVideoThumbnail(filePath: string): Promise<string | null> {
    const thumbnailPath = `${filePath}-thumbnail.jpg`;

    try {
        await execPromise(
            `ffmpeg -i "${filePath}" -ss 00:00:01.000 -vframes 1 "${thumbnailPath}"`
        );
        return `file:///${thumbnailPath}`;
    } catch (error) {
        console.error("Error generating video thumbnail:", error);
        return null;
    }
}

// Get video length using ffprobe
export async function getVideoLength(filePath: string): Promise<number | null> {
    try {
        const { stdout } = await execPromise(
            `ffprobe -v error -show_entries format=duration -of csv=p=0 "${filePath}"`
        );
        return parseFloat(stdout.trim());
    } catch (error) {
        console.error("Error getting video length:", error);
        return null;
    }
}
