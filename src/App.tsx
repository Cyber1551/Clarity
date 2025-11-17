import Header from "@/components/Header";
import MainContent from "@/components/MainContent";
import { Box, Button, Center, Spinner, Tabs } from "@chakra-ui/react";
import { useConfigStore } from "@/stores/configStore.ts";
import { useEffect } from "react";
import { SettingsDialog } from "@/components/SettingsDialog.tsx";
import { initialize_library_dirs } from "@/api/libraryApi.ts";

const App = () => {
    const config = useConfigStore(s => s.config);
    const isLoading = useConfigStore(s => s.isLoading);
    const error = useConfigStore(s => s.error);
    const initConfig = useConfigStore(s => s.initConfig);
    const pickLibraryRoot = useConfigStore(s => s.pickLibraryRoot);

    useEffect(() => {
        void initConfig();
    }, [initConfig]);

    useEffect(() => {
        if (config?.libraryRoot) {
            void initialize_library_dirs();
        }
    }, [config?.libraryRoot]);

    if (isLoading && !config) {
        return (
            <Box minH={"100vh"}>
                <Spinner />
            </Box>
        )
    }

    if (!config?.libraryRoot) {
        return (
            <Center minH={"100vh"} gap={4} flexDirection={"column"}>
                <div className="flex flex-col items-center gap-2">
                    <h1 className="text-xl font-semibold">Choose a media library folder</h1>
                    <p className="text-sm text-muted-foreground max-w-md text-center">
                        Pick a folder to use as your library root. All scans, tags, and
                        thumbnails will live inside this folder.
                    </p>
                </div>

                {error && <p className="text-xs text-red-500">{error}</p>}

                <Button
                    marginTop={4}
                    onClick={pickLibraryRoot}
                    disabled={isLoading}
                    className="px-4 py-2 rounded-md border"
                >
                    {isLoading ? "Opening picker…" : "Pick a folder to continue"}
                </Button>
            </Center>
        )
    }

    return (
        <Box minH={"100vh"}>
            <Tabs.Root
                defaultValue="dashboard"
                variant={"enclosed"}
                display="flex"
                flexDirection="column"
                h="100vh"
            >
                {/* Header with app title, folder selection, and cache status */}
                <Header
                    folderPath={""}
                    cacheActionText={{}}
                    onPickFolder={() => Promise.resolve()}
                />
                {/* Main content area that should fill the remaining space */}
                <MainContent />
            </Tabs.Root>
            <SettingsDialog />
        </Box>
    );
}

export default App;
