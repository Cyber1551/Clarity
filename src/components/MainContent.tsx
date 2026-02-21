import { Box, Tabs } from "@chakra-ui/react";
import { useEffect } from "react";
import { useMediaStore } from "@/stores/mediaStore";
import MediaGrid from "./MediaGrid";
import ImportsView from "./ImportsView";
import { listen } from "@tauri-apps/api/event";
import { useInterfaceStore } from "@/stores/interfaceStore";

type JobCompletedPayload = {
    jobType: string;
    mediaId: number | null;
    fileId: number | null;
    relPath: string | null;
    status: string;
};

const MainContent = () => {
    const items = useMediaStore((s) => s.items);
    const isLoading = useMediaStore((s) => s.isLoading);
    const error = useMediaStore((s) => s.error);
    const loadAllMedia = useMediaStore((s) => s.loadAllMedia);
    const refreshItemByRelPath = useMediaStore((s) => s.refreshItemByRelPath);
    const scrollTargetMediaId = useMediaStore((s) => s.scrollTargetMediaId);
    const setScrollTargetMediaId = useMediaStore((s) => s.setScrollTargetMediaId);
    const clearHighlight = useMediaStore((s) => s.clearHighlight);
    const activeTab = useInterfaceStore(s => s.activeTab);

    // Listen for job completion and library change events
    useEffect(() => {
        const unsubscribeJobCompleted = listen<JobCompletedPayload>("job-completed", (event) => {
            if (activeTab !== "library") return;
            const relPath = event.payload?.relPath;
            if (!relPath || !relPath.startsWith("Library/")) return;
            void refreshItemByRelPath(relPath);
        });

        const unsubscribeLibraryChanged = listen("library-changed", () => {
            void loadAllMedia();
        });

        return () => {
            unsubscribeJobCompleted.then(fn => fn());
            unsubscribeLibraryChanged.then(fn => fn());
        };
    }, [activeTab, loadAllMedia, refreshItemByRelPath]);

    return (
        <Box as={"main"} flex="1" minH={0} display="flex" flexDirection="column">
            <Tabs.Content value="dashboard" flex="1" overflowY="auto">
                Dashboard
            </Tabs.Content>
            <Tabs.Content value="imports" flex="1" minH={0} display="flex" flexDirection="column">
                <ImportsView />
            </Tabs.Content>
            <Tabs.Content value="library" flex="1" minH={0} display="flex" flexDirection="column" onClick={clearHighlight}>
                <MediaGrid
                    items={items}
                    isLoading={isLoading}
                    error={error}
                    scrollToMediaId={scrollTargetMediaId}
                    onScrolledToMediaId={() => setScrollTargetMediaId(null)}
                />
            </Tabs.Content>
            <Tabs.Content value="people" flex="1" overflowY="auto">
                People
            </Tabs.Content>
            <Tabs.Content value="explore" flex="1" overflowY="auto">
                Explore
            </Tabs.Content>
            <Tabs.Content value="moments" flex="1" overflowY="auto">
                Moments
            </Tabs.Content>
            <Tabs.Content value="session" flex="1" overflowY="auto">
                Sessions
            </Tabs.Content>
        </Box>
    );
};

export default MainContent;
