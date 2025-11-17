// src/stores/configStore.ts
import { create } from "zustand";
import { getAppConfig, chooseLibraryRoot } from "../api/configApi";
import { AppConfig } from "@/types/configTypes.ts";

type ConfigState = {
    config: AppConfig | null;
    isLoading: boolean;
    error: string | null;

    initConfig: () => Promise<void>;
    pickLibraryRoot: () => Promise<void>;
};

export const useConfigStore = create<ConfigState>((set, get) => ({
    config: null,
    isLoading: false,
    error: null,

    async initConfig() {
        if (get().config || get().isLoading) return;

        set({ isLoading: true, error: null });
        try {
            const cfg = await getAppConfig();
            console.log("Loaded config:", cfg);
            set({ config: cfg, isLoading: false });
        } catch (err) {
            console.error("Failed to load config", err);
            set({
                error: err?.toString?.() ?? "Failed to load configuration.",
                isLoading: false,
            });
        }
    },

    async pickLibraryRoot() {
        set({ isLoading: true, error: null });
        try {
            const selected = await chooseLibraryRoot();
            if (selected) {
                const prev = get().config ?? { libraryRoot: null };
                set({
                    config: { ...prev, libraryRoot: selected },
                    isLoading: false,
                });
            } else {
                // user cancelled
                set({ isLoading: false });
            }
        } catch (err) {
            console.error("Failed to choose library root", err);
            set({
                error: err?.toString?.() ?? "Failed to choose library root.",
                isLoading: false,
            });
        }
    },
}));
