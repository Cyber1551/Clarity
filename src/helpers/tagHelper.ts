import {Mediaitem} from "@/types/media_item.ts";
import fs from "fs";
import {readCache} from "@/helpers/cacheHelper.ts";

export async function updateTags(filePath: string, newTags: string[], cacheFile: string) {
    const cache = await readCache(cacheFile);

    const file = cache.find((item: Mediaitem) => item.path === filePath);
    if (file) {
        file.tags = newTags;
        await fs.promises.writeFile(cacheFile, JSON.stringify(cache, null, 2));
    } else {
        throw new Error(`File not found in cache: ${filePath}`);
    }
}