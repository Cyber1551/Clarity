export type SearchableType = "image" | "video";

/** Parsed palette state: free text plus structured filters. */
export interface SearchFilters {
    text: string;
    type: SearchableType | null;
    favorite: number | null;
    quality: number | null;
    loved: boolean | null;
    tags: string[];
}

/** Wire format for the `search_media` command (mirrors the Rust `SearchQuery` DTO). */
export interface SearchQuery {
    text?: string;
    mediaType?: SearchableType;
    quality?: number;
    favorite?: number;
    loved?: boolean;
    tags?: string[];
    reviewed?: boolean;
    limit?: number;
    offset?: number;
}

export const EMPTY_FILTERS: SearchFilters = {
    text: "",
    type: null,
    favorite: null,
    quality: null,
    loved: null,
    tags: [],
};
