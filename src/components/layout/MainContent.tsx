import React from "react";
import { Box, Tabs } from "@chakra-ui/react";
import { useMediaStore } from "@/stores/mediaStore";
import { MediaGrid } from "@/components/library";
import { ImportsView } from "@/components/imports";
import { useMediaItems } from "@/queries/library/useMediaItems";
import { TABS, type ActiveTab } from "@/constants/tabs";

const TAB_PANELS: Record<ActiveTab, () => React.ReactNode> = {
    dashboard: () => "Dashboard",
    imports: () => <ImportsView />,
    library: () => <LibraryPanel />,
    people: () => "People",
    //explore: () => "Explore",
    //moments: () => "Moments",
    //session: () => "Sessions",
};

function LibraryPanel() {
    const { items, isLoading, error } = useMediaItems();
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
    const clearHighlight = useMediaStore((s) => s.clearHighlight);

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
