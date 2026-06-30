import { create } from "zustand";
import { type ImportResult } from "@/types/importTypes";
import { type ActiveTab, DEFAULT_ACTIVE_TAB, isActiveTab } from "@/constants/tabs";

interface InterfaceState {
    isSettingsDialogOpen: boolean;
    setSettingsDialogOpen: (open: boolean) => void;
    activeTab: ActiveTab;
    setActiveTab: (tab: string) => void;
    selectedImportFolder: string | null;
    setSelectedImportFolder: (folder: string | null) => void;
    lastImportResult: ImportResult | null;
    setLastImportResult: (result: ImportResult | null) => void;
    viewerSidebarOpen: boolean;
    setViewerSidebarOpen: (open: boolean) => void;
    toggleViewerSidebar: () => void;
    isLibraryReady: boolean;
    setLibraryReady: (ready: boolean) => void;
}

export const useInterfaceStore = create<InterfaceState>((set) => ({
    isSettingsDialogOpen: false,
    setSettingsDialogOpen: (open) => set({ isSettingsDialogOpen: open }),
    activeTab: DEFAULT_ACTIVE_TAB,
    setActiveTab: (tab) => {
        if (isActiveTab(tab)) set({ activeTab: tab });
    },
    selectedImportFolder: null,
    setSelectedImportFolder: (folder) => set({ selectedImportFolder: folder }),
    lastImportResult: null,
    setLastImportResult: (result) => set({ lastImportResult: result }),
    viewerSidebarOpen: false,
    setViewerSidebarOpen: (open) => set({ viewerSidebarOpen: open }),
    toggleViewerSidebar: () => set((s) => ({ viewerSidebarOpen: !s.viewerSidebarOpen })),
    isLibraryReady: false,
    setLibraryReady: (ready) => set({ isLibraryReady: ready }),
}));
