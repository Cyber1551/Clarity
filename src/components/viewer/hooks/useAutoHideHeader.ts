import { useCallback, useEffect, useRef, useState } from "react";
import { AUTO_HIDE_DELAY } from "../constants";

interface UseAutoHideHeaderArgs {
    /** Sidebar pinned-open state: pauses auto-hide. */
    sidebarOpen: boolean;
    /** Rename in progress: pauses auto-hide. */
    renameActive: boolean;
}

interface UseAutoHideHeaderResult {
    visible: boolean;
    /** Mark interaction; resets the auto-hide countdown. */
    reset: () => void;
}

/**
 * Tracks whether the viewer's floating header should be visible. The header
 * is shown on mouse movement / interaction and auto-hides after
 * `AUTO_HIDE_DELAY` ms of inactivity, unless the sidebar is pinned open or
 * a rename is in progress.
 */
export function useAutoHideHeader({
    sidebarOpen,
    renameActive,
}: UseAutoHideHeaderArgs): UseAutoHideHeaderResult {
    const [hidden, setHidden] = useState(false);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const isPinned = sidebarOpen || renameActive;

    const clearTimer = useCallback(() => {
        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
    }, []);

    const reset = useCallback(() => {
        setHidden(false);
        clearTimer();
        if (!isPinned) {
            timerRef.current = setTimeout(() => setHidden(true), AUTO_HIDE_DELAY);
        }
    }, [isPinned, clearTimer]);

    // (Re)schedule the auto-hide timer whenever the pinned state changes.
    useEffect(() => {
        if (isPinned) {
            clearTimer();
            return;
        }
        timerRef.current = setTimeout(() => setHidden(true), AUTO_HIDE_DELAY);
        return clearTimer;
    }, [isPinned, clearTimer]);

    // While pinned the header is always visible regardless of the timer.
    const visible = isPinned ? true : !hidden;

    return { visible, reset };
}
