import { Box, Button, Center, Spinner, Tabs, Text, VStack } from "@chakra-ui/react";
import { useEffect } from "react";
import { Header, MainContent } from "@/components/layout";
import { SettingsDialog } from "@/components/settings";
import { Viewer } from "@/components/viewer";
import { WorkerStatusBanner } from "@/components/status";
import { useConfigStore } from "@/stores/configStore.ts";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { initialize_library } from "@/api/libraryApi.ts";
import { useQueryInvalidationBridge } from "@/queries/useQueryInvalidationBridge";

const App = () => {
    const config = useConfigStore(s => s.config);
    const isLoading = useConfigStore(s => s.isLoading);
    const error = useConfigStore(s => s.error);
    const initConfig = useConfigStore(s => s.initConfig);
    const pickLibraryRoot = useConfigStore(s => s.pickLibraryRoot);
    const activeTab = useInterfaceStore(s => s.activeTab);
    const setActiveTab = useInterfaceStore(s => s.setActiveTab);
    const setLibraryReady = useInterfaceStore(s => s.setLibraryReady);

    // Single Tauri-event-to-query-invalidation bridge.
    // Replaces the scattered useTauriEvent calls that used to live in MainContent + import hooks.
    useQueryInvalidationBridge();

    useEffect(() => {
        void initConfig();
    }, [initConfig]);

    useEffect(() => {
        if (config?.libraryRoot) {
            // Reset the readiness flag so the catalog query waits for the new `library-initialized` event before refetching against a freshly (re-)initialized library.
            setLibraryReady(false);
            void initialize_library();
        }
    }, [config?.libraryRoot, setLibraryReady]);

    if (isLoading && !config) {
        return (
            <Box minH="100vh">
                <Spinner />
            </Box>
        );
    }

    if (!config?.libraryRoot) {
        return (
            <Center minH="100vh" gap={4} flexDirection="column">
                <VStack gap={2}>
                    <Text fontSize="xl" fontWeight="semibold">Choose a media library folder</Text>
                    <Text fontSize="sm" color="gray.500" maxW="md" textAlign="center">
                        Pick a folder to use as your library root. All scans, tags, and
                        thumbnails will live inside this folder.
                    </Text>
                </VStack>

                {error && <Text fontSize="xs" color="red.500">{error}</Text>}

                <Button mt={4} onClick={() => void pickLibraryRoot()} disabled={isLoading} variant="outline">
                    {isLoading ? "Opening picker…" : "Pick a folder to continue"}
                </Button>
            </Center>
        );
    }

    return (
        <Box minH="100vh">
            <WorkerStatusBanner />
            <Tabs.Root
                value={activeTab}
                onValueChange={(e) => setActiveTab(e.value)}
                variant="enclosed"
                display="flex"
                flexDirection="column"
                h="100vh"
            >
                <Header />
                <MainContent />
            </Tabs.Root>
            <Viewer />
            <SettingsDialog />
        </Box>
    );
};

export default App;
