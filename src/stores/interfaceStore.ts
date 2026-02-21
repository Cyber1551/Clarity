import { create } from "zustand";
import { ImportResult } from "@/types/importTypes";


type interfaceState = {
    isSettingsDialogOpen: boolean;
    setSettingsDialogOpen: (open: boolean) => void;
    activeTab: string;
    setActiveTab: (tab: string) => void;
    selectedImportFolder: string | null;
    setSelectedImportFolder: (folder: string | null) => void;
    lastImportResult: ImportResult | null;
    setLastImportResult: (result: ImportResult | null) => void;
}

export const useInterfaceStore = create<interfaceState>((set) => ({
    isSettingsDialogOpen: false,
    setSettingsDialogOpen: (open) => set({isSettingsDialogOpen: open}),
    activeTab: "library",
    setActiveTab: (tab) => set({activeTab: tab}),
    selectedImportFolder: null,
    setSelectedImportFolder: (folder) => set({selectedImportFolder: folder}),
    lastImportResult: null,
    setLastImportResult: (result) => set({lastImportResult: result})
}));
