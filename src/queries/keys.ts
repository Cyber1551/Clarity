/**
 * Single source of truth for TanStack Query keys.
 *
 * Keys are hierarchical so each domain has an `all()` accessor that matches every key inside it as a prefix.
 * That means `invalidateQueries({ queryKey: queryKeys.library.all() })` invalidates `library.items()`, `library.detail(id)`, and any future `library.*` entries in one call
 */
export const queryKeys = {
    library: {
        all: () => ["library"] as const,
        items: () => [...queryKeys.library.all(), "items"] as const,
        detail: (mediaId: number) =>
            [...queryKeys.library.all(), "detail", mediaId] as const,
    },
    imports: {
        all: () => ["imports"] as const,
        folders: () => [...queryKeys.imports.all(), "folders"] as const,
        folderItems: (folder: string) =>
            [...queryKeys.imports.all(), "items", folder] as const,
    },
    thumbnail: (hash: string) => ["thumbnail", hash] as const,
} as const;
