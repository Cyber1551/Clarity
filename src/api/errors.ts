import { formatError } from "@/utils/format";

/**
 * Stable string codes mirroring the `AppError::code()` mapping in `src-tauri/src/core/error.rs`.
 * `"UNKNOWN"` is reserved for the frontend's `parseTauriError` to label rejections that don't conform to the structured `{ code, message, context? }` contract.
 */
export type TauriErrorCode =
    | "LIBRARY_ROOT_MISSING"
    | "NOT_FOUND"
    | "FILE_NOT_FOUND"
    | "DATABASE"
    | "REFINERY"
    | "IMAGE"
    | "CONFIG"
    | "IO"
    | "JSON"
    | "INVALID_FILE_NAME"
    | "TIME"
    | "JOB"
    | "HASH"
    | "DEDUP"
    | "METADATA"
    | "THUMBNAIL"
    | "SCAN"
    | "UNEXPECTED"
    | "OTHER"
    | "UNKNOWN";

export interface SerializedTauriError {
    code: TauriErrorCode;
    message: string;
    context?: string;
}

/**
 * Wraps a Tauri `invoke` rejection in a typed Error subclass so call sites can branch on `code` without parsing the message string.
 * Always thrown by `_invoke`; never constructed directly outside `parseTauriError`.
 */
export class TauriInvokeError extends Error {
    readonly code: TauriErrorCode;
    readonly context?: string;
    readonly command: string;

    constructor(serialized: SerializedTauriError, command: string) {
        super(serialized.message);
        this.name = "TauriInvokeError";
        this.code = serialized.code;
        this.command = command;
        if (serialized.context !== undefined) {
            this.context = serialized.context;
        }
    }
}

export function isTauriInvokeError(e: unknown): e is TauriInvokeError {
    return e instanceof TauriInvokeError;
}

const KNOWN_CODES: ReadonlySet<TauriErrorCode> = new Set<TauriErrorCode>([
    "LIBRARY_ROOT_MISSING",
    "NOT_FOUND",
    "FILE_NOT_FOUND",
    "DATABASE",
    "REFINERY",
    "IMAGE",
    "CONFIG",
    "IO",
    "JSON",
    "INVALID_FILE_NAME",
    "TIME",
    "JOB",
    "HASH",
    "DEDUP",
    "METADATA",
    "THUMBNAIL",
    "SCAN",
    "UNEXPECTED",
    "OTHER",
    "UNKNOWN",
]);

function isSerializedTauriError(raw: unknown): raw is SerializedTauriError {
    if (typeof raw !== "object" || raw === null) return false;
    const obj = raw as Record<string, unknown>;
    const code = obj["code"];
    const message = obj["message"];
    const context = obj["context"];
    if (typeof code !== "string" || typeof message !== "string") return false;
    if (!KNOWN_CODES.has(code as TauriErrorCode)) return false;
    if (context !== undefined && typeof context !== "string") return false;
    return true;
}

/**
 * Coerces an arbitrary `invoke` rejection into a `TauriInvokeError`.
 * Accepts the structured `{ code, message, context? }` shape produced by `AppError`'s `Serialize` impl,
 * and falls back to `code: "UNKNOWN"` for plugin-emitted strings, plain `Error` instances, or any other shape.
 */
export function parseTauriError(raw: unknown, command: string): TauriInvokeError {
    if (isSerializedTauriError(raw)) {
        return new TauriInvokeError(raw, command);
    }
    if (raw instanceof Error) {
        return new TauriInvokeError({ code: "UNKNOWN", message: raw.message }, command);
    }
    if (typeof raw === "string") {
        return new TauriInvokeError({ code: "UNKNOWN", message: raw }, command);
    }
    return new TauriInvokeError({ code: "UNKNOWN", message: formatError(raw) }, command);
}
