import { queryOptions, useQuery } from "@tanstack/react-query";
import { get_import_folders } from "@/api/importApi";
import { formatError } from "@/utils/format";
import { queryKeys } from "@/queries/keys";

interface UseImportFoldersResult {
    folders: string[];
    isLoading: boolean;
    error: string | null;
}

/**
 * Options factory for the list of import session folders.
 */
export const importFoldersQuery = () =>
    queryOptions({
        queryKey: queryKeys.imports.folders(),
        queryFn: () => get_import_folders(),
    });

/**
 * Invalidated by the `library-changed` event via the central invalidation bridge.
 */
export function useImportFolders(): UseImportFoldersResult {
    const query = useQuery(importFoldersQuery());

    return {
        folders: query.data ?? [],
        isLoading: query.isPending,
        error: query.error ? formatError(query.error) : null,
    };
}
