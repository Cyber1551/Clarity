import { invoke as rawInvoke } from "@tauri-apps/api/core";

import { logger } from "@/utils/logger";

import { parseTauriError } from "./errors";

/**
 * Sole chokepoint for `invoke` calls in the application. Wraps every Tauri
 * command so that:
 *
 * 1. Backend rejections are parsed into a typed `TauriInvokeError` (see`src/api/errors.ts`)
 * 2. Every rejection is logged exactly once at the API boundary with a `scope: "invoke:<command>"` tag, matching the scoping convention used by `src/utils/logger.ts`.
 *    Downstream `catch` blocks therefore decide only whether to surface the error to the user (notify/inline state) and never re-log.
 * 3. The original error is re-thrown, so the async control flow is preserved.
 */
export async function _invoke<T>(
    command: string,
    args?: Record<string, unknown>,
): Promise<T> {
    try {
        return await rawInvoke<T>(command, args);
    } catch (raw) {
        const err = parseTauriError(raw, command);
        logger.error(`invoke:${command}`, err, args !== undefined ? { args } : undefined);
        throw err;
    }
}
