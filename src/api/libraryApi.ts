import { invoke } from "@tauri-apps/api/core";


export async function initialize_library_dirs(): Promise<void> {
    await invoke("initialize_library_dirs");
}

