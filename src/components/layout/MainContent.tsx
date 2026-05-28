import React from "react";
import { Box, Tabs } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { MediaGrid } from "@/components/library";
import { ImportsView } from "@/components/imports";
import { useInterfaceStore } from "@/stores/interfaceStore";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { type JobCompletedPayload } from "@/types/eventTypes";
import { TABS, type ActiveTab } from "@/constants/tabs";

const TAB_PANELS: Record<ActiveTab, () => React.ReactNode> = {
    dashboard: () => "Dashboard",
    imports: () => <ImportsView />,
    library: () => <LibraryPanel />,
    people: () => "People",
    explore: () => "Explore",
    moments: () => "Moments",
    session: () => "Sessions",
};

function LibraryPanel() {
    const items = useMediaStore((s) => s.items);
    const isLoading = useMediaStore((s) => s.isLoading);
    const error = useMediaStore((s) => s.error);
    const scrollTargetMediaId = useMediaStore((s) => s.scrollTargetMediaId);
    const setScrollTargetMediaId = useMediaStore((s) => s.setScrollTargetMediaId);

    return (
        <MediaGrid
            items={items}
            isLoading={isLoading}
            error={error}
            mode="library"
            scrollToMediaId={scrollTargetMediaId}
            onScrolledToMediaId={() => setScrollTargetMediaId(null)}
        />
    );
}

function tabContentExtraProps(value: ActiveTab) {
    if (value === "imports" || value === "library") {
        return { minH: 0, display: "flex", flexDirection: "column" } as const;
    }
    return { overflowY: "auto" } as const;
}

export const MainContent = () => {
    const loadAllMedia = useMediaStore((s) => s.loadAllMedia);
    const refreshItemByRelPath = useMediaStore((s) => s.refreshItemByRelPath);
    const clearHighlight = useMediaStore((s) => s.clearHighlight);

    useTauriEvent<JobCompletedPayload>("job-completed", (event) => {
        if (useInterfaceStore.getState().activeTab !== "library") return;
        const relPath = event.payload?.relPath;
        if (!relPath?.startsWith("Library/")) return;
        void refreshItemByRelPath(relPath);
    });

    useTauriEvent("library-changed", () => {
        void loadAllMedia();
    });

    return (
        <Box as="main" flex="1" minH={0} display="flex" flexDirection="column">
            {TABS.map((tab) => (
                <Tabs.Content
                    key={tab.value}
                    value={tab.value}
                    flex="1"
                    onClick={tab.value === "library" ? clearHighlight : undefined}
                    {...tabContentExtraProps(tab.value)}
                >
                    {TAB_PANELS[tab.value]()}
                </Tabs.Content>
            ))}
        </Box>
    );
};
