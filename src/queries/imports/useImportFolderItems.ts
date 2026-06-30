import { queryOptions, useQuery } from "@tanstack/react-query";
import { get_items_in_import_folder } from "@/api/importApi";
import { type MediaItem } from "@/types/mediaTypes";
import { queryKeys } from "@/queries/keys";

interface UseImportFolderItemsResult {
    items: MediaItem[];
    isLoading: boolean;
}

/**
 * Options factory for media items inside a single import folder.
 */
export const importFolderItemsQuery = (folder: string) =>
    queryOptions({
        queryKey: queryKeys.imports.folderItems(folder),
        queryFn: () => get_items_in_import_folder(folder),
    });

/**
 * Media items inside a single import folder. Pass `null` to clear (no fetch).
 */
export function useImportFolderItems(folderName: string | null): UseImportFolderItemsResult {
    const query = useQuery({
        ...importFolderItemsQuery(folderName ?? ""),
        enabled: folderName != null,
    });

    return {
        items: query.data ?? [],
        // Treat the "no folder selected" state as not-loading so the empty placeholder shows instead of a spinner.
        isLoading: folderName != null && query.isPending,
    };
}
