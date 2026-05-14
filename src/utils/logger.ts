// Bypassing the API layer is intentional here:
// this module is the single allowed home for direct `invoke` usage so a logging failure can't recurse through `_invoke` (which itself logs via this module).
import { invoke } from "@tauri-apps/api/core";

import { formatError } from "@/utils/format";

export type LogLevel = "error" | "warn" | "info" | "debug";

type LogContext = Record<string, unknown>;

function forwardToRust(
    level: LogLevel,
    scope: string,
    message: string,
    context: LogContext | undefined,
): void {
    void invoke("log_event", {
        level,
        scope,
        message,
        context: context ? JSON.stringify(context) : undefined,
    }).catch(() => {
        // Never recurse on a logging failure; the local console call above is enough.
    });
}

/**
 * The logger is the project's single source of truth for diagnostic output.
 * Every call mirrors to `console.*` (gated to dev for the `info` and `debug` levels) and fire-and-forgets the same payload to the Rust `log_event` command 
 * so the whole event stream lands in the same `tracing` subscriber as backend errors.
 *
 * `error` accepts any thrown value, normalizes via `formatError`, and includes the stack trace in the forwarded payload when present.
 * The `_invoke` chokepoint is the canonical caller for invoke rejections;
 * sweep policies (mutation -> notify, read-with-UI -> drop, etc.) live alongside individual call sites in feature code.
 */
export const logger = {
    error(scope: string, err: unknown, context?: LogContext): void {
        const message = formatError(err);
        const stack = err instanceof Error ? err.stack : undefined;
        const payload: LogContext | undefined =
            context !== undefined || stack !== undefined
                ? { ...(context ?? {}), ...(stack !== undefined ? { stack } : {}) }
                : undefined;

        console.error(`[${scope}] ${message}`, payload ?? "");
        forwardToRust("error", scope, message, payload);
    },

    warn(scope: string, message: string, context?: LogContext): void {
        console.warn(`[${scope}] ${message}`, context ?? "");
        forwardToRust("warn", scope, message, context);
    },

    info(scope: string, message: string, context?: LogContext): void {
        if (import.meta.env.DEV) {
            console.info(`[${scope}] ${message}`, context ?? "");
        }
        forwardToRust("info", scope, message, context);
    },

    debug(scope: string, message: string, context?: LogContext): void {
        if (import.meta.env.DEV) {
            console.debug(`[${scope}] ${message}`, context ?? "");
        }
        forwardToRust("debug", scope, message, context);
    },
};
