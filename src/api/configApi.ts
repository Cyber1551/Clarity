import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "@/types/configTypes.ts";

export async function getAppConfig(): Promise<AppConfig> {
    return await invoke<AppConfig>("get_app_config");
}

export async function chooseLibraryRoot(): Promise<string | null> {
    return await invoke<string | null>("choose_library_root");
}