import { useCallback, useEffect, useState } from "react";
import { get_import_folders } from "@/api/importApi";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { formatError } from "@/utils/format";

interface UseImportFoldersResult {
    folders: string[];
    isLoading: boolean;
    error: string | null;
    reload: () => Promise<void>;
}

/**
 * Tracks the list of import session folders and re-fetches whenever the
 * library changes (new import, deletion, etc.).
 */
export function useImportFolders(): UseImportFoldersResult {
    const [folders, setFolders] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const reload = useCallback(async () => {
        setIsLoading(true);
        setError(null);
        try {
            const next = await get_import_folders();
            setFolders(next);
        } catch (e: unknown) {
            setError(formatError(e));
        } finally {
            setIsLoading(false);
        }
    }, []);

    useEffect(() => {
        void reload();
    }, [reload]);

    useTauriEvent("library-changed", () => {
        void reload();
    });

    return { folders, isLoading, error, reload };
}
