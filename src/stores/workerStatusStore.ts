import { create } from "zustand";
import { restart_workers } from "@/api/libraryApi";
import { type WorkerStalledPayload } from "@/types/eventTypes";

const RETRY_TIMEOUT_MS = 5000;

type WorkerStatusState = {
    stalled: boolean;
    lastError: string | null;
    consecutiveFailures: number;
    retrying: boolean;
    lastRetryFailed: boolean;
    /**
     * Monotonically increases each time a retry is initiated. The fallback
     * timer captures this counter at start time and only flips
     * `lastRetryFailed=true` if the value still matches when it fires
     * The prevents a stale timer from stalling a successful recovery.
     */
    retryGen: number;

    onStalled: (payload: WorkerStalledPayload) => void;
    onRecovered: () => void;
    retry: () => Promise<void>;
};

export const useWorkerStatusStore = create<WorkerStatusState>((set, get) => ({
    stalled: false,
    lastError: null,
    consecutiveFailures: 0,
    retrying: false,
    lastRetryFailed: false,
    retryGen: 0,

    onStalled(payload) {
        set({
            stalled: true,
            lastError: payload.errorMessage,
            consecutiveFailures: payload.consecutiveFailures,
        });
    },

    onRecovered() {
        set((s) => ({
            stalled: false,
            lastError: null,
            consecutiveFailures: 0,
            retrying: false,
            lastRetryFailed: false,
            retryGen: s.retryGen + 1, // invalidate any in-flight retry timer
        }));
    },

    async retry() {
        const startGen = get().retryGen + 1;
        set({ retrying: true, lastRetryFailed: false, retryGen: startGen });

        try {
            await restart_workers();
            // Wait up to RETRY_TIMEOUT_MS for `worker-recovered` to land.
            // If a different retry has started in the meantime (or the worker has already recovered)
            // bail out without flipping flags.
            window.setTimeout(() => {
                if (get().retryGen !== startGen) return;
                if (!get().stalled) return;
                set({ retrying: false, lastRetryFailed: true });
            }, RETRY_TIMEOUT_MS);
        } catch (e) {
            console.error("restart_workers failed", e);
            if (get().retryGen === startGen) {
                set({ retrying: false, lastRetryFailed: true });
            }
        }
    },
}));