import { Box, Tabs } from "@chakra-ui/react";
import { useEffect } from "react";
import { useMediaStore } from "@/stores/mediaStore";
import MediaGrid from "./MediaGrid";
import { listen } from "@tauri-apps/api/event";

const MainContent = () => {
    const items = useMediaStore((s) => s.items);
    const isLoading = useMediaStore((s) => s.isLoading);
    const error = useMediaStore((s) => s.error);
    const loadAllMedia = useMediaStore((s) => s.loadAllMedia);

    // Listen for job completion and library change events
    useEffect(() => {
        const unsubscribeJobCompleted = listen("job-completed", () => {
            void loadAllMedia();
        });

        const unsubscribeLibraryChanged = listen("library-changed", () => {
            void loadAllMedia();
        });

        return () => {
            unsubscribeJobCompleted.then(fn => fn());
            unsubscribeLibraryChanged.then(fn => fn());
        };
    }, [loadAllMedia]);

    return (
        <Box as={"main"} flex="1" minH={0} overflowY="auto">
            <Tabs.Content value="dashboard">
                Dashboard
            </Tabs.Content>
            <Tabs.Content value="library">
                <MediaGrid items={items} isLoading={isLoading} error={error} />
            </Tabs.Content>
            <Tabs.Content value="explore">
                Explore
            </Tabs.Content>
            <Tabs.Content value="moments">
                Moments
            </Tabs.Content>
            <Tabs.Content value="session">
                Sessions
            </Tabs.Content>
        </Box>
    );
};

export default MainContent;
