/**
 * * Single source of truth for TanStack Query keys.
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
    tags: {
        all: () => ["tags"] as const,
        list: () => [...queryKeys.tags.all(), "list"] as const,
        forMedia: (mediaId: number) =>
            [...queryKeys.tags.all(), "media", mediaId] as const,
    },
    sync: {
        all: () => ["sync"] as const,
        status: () => [...queryKeys.sync.all(), "status"] as const,
    },
    thumbnail: (hash: string) => ["thumbnail", hash] as const,
} as const;
