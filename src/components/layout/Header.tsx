import { Button, Flex, HStack, IconButton, Tabs, Text } from "@chakra-ui/react";
import { Search, Settings, Plus, FolderOpen } from "lucide-react";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { useConfigStore } from "@/stores/configStore.ts";
import { import_files } from "@/api/importApi";
import { openLibraryRoot } from "@/api/configApi";
import { TABS } from "@/constants/tabs";

export const Header = () => {
    const setSettingsDialogOpen = useInterfaceStore(s => s.setSettingsDialogOpen);
    const setActiveTab = useInterfaceStore(s => s.setActiveTab);
    const setSelectedImportFolder = useInterfaceStore(s => s.setSelectedImportFolder);
    const setLastImportResult = useInterfaceStore(s => s.setLastImportResult);
    const libraryRoot = useConfigStore(s => s.config?.libraryRoot);

    const handleImport = async () => {
        try {
            const result = await import_files();
            if (result?.folderName) {
                setSelectedImportFolder(result.folderName);
                setActiveTab("imports");
            }
            if (result) {
                setLastImportResult(result);
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
                {TABS.map((tab) => (
                    <Tabs.Trigger key={tab.value} value={tab.value}>
                        {tab.label}
                    </Tabs.Trigger>
                ))}
            </Tabs.List>

            <HStack gap={3}>
                <Button size="sm" onClick={handleImport} variant="outline">
                    <Plus size={16} /> Import
                </Button>

                <IconButton aria-label="Search" variant="ghost" rounded="full">
                    <Search size={18} />
                </IconButton>

                <IconButton
                    aria-label="Open Library Folder"
                    variant="ghost"
                    rounded="full"
                    disabled={!libraryRoot}
                    onClick={handleOpenLibraryRoot}
                >
                    <FolderOpen size={18} />
                </IconButton>

                <IconButton
                    aria-label="Settings"
                    variant="ghost"
                    rounded="full"
                    onClick={() => setSettingsDialogOpen(true)}
                >
                    <Settings size={18} />
                </IconButton>
            </HStack>
        </Flex>
    );
};
