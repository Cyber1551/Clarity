import { readDir, FileEntry } from "@tauri-apps/api/fs";

export async function listMediaFiles(
    folderPath: string,
    mediaType: "pictures" | "videos"
) {
    const extensions =
        mediaType === "pictures"
            ? ["jpg", "jpeg", "png", "gif", "bmp", "webp"]
            : ["mp4", "mov", "avi", "mkv", "webm"];

    const results: { title: string; path: string }[] = [];

    try {
        // readDir can optionally be recursive. We'll keep it shallow here:
        const entries = await readDir(folderPath, { recursive: false });
        gatherFiles(entries, extensions, results);
    } catch (err) {
        console.error("Error reading directory:", err);
    }

    return results;
}

function gatherFiles(
    entries: FileEntry[],
    exts: string[],
    collector: { title: string; path: string }[]
) {
    for (const entry of entries) {
        if (entry.children && entry.children.length > 0) {
            // If you wanted to handle subfolders recursively, do:
            // gatherFiles(entry.children, exts, collector);
            // but for now we skip recursion.
        } else if (entry.name) {
            const lowerName = entry.name.toLowerCase();
            if (exts.some((ext) => lowerName.endsWith(`.${ext}`))) {
                collector.push({
                    title: entry.name,
                    path: entry.path,
                });
            }
        }
    }
}
