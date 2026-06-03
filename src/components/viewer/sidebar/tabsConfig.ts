import { type MediaDetail } from "@/types/mediaTypes";
import { type ViewerMode } from "@/stores/viewerStore";

export type ViewerTabId = "info" | "files" | "subtitles";

export interface ViewerTabConfig {
    id: ViewerTabId;
    label: string;
    /** When omitted, the tab is visible in every mode. */
    visibleInModes?: readonly ViewerMode[];
    /** Optional badge accessor (e.g. file count). */
    badge?: (detail: MediaDetail | null) => string | number | null;
}

export const VIEWER_TABS: readonly ViewerTabConfig[] = [
    { id: "info", label: "Info" },
    {
        id: "files",
        label: "Files",
        visibleInModes: ["library"],
        badge: (detail) => detail?.files.length ?? null,
    },
    { id: "subtitles", label: "Subtitles" },
];

export function getVisibleTabs(mode: ViewerMode): readonly ViewerTabConfig[] {
    return VIEWER_TABS.filter((t) => !t.visibleInModes || t.visibleInModes.includes(mode));
}
