import { create } from "zustand";
import { getAppConfig, chooseLibraryRoot } from "@/api/configApi";
import { type AppConfig } from "@/types/configTypes.ts";
import { formatError } from "@/utils/format";
import { notify } from "@/utils/notify";

interface ConfigState {
    config: AppConfig | null;
    isLoading: boolean;
    error: string | null;

    initConfig: () => Promise<void>;
    pickLibraryRoot: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
    config: null,
    isLoading: false,
    error: null,

    async initConfig() {
        if (get().config || get().isLoading) return;

        set({ isLoading: true, error: null });
        try {
            const cfg = await getAppConfig();
            // `_invoke` already logged any failure path; the inline `error` state surfaces it in the picker view.
            set({ config: cfg, isLoading: false });
        } catch (err) {
            set({
                error: formatError(err, "Failed to load configuration."),
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
            notify.error("Couldn't choose library root", err);
            set({
                error: formatError(err, "Failed to choose library root."),
                isLoading: false,
            });
        }
    },
}));
