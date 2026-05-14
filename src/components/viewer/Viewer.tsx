import { useCallback, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Box, Dialog, Portal } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { ViewerHeader } from "./header/ViewerHeader";
import { ViewerStage } from "./stage/ViewerStage";
import { ViewerSidebar } from "./sidebar/ViewerSidebar";
import { useMediaDetail } from "./hooks/useMediaDetail";
import { useAutoHideHeader } from "./hooks/useAutoHideHeader";
import { useViewerKeyboard } from "./hooks/useViewerKeyboard";
import { useViewerMutations } from "./hooks/useViewerMutations";
import { NAV_TRANSITION_MS } from "./constants";

export function Viewer() {
    const viewer = useMediaStore((s) => s.viewer);
    const closeViewer = useMediaStore((s) => s.closeViewer);
    const navigateViewer = useMediaStore((s) => s.navigateViewer);
    const isOpen = Boolean(viewer);

    const sidebarOpen = useInterfaceStore((s) => s.viewerSidebarOpen);
    const toggleSidebar = useInterfaceStore((s) => s.toggleViewerSidebar);

    const [transitioning, setTransitioning] = useState(false);
    const [renameActive, setRenameActive] = useState(false);

    const mediaId = viewer?.mediaId ?? null;
    const items = viewer?.items ?? [];
    const currentIndex = viewer
        ? viewer.items.findIndex((i) => i.mediaId === viewer.mediaId)
        : -1;
    const totalCount = viewer?.items.length ?? 0;
    const canGoPrev = currentIndex > 0;
    const canGoNext = viewer ? currentIndex < viewer.items.length - 1 : false;

    const { detail, loading, error, reload, invalidate } = useMediaDetail({
        mediaId,
        items,
    });

    const handleMutated = useCallback(() => {
        if (mediaId == null) return;
        invalidate(mediaId);
        reload();
    }, [mediaId, invalidate, reload]);

    const mutations = useViewerMutations({
        mediaId,
        onMutated: handleMutated,
    });

    const { visible: headerVisible, reset: resetHideTimer } = useAutoHideHeader({
        isOpen,
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
        enabled: isOpen,
        canGoPrev,
        canGoNext,
        isImportMode: viewer?.mode === "import",
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
        <Dialog.Root open={isOpen} onOpenChange={(e) => !e.open && closeViewer()} size="full">
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
                                mode={viewer?.mode ?? "library"}
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
                                mode={viewer?.mode ?? "library"}
                                onDetailChanged={handleMutated}
                            />
                        </Box>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}
