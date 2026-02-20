import React from 'react';
import { Button, Flex, HStack, IconButton, Tabs, Text } from "@chakra-ui/react";
import { LuSearch, LuSettings, LuPlus, LuFolderOpen } from "react-icons/lu";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { useConfigStore } from "@/stores/configStore.ts";
import { import_files } from "@/api/importApi";
import { openLibraryRoot } from "@/api/configApi";

interface HeaderProps {
    folderPath: string | null;
    cacheActionText: { [key: number]: string };
    onPickFolder: () => Promise<void>;
}

const Header: React.FC<HeaderProps> = () => {
    const setSettingsDialogOpen = useInterfaceStore(s => s.setSettingsDialogOpen);
    const setActiveTab = useInterfaceStore(s => s.setActiveTab);
    const setSelectedImportFolder = useInterfaceStore(s => s.setSelectedImportFolder);
    const libraryRoot = useConfigStore(s => s.config?.libraryRoot);

    const handleImport = async () => {
        try {
            const folderName = await import_files();
            if (folderName) {
                setSelectedImportFolder(folderName);
                setActiveTab("imports");
            }
        } catch (e) {
            console.error("Import failed", e);
        }
    };

    const handleOpenLibraryRoot = async () => {
        if (!libraryRoot) return;

        try {
            await openLibraryRoot();
        } catch (e) {
            console.error("Failed to open library folder", e);
        }
    };

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
                <Tabs.Trigger value="imports">
                    Imports
                </Tabs.Trigger>
                <Tabs.Trigger value="library">
                    Library
                </Tabs.Trigger>
                <Tabs.Trigger value="people">
                    People
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
                <Button size="sm" onClick={handleImport} variant="outline">
                    <LuPlus /> Import
                </Button>
                
                <IconButton aria-label="Search" variant="ghost" rounded="full">
                    <LuSearch />
                </IconButton>

                <IconButton
                    aria-label="Open Library Folder"
                    variant="ghost"
                    rounded="full"
                    disabled={!libraryRoot}
                    onClick={handleOpenLibraryRoot}
                >
                    <LuFolderOpen />
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
