import { useEffect } from "react";

interface UseViewerKeyboardArgs {
    canGoPrev: boolean;
    canGoNext: boolean;
    isImportMode: boolean;
    /** When true, all shortcuts are suppressed (e.g. while renaming). */
    renameActive: boolean;
    onPrev: () => void;
    onNext: () => void;
    onClose: () => void;
    onToggleSidebar: () => void;
    onLoveToggle: () => void;
    onRate: (value: 1 | 2 | 3) => void;
    onMarkReviewed: () => void;
    onMouseMove: () => void;
}

const RATING_KEYS = new Set(["1", "2", "3"]);

/**
 * Wires the viewer's keyboard shortcuts and mousemove-driven header reveal.
 * All callbacks are stable references when memoized at the call site.
 */
export function useViewerKeyboard({
    canGoPrev,
    canGoNext,
    isImportMode,
    renameActive,
    onPrev,
    onNext,
    onClose,
    onToggleSidebar,
    onLoveToggle,
    onRate,
    onMarkReviewed,
    onMouseMove,
}: UseViewerKeyboardArgs): void {
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (renameActive) return;
            const target = e.target as HTMLElement;
            if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") return;

            switch (e.key) {
                case "ArrowLeft":
                    if (!canGoPrev) return;
                    e.preventDefault();
                    onPrev();
                    return;
                case "ArrowRight":
                    if (!canGoNext) return;
                    e.preventDefault();
                    onNext();
                    return;
                case "Escape":
                    e.preventDefault();
                    onClose();
                    return;
                case "\\":
                    e.preventDefault();
                    onToggleSidebar();
                    return;
                case "f":
                case "F":
                    e.preventDefault();
                    onLoveToggle();
                    return;
                case "r":
                case "R":
                    if (!isImportMode) return;
                    e.preventDefault();
                    onMarkReviewed();
                    return;
            }

            if (RATING_KEYS.has(e.key)) {
                e.preventDefault();
                onRate(parseInt(e.key, 10) as 1 | 2 | 3);
            }
        };

        window.addEventListener("keydown", handleKeyDown);
        window.addEventListener("mousemove", onMouseMove);
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
            window.removeEventListener("mousemove", onMouseMove);
        };
    }, [
        canGoPrev, canGoNext, isImportMode, renameActive,
        onPrev, onNext, onClose, onToggleSidebar, onLoveToggle, onRate,
        onMarkReviewed, onMouseMove,
    ]);
}
