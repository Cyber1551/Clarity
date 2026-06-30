import { type Tag } from "@/types/mediaTypes";
import {
    EMPTY_FILTERS,
    type SearchableType,
    type SearchFilters,
    type SearchQuery,
} from "@/types/searchTypes";

const FILTER_KEYS = ["tag", "type", "favorite", "quality", "loved"] as const;
type FilterKey = (typeof FILTER_KEYS)[number];

const RATING_VALUES = ["1", "2", "3", "4", "5"] as const;
const TYPE_VALUES = ["photo", "video"] as const;
const LOVED_VALUES = ["true", "false"] as const;

function isFilterKey(value: string): value is FilterKey {
    return (FILTER_KEYS as readonly string[]).includes(value);
}

/** Lowercase, hyphenated slug. Mirrors the Rust `slugify` so typed tags resolve to stored slugs. */
export function slugify(name: string): string {
    const slug = name
        .trim()
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/^-+|-+$/g, "");
    return slug || "untitled";
}

function parseType(value: string): SearchableType | null {
    const v = value.toLowerCase();
    if (v === "photo" || v === "image") return "image";
    if (v === "video") return "video";
    return null;
}

function parseRating(value: string): number | null {
    const n = Number.parseInt(value, 10);
    if (Number.isNaN(n)) return null;
    return Math.min(Math.max(n, 0), 5);
}

function parseLoved(value: string): boolean | null {
    const v = value.toLowerCase();
    if (v === "true" || v === "1" || v === "yes") return true;
    if (v === "false" || v === "0" || v === "no") return false;
    return null;
}

/** Splits a `key:value` token; returns null for bare words. */
function splitToken(token: string): { key: FilterKey; value: string } | null {
    const idx = token.indexOf(":");
    if (idx <= 0) return null;
    const key = token.slice(0, idx).toLowerCase();
    if (!isFilterKey(key)) return null;
    return { key, value: token.slice(idx + 1) };
}

export function parse(input: string): SearchFilters {
    const filters: SearchFilters = { ...EMPTY_FILTERS, tags: [] };
    const textTerms: string[] = [];

    for (const token of input.split(/\s+/)) {
        if (!token) continue;
        const parsed = splitToken(token);
        if (!parsed) {
            textTerms.push(token);
            continue;
        }

        const { key, value } = parsed;
        if (!value) continue;

        switch (key) {
            case "tag": {
                const slug = slugify(value);
                if (slug && !filters.tags.includes(slug)) filters.tags.push(slug);
                break;
            }
            case "type":
                filters.type = parseType(value);
                break;
            case "favorite":
                filters.favorite = parseRating(value);
                break;
            case "quality":
                filters.quality = parseRating(value);
                break;
            case "loved":
                filters.loved = parseLoved(value);
                break;
        }
    }

    filters.text = textTerms.join(" ");
    return filters;
}

/** Builds the wire query, omitting absent fields (kept compatible with exactOptionalPropertyTypes). */
export function toSearchQuery(filters: SearchFilters, limit?: number): SearchQuery {
    const query: SearchQuery = {};
    const text = filters.text.trim();
    if (text) query.text = text;
    if (filters.type) query.mediaType = filters.type;
    if (filters.quality != null) query.quality = filters.quality;
    if (filters.favorite != null) query.favorite = filters.favorite;
    if (filters.loved != null) query.loved = filters.loved;
    if (filters.tags.length > 0) query.tags = filters.tags;
    if (limit != null) query.limit = limit;
    return query;
}

/** Stable key for caching: order-independent and case-normalized. */
export function serializeFilters(filters: SearchFilters): string {
    return JSON.stringify({
        text: filters.text.trim().toLowerCase(),
        type: filters.type,
        favorite: filters.favorite,
        quality: filters.quality,
        loved: filters.loved,
        tags: [...filters.tags].sort(),
    });
}

export function hasCriteria(filters: SearchFilters): boolean {
    return (
        filters.text.trim().length > 0 ||
        filters.type != null ||
        filters.favorite != null ||
        filters.quality != null ||
        filters.loved != null ||
        filters.tags.length > 0
    );
}

export interface Suggestion {
    /** Replacement text for the active token (no trailing space). */
    token: string;
    label: string;
    hint?: string;
}

function valueSuggestions(key: FilterKey, partial: string, tags: Tag[]): Suggestion[] {
    const p = partial.toLowerCase();
    switch (key) {
        case "type":
            return TYPE_VALUES.filter((v) => v.startsWith(p)).map((v) => ({
                token: `type:${v}`,
                label: `type:${v}`,
            }));
        case "favorite":
        case "quality":
            return RATING_VALUES.filter((v) => v.startsWith(p)).map((v) => ({
                token: `${key}:${v}`,
                label: `${key}:${v}`,
                hint: `${v}+`,
            }));
        case "loved":
            return LOVED_VALUES.filter((v) => v.startsWith(p)).map((v) => ({
                token: `loved:${v}`,
                label: `loved:${v}`,
            }));
        case "tag":
            return tags
                .filter((t) => t.name.toLowerCase().includes(p) || t.slug.includes(p))
                .slice(0, 6)
                .map((t) => ({ token: `tag:${t.slug}`, label: t.name, hint: "tag" }));
    }
}

/** Suggestions for the token currently being typed (the last whitespace-delimited chunk). */
export function getSuggestions(input: string, tags: Tag[]): Suggestion[] {
    const activeToken = input.split(/\s+/).pop() ?? "";
    if (!activeToken) return [];

    const parsed = splitToken(activeToken);
    if (parsed) {
        return valueSuggestions(parsed.key, parsed.value, tags);
    }

    if (activeToken.includes(":")) return [];

    const p = activeToken.toLowerCase();
    return FILTER_KEYS.filter((k) => k.startsWith(p)).map((k) => ({
        token: `${k}:`,
        label: `${k}:`,
        hint: "filter",
    }));
}

/** Replaces the active (last) token with the suggestion and adds a trailing space. */
export function applySuggestion(input: string, suggestion: Suggestion): string {
    const tokens = input.split(/(\s+)/);
    let lastTokenIdx = -1;
    for (let i = tokens.length - 1; i >= 0; i--) {
        const t = tokens[i];
        if (t != null && t.trim().length > 0) {
            lastTokenIdx = i;
            break;
        }
    }

    if (lastTokenIdx === -1) {
        return `${suggestion.token} `;
    }

    tokens[lastTokenIdx] = suggestion.token;
    const rebuilt = tokens.join("");
    const needsSpace = suggestion.token.endsWith(":") ? "" : " ";
    return `${rebuilt}${needsSpace}`;
}
