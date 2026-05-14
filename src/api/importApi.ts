import { invoke } from "@tauri-apps/api/core";
import { type MediaItem } from "@/types/mediaTypes";
import { type ImportResult } from "@/types/importTypes";

export async function import_files(): Promise<ImportResult | null> {
    return await invoke("import_files");
}

export async function get_import_folders(): Promise<string[]> {
    return await invoke("get_import_folders");
}

export async function get_items_in_import_folder(folderName: string): Promise<MediaItem[]> {
    return await invoke("get_items_in_import_folder", { folderName });
}
