import { type AppConfig } from "@/types/configTypes.ts";

import { _invoke } from "./_invoke";

export async function getAppConfig(): Promise<AppConfig> {
    return await _invoke<AppConfig>("get_app_config");
}

export async function chooseLibraryRoot(): Promise<string | null> {
    return await _invoke<string | null>("choose_library_root");
}

export async function openLibraryRoot(): Promise<void> {
    await _invoke("open_library_root");
}
