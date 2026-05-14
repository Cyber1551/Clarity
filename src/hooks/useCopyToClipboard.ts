import { useCallback, useEffect, useRef, useState } from "react";
import { logger } from "@/utils/logger";
import { notify } from "@/utils/notify";

interface UseCopyToClipboardOptions {
    /** How long the `copied` flag stays true after a successful copy (ms). */
    resetMs?: number;
}

/**
 * Wrapper around `navigator.clipboard.writeText` that exposes a transient "just copied" flag for confirming UI feedback.
 *
 * Returns `[copy, copied]`. The `copied` flag flips back to `false` after `resetMs` (default 2000ms) or unmount, whichever comes first.
 */
export function useCopyToClipboard(
    { resetMs = 2000 }: UseCopyToClipboardOptions = {},
): readonly [(text: string) => Promise<void>, boolean] {
    const [copied, setCopied] = useState(false);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    useEffect(() => () => {
        if (timerRef.current) clearTimeout(timerRef.current);
    }, []);

    const copy = useCallback(async (text: string) => {
        try {
            await navigator.clipboard.writeText(text);
            setCopied(true);
            if (timerRef.current) clearTimeout(timerRef.current);
            timerRef.current = setTimeout(() => setCopied(false), resetMs);
        } catch (err) {
            // Not a Tauri command, so `_invoke` won't have logged it.
            // Do both here: log for diagnostics and surface so the user knows the user-initiated copy didn't go through.
            logger.error("clipboard", err);
            notify.error("Couldn't copy to clipboard", err);
        }
    }, [resetMs]);

    return [copy, copied] as const;
}
