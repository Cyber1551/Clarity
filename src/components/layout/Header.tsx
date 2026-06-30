import { Button, Flex, HStack, IconButton, Kbd, Tabs, Text } from "@chakra-ui/react";
import { Search, Settings, Plus, FolderOpen } from "lucide-react";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { useConfigStore } from "@/stores/configStore.ts";
import { import_files } from "@/api/importApi";
import { openLibraryRoot } from "@/api/configApi";
import { TABS } from "@/constants/tabs";
import { notify } from "@/utils/notify";
import { SyncButton } from "./SyncButton";

const isMac = typeof navigator !== "undefined" && navigator.userAgent.toLowerCase().includes("mac");
const SEARCH_SHORTCUT = isMac ? "⌘K" : "Ctrl+K";

export const Header = () => {
    const setSettingsDialogOpen = useInterfaceStore(s => s.setSettingsDialogOpen);
    const setSearchOpen = useInterfaceStore(s => s.setSearchOpen);
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
            notify.error("Import failed", e);
        }
    };

    const handleOpenLibraryRoot = async () => {
        if (!libraryRoot) return;

        try {
            await openLibraryRoot();
        } catch (e) {
            notify.error("Couldn't open library folder", e);
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
                <SyncButton />

                <Button size="sm" onClick={() => void handleImport()} variant="outline">
                    <Plus size={16} /> Import
                </Button>

                <Button
                    aria-label="Search"
                    size="sm"
                    variant="outline"
                    color="gray.500"
                    onClick={() => setSearchOpen(true)}
                >
                    <Search size={16} />
                    Search
                    <Kbd>{SEARCH_SHORTCUT}</Kbd>
                </Button>

                <IconButton
                    aria-label="Open Library Folder"
                    variant="ghost"
                    rounded="full"
                    disabled={!libraryRoot}
                    onClick={() => void handleOpenLibraryRoot()}
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
