import { Box, Tabs } from "@chakra-ui/react";
import { useEffect, useMemo } from "react";
import { useMediaStore } from "@/stores/mediaStore";
import MediaGrid from "./MediaGrid";
import ImportsView from "./ImportsView";
import { listen } from "@tauri-apps/api/event";
import { throttle } from "lodash-es";

const MainContent = () => {
    const items = useMediaStore((s) => s.items);
    const isLoading = useMediaStore((s) => s.isLoading);
    const error = useMediaStore((s) => s.error);
    const loadAllMedia = useMediaStore((s) => s.loadAllMedia);

    // Throttle the loadAllMedia calls to once every 2 seconds to avoid UI freezing
    // during heavy background processing.
    const throttledLoad = useMemo(
        () => throttle(loadAllMedia, 2000, { leading: true, trailing: true }),
        [loadAllMedia]
    );

    // Listen for job completion and library change events
    useEffect(() => {
        const unsubscribeJobCompleted = listen("job-completed", () => {
            void throttledLoad();
        });

        const unsubscribeLibraryChanged = listen("library-changed", () => {
            void loadAllMedia();
        });

        return () => {
            unsubscribeJobCompleted.then(fn => fn());
            unsubscribeLibraryChanged.then(fn => fn());
            throttledLoad.cancel();
        };
    }, [loadAllMedia, throttledLoad]);

    return (
        <Box as={"main"} flex="1" minH={0} display="flex" flexDirection="column">
            <Tabs.Content value="dashboard" flex="1" overflowY="auto">
                Dashboard
            </Tabs.Content>
            <Tabs.Content value="imports" flex="1" minH={0} display="flex" flexDirection="column">
                <ImportsView />
            </Tabs.Content>
            <Tabs.Content value="library" flex="1" minH={0} display="flex" flexDirection="column">
                <MediaGrid items={items} isLoading={isLoading} error={error} />
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
