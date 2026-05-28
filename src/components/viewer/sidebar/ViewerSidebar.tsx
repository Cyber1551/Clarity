import { Box, Tabs, Text } from "@chakra-ui/react";
import { type MediaDetail } from "@/types/mediaTypes";
import { type ViewerMode } from "@/stores/mediaStore";
import { InfoTab } from "./tabs/InfoTab";
import { FilesTab } from "./tabs/FilesTab";
import { SubtitlesTab } from "./tabs/SubtitlesTab";
import { getVisibleTabs, type ViewerTabId } from "./tabsConfig";
import { useState } from "react";
import { SIDEBAR_WIDTH } from "../constants";

interface ViewerSidebarProps {
    open: boolean;
    detail: MediaDetail | null;
    mode: ViewerMode;
    onDetailChanged: () => void;
}

const TAB_RENDERERS: Record<
    ViewerTabId,
    (props: { detail: MediaDetail | null; mode: ViewerMode; onDetailChanged: () => void }) => React.ReactNode
> = {
    info: ({ detail, mode, onDetailChanged }) =>
        detail ? <InfoTab detail={detail} mode={mode} onDetailChanged={onDetailChanged} /> : null,
    files: ({ detail }) => (detail ? <FilesTab files={detail.files} /> : null),
    subtitles: () => <SubtitlesTab />,
};

export function ViewerSidebar({ open, detail, mode, onDetailChanged }: ViewerSidebarProps) {
    const visibleTabs = getVisibleTabs(mode);
    const [activeTab, setActiveTab] = useState<ViewerTabId>("info");

    const safeActiveTab = visibleTabs.some((t) => t.id === activeTab) ? activeTab : visibleTabs[0]?.id ?? "info";

    return (
        <Box
            w={open ? SIDEBAR_WIDTH : "0px"}
            minW={open ? SIDEBAR_WIDTH : "0px"}
            overflow="hidden"
            transition="width 0.25s ease, min-width 0.25s ease"
            borderLeftWidth={open ? "1px" : "0px"}
            borderColor="whiteAlpha.100"
            bg="rgba(17, 17, 17, 0.95)"
            h="full"
        >
            <Tabs.Root
                value={safeActiveTab}
                onValueChange={(e) => setActiveTab(e.value as ViewerTabId)}
                variant="line"
                w={SIDEBAR_WIDTH}
                h="full"
                display="flex"
                flexDirection="column"
            >
                <Tabs.List borderBottomWidth="1px" borderColor="whiteAlpha.100" flexShrink={0}>
                    {visibleTabs.map((tab) => {
                        const badge = tab.badge?.(detail);
                        return (
                            <Tabs.Trigger
                                key={tab.id}
                                value={tab.id}
                                flex={1}
                                justifyContent="center"
                                fontSize="xs"
                            >
                                {tab.label}
                                {badge != null && (
                                    <Text as="span" ml={1} fontSize="2xs" color="gray.500">
                                        ({badge})
                                    </Text>
                                )}
                            </Tabs.Trigger>
                        );
                    })}
                </Tabs.List>

                {visibleTabs.map((tab) => (
                    <Tabs.Content
                        key={tab.id}
                        value={tab.id}
                        flex={1}
                        p={4}
                        overflowY="auto"
                    >
                        {TAB_RENDERERS[tab.id]({ detail, mode, onDetailChanged })}
                    </Tabs.Content>
                ))}
            </Tabs.Root>
        </Box>
    );
}
