import { type SyncReport, type SyncStatus } from "@/types/mediaTypes";
import { _invoke } from "./_invoke";

export async function sync_library(): Promise<SyncReport> {
    return await _invoke("sync_library");
}

export async function rebuild_library(): Promise<SyncReport> {
    return await _invoke("rebuild_library");
}

export async function get_sync_status(): Promise<SyncStatus> {
    return await _invoke("get_sync_status");
}
