import { createToaster } from "@chakra-ui/react";

import { isTauriInvokeError } from "@/api/errors";
import { formatError } from "@/utils/format";

/**
 * Module-level toaster instance. The matching `<Toaster>` JSX lives in `src/components/ui/provider.tsx`;
 * everything else writes through the helpers below so the dedupe/coercion logic isn't bypassed.
 */
export const toaster = createToaster({
    placement: "bottom-end",
    overlap: true,
    max: 4,
});

const DEDUPE_WINDOW_MS = 2000;
const lastSeen = new Map<string, number>();

function shouldEmit(key: string): boolean {
    const now = Date.now();
    const prev = lastSeen.get(key);
    if (prev !== undefined && now - prev < DEDUPE_WINDOW_MS) {
        return false;
    }
    lastSeen.set(key, now);
    return true;
}

interface ToastPayload {
    title: string;
    description?: string;
}

function buildPayload(title: string, description: string | undefined): ToastPayload {
    return description !== undefined ? { title, description } : { title };
}

function describeError(err: unknown): string {
    if (isTauriInvokeError(err)) {
        return `[${err.code}] ${err.message}`;
    }
    return formatError(err);
}

/**
 * Thin coercion layer over the Chakra toaster. Centralizing here keeps three invariants in one place: 
 * 1. Every error toast accepts arbitrary thrown values.
 * 2. `TauriInvokeError` render with their `[CODE]` prefix so the user can quote it in a bug report.
 * 3. Duplicate `(title, message)` pairs inside a 2-second window collapse to a single toast.
 */
export const notify = {
    error(title: string, errOrDetail?: unknown): void {
        const description =
            errOrDetail !== undefined ? describeError(errOrDetail) : undefined;
        const key = `error:${title}:${description ?? ""}`;
        if (!shouldEmit(key)) return;
        toaster.error(buildPayload(title, description));
    },

    success(title: string, detail?: string): void {
        const key = `success:${title}:${detail ?? ""}`;
        if (!shouldEmit(key)) return;
        toaster.success(buildPayload(title, detail));
    },

    info(title: string, detail?: string): void {
        const key = `info:${title}:${detail ?? ""}`;
        if (!shouldEmit(key)) return;
        toaster.info(buildPayload(title, detail));
    },
};
