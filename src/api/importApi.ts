import { invoke } from "@tauri-apps/api/core";
import { MediaItem } from "@/types/mediaTypes";

export async function import_files(): Promise<string> {
    return await invoke("import_files");
}

export async function get_import_folders(): Promise<string[]> {
    return await invoke("get_import_folders");
}

export async function get_items_in_import_folder(folderName: string): Promise<MediaItem[]> {
    return await invoke("get_items_in_import_folder", { folderName });
}
