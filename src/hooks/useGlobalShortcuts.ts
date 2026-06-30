import { useEffect } from "react";
import { useInterfaceStore } from "@/stores/interfaceStore";

/** App-wide keyboard shortcuts. Cmd/Ctrl+K toggles the search palette. Mount once. */
export function useGlobalShortcuts() {
    const toggleSearch = useInterfaceStore((s) => s.toggleSearch);

    useEffect(() => {
        const handler = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k") {
                e.preventDefault();
                toggleSearch();
            }
        };
        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    }, [toggleSearch]);
}
