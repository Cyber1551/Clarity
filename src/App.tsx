import { Box, Button, Center, Spinner, Tabs, Text, VStack } from "@chakra-ui/react";
import { useEffect } from "react";
import { Header, MainContent } from "@/components/layout";
import { SettingsDialog } from "@/components/settings";
import { Viewer } from "@/components/viewer";
import { WorkerStatusBanner } from "@/components/status";
import { ErrorBoundary } from "@/components/common";
import { useConfigStore } from "@/stores/configStore.ts";
import { useMediaStore } from "@/stores/mediaStore.ts";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { initialize_library } from "@/api/libraryApi.ts";
import { useTauriEvent } from "@/hooks/useTauriEvent";

const App = () => {
    const config = useConfigStore(s => s.config);
    const isLoading = useConfigStore(s => s.isLoading);
    const error = useConfigStore(s => s.error);
    const initConfig = useConfigStore(s => s.initConfig);
    const pickLibraryRoot = useConfigStore(s => s.pickLibraryRoot);
    const loadAllMedia = useMediaStore(s => s.loadAllMedia);
    const setIsLoading = useMediaStore(s => s.setIsLoading);
    const activeTab = useInterfaceStore(s => s.activeTab);
    const setActiveTab = useInterfaceStore(s => s.setActiveTab);

    useEffect(() => {
        void initConfig();
    }, [initConfig]);

    useTauriEvent("library-initialized", () => {
        console.log("Library initialized, loading media...");
        void loadAllMedia();
    });

    useEffect(() => {
        if (config?.libraryRoot) {
            setIsLoading(true);
            void initialize_library();
        }
    }, [config?.libraryRoot, setIsLoading]);

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

                <Button mt={4} onClick={pickLibraryRoot} disabled={isLoading} variant="outline">
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
            <ErrorBoundary
                level="route"
                title="This view ran into a problem"
                description="You can close the viewer and keep using the rest of the app. Your library and files are unaffected."
                resetLabel="Close viewer"
                onReset={() => useMediaStore.getState().closeViewer()}
            >
                <Viewer />
            </ErrorBoundary>
            <SettingsDialog />
        </Box>
    );
};

export default App;
