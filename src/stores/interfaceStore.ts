import { create } from "zustand";


type interfaceState = {
    isSettingsDialogOpen: boolean;
    setSettingsDialogOpen: (open: boolean) => void;
}

export const useInterfaceStore = create<interfaceState>((set) => ({
    isSettingsDialogOpen: false,
    setSettingsDialogOpen: (open) => set({isSettingsDialogOpen: open})
}));