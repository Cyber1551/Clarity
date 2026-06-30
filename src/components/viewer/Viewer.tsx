import { useCallback, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Box, Dialog, Portal } from "@chakra-ui/react";
import { useViewerStore, type ViewerState } from "@/stores/viewerStore";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { ViewerHeader } from "./header/ViewerHeader";
import { ViewerStage } from "./stage/ViewerStage";
import { ViewerSidebar } from "./sidebar/ViewerSidebar";
import { useMediaDetail } from "@/queries/library/useMediaDetail";
import { useAutoHideHeader } from "./hooks/useAutoHideHeader";
import { useViewerKeyboard } from "./hooks/useViewerKeyboard";
import { useViewerMutations } from "./hooks/useViewerMutations";
import { NAV_TRANSITION_MS } from "./constants";

/**
 * Thin gate that returns null when no media is open. All hooks (data fetch,
 * keyboard listeners, auto-hide timer, mutations) live in {@link ViewerImpl}
 * so they only subscribe while the viewer is actually open. Render errors in
 * the viewer subtree bubble to the single root boundary in main.tsx.
 */
export function Viewer() {
    const viewer = useViewerStore((s) => s.viewer);
    if (!viewer) return null;
    return <ViewerImpl viewer={viewer} />;
}

function ViewerImpl({ viewer }: { viewer: NonNullable<ViewerState> }) {
    const closeViewer = useViewerStore((s) => s.closeViewer);
    const navigateViewer = useViewerStore((s) => s.navigateViewer);

    const sidebarOpen = useInterfaceStore((s) => s.viewerSidebarOpen);
    const toggleSidebar = useInterfaceStore((s) => s.toggleViewerSidebar);

    const [transitioning, setTransitioning] = useState(false);
    const [renameActive, setRenameActive] = useState(false);

    const mediaId = viewer.mediaId;
    const items = viewer.items;
    const currentIndex = items.findIndex((i) => i.mediaId === mediaId);
    const totalCount = items.length;
    const canGoPrev = currentIndex > 0;
    const canGoNext = currentIndex < items.length - 1;

    const { detail, loading, error, reload } = useMediaDetail({
        mediaId,
        items,
    });

    const mutations = useViewerMutations({ mediaId });

    const { visible: headerVisible, reset: resetHideTimer } = useAutoHideHeader({
        sidebarOpen,
        renameActive,
    });

    const handleNavigate = useCallback((dir: "prev" | "next") => {
        setTransitioning(true);
        setTimeout(() => {
            navigateViewer(dir);
            setTransitioning(false);
        }, NAV_TRANSITION_MS);
        resetHideTimer();
    }, [navigateViewer, resetHideTimer]);

    const handleToggleSidebar = useCallback(() => {
        toggleSidebar();
        resetHideTimer();
    }, [toggleSidebar, resetHideTimer]);

    const handleRate = useCallback((value: 1 | 2 | 3) => {
        if (!detail) return;
        const next = detail.qualityRating === value ? 0 : value;
        void mutations.setQuality(next);
    }, [detail, mutations]);

    useViewerKeyboard({
        canGoPrev,
        canGoNext,
        isImportMode: viewer.mode === "import",
        renameActive,
        onPrev: () => handleNavigate("prev"),
        onNext: () => handleNavigate("next"),
        onClose: closeViewer,
        onToggleSidebar: handleToggleSidebar,
        onLoveToggle: () => void mutations.toggleLoved(),
        onRate: handleRate,
        onMarkReviewed: () => void mutations.markReviewed(),
        onMouseMove: resetHideTimer,
    });

    const mediaSrc = detail ? convertFileSrc(detail.canonicalPath) : "";
    const isReviewed = detail?.reviewedAt != null;

    return (
        <Dialog.Root open onOpenChange={(e) => !e.open && closeViewer()} size="full">
            <Portal>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content
                        bg="black"
                        color="white"
                        h="100vh"
                        w="100vw"
                        m={0}
                        p={0}
                        border="none"
                        borderRadius={0}
                    >
                        <Box display="flex" h="full" w="full" position="relative">
                            <ViewerHeader
                                detail={detail}
                                mode={viewer.mode}
                                currentIndex={currentIndex}
                                totalCount={totalCount}
                                visible={headerVisible}
                                sidebarOpen={sidebarOpen}
                                isReviewed={isReviewed}
                                onClose={closeViewer}
                                onToggleSidebar={handleToggleSidebar}
                                onMarkReviewed={() => void mutations.markReviewed()}
                                onRename={mutations.rename}
                                onLoveToggle={() => void mutations.toggleLoved()}
                                onRenameActiveChange={setRenameActive}
                            />

                            <Box flex={1} display="flex" flexDirection="column" minW={0} transition="all 0.25s ease">
                                <ViewerStage
                                    detail={detail}
                                    src={mediaSrc}
                                    loading={loading}
                                    error={error}
                                    canGoPrev={canGoPrev}
                                    canGoNext={canGoNext}
                                    onPrev={() => handleNavigate("prev")}
                                    onNext={() => handleNavigate("next")}
                                    onRetry={reload}
                                    transitioning={transitioning}
                                />
                            </Box>

                            <ViewerSidebar
                                open={sidebarOpen}
                                detail={detail}
                                mode={viewer.mode}
                            />
                        </Box>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}
