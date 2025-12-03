import React from 'react';
import { Flex, HStack, IconButton, Tabs, Text } from "@chakra-ui/react";
import { LuSearch, LuSettings } from "react-icons/lu";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";

interface HeaderProps {
    folderPath: string | null;
    cacheActionText: { [key: number]: string };
    onPickFolder: () => Promise<void>;
}

const Header: React.FC<HeaderProps> = () => {
    const setSettingsDialogOpen = useInterfaceStore(s => s.setSettingsDialogOpen);
    return (
        <Flex
            as="header"
            h="64px"
            px="32px"
            align="center"
            justify="space-between"
            borderBottomWidth="1px"
            borderColor="gray.100"
        >
            <Text fontSize="xl" fontWeight="bold">
                Clarity
            </Text>
            <Tabs.List>
                <Tabs.Trigger value="dashboard">
                    Dashboard
                </Tabs.Trigger>
                <Tabs.Trigger value="library">
                    Library
                </Tabs.Trigger>
                <Tabs.Trigger value="explore">
                    Explore
                </Tabs.Trigger>
                <Tabs.Trigger value="moments">
                    Moments
                </Tabs.Trigger>
                <Tabs.Trigger value="session">
                    Sessions
                </Tabs.Trigger>
            </Tabs.List>

            {/* Right: search, bell, avatar */}
            <HStack gap={3}>
                <IconButton aria-label="Search" variant="ghost" rounded="full">
                    <LuSearch />
                </IconButton>

                <IconButton aria-label="Settings" variant="ghost" rounded="full"
                            onClick={() => setSettingsDialogOpen(true)}>
                    <LuSettings />
                </IconButton>
            </HStack>
        </Flex>
    );
};

export default Header;
