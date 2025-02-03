import {Mediaitem} from "@/types/media_item.ts";
import fs from "fs";
import {readCache} from "@/helpers/cacheHelper.ts";

export async function addBookmark(filePath: string, description: string, timestamp: number, cacheFile: string) {
    const cache = await readCache(cacheFile);

    const file = cache.find((item: Mediaitem) => item.path === filePath);
    if (file && file.type === "video") {
        file.bookmarks.push({ description, timestamp });
        await fs.promises.writeFile(cacheFile, JSON.stringify(cache, null, 2));
    } else {
        throw new Error(`File not found or not a video: ${filePath}`);
    }
}

export async function removeBookmark(filePath: string, timestamp: number, cacheFile: string) {
    const cache = await readCache(cacheFile);

    const file = cache.find((item: Mediaitem) => item.path === filePath);
    if (file && file.type === "video") {
        file.bookmarks = file.bookmarks.filter((bm) => bm.timestamp !== timestamp);
        await fs.promises.writeFile(cacheFile, JSON.stringify(cache, null, 2));
    } else {
        throw new Error(`File not found or not a video: ${filePath}`);
    }
}
