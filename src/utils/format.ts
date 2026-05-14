import { DurationFormat } from '@formatjs/intl-durationformat'

const relativeTimeFormat = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

const RELATIVE_TIME_UNITS: Array<{ unit: Intl.RelativeTimeFormatUnit; secondsPer: number }> = [
    { unit: "year", secondsPer: 60 * 60 * 24 * 365 },
    { unit: "month", secondsPer: 60 * 60 * 24 * 30 },
    { unit: "day", secondsPer: 60 * 60 * 24 },
    { unit: "hour", secondsPer: 60 * 60 },
    { unit: "minute", secondsPer: 60 },
    { unit: "second", secondsPer: 1 },
];

/**
 * Format a past timestamp (epoch ms) as a relative phrase like "5 minutes ago".
 * Picks the largest applicable unit automatically.
 */
export function formatRelativeTime(epochMs: number): string {
    const diffSec = Math.round((epochMs - Date.now()) / 1000);
    const abs = Math.abs(diffSec);

    for (const { unit, secondsPer } of RELATIVE_TIME_UNITS) {
        if (abs >= secondsPer || unit === "second") {
            const value = Math.round(diffSec / secondsPer);
            return relativeTimeFormat.format(value, unit);
        }
    }
    return relativeTimeFormat.format(diffSec, "second");
}

/**
 * Format a duration in milliseconds (e.g. video length) as "1h 23m 4s".
 */
export function formatDuration(
    ms: number,
    style: "long" | "short" | "narrow" | "digital",
): string {
    const totalSeconds = Math.max(0, Math.floor(ms / 1000));
    const hours = Math.floor(totalSeconds / 3600);
    const minutes = Math.floor((totalSeconds % 3600) / 60);
    const seconds = totalSeconds % 60;

    const parts = {
        hours: hours > 0 ? hours : 0,
        minutes: minutes > 0 || hours > 0 ? minutes : 0,
        seconds: seconds
    };

    return new DurationFormat("en", { style }).format(parts);
}

/**
 * Coerce an unknown thrown value into a human-readable string.
 */
export function formatError(err: unknown, fallback = "An unknown error occurred"): string {
    if (err == null) return fallback;
    if (err instanceof Error) return err.message;
    if (typeof err === "string") return err;
    try {
        const str = String(err);
        return str === "[object Object]" ? fallback : str;
    } catch {
        return fallback;
    }
}
