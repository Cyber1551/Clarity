import FileGrid from '@/components/FileGrid.tsx';
import { Box, Tabs } from "@chakra-ui/react";
import FileTree from "@/components/FileTree.tsx";

const MainContent = () => {
    return (
        <Box as={"main"} flex="1" minH={0} overflowY="auto">
            <Tabs.Content value="dashboard">
                Dashboard
            </Tabs.Content>
            <Tabs.Content value="files" className={"flex h-full"} py={0}>
                <FileTree />
                <div className="min-w-0 flex-1 flex flex-col">
                    <FileGrid />
                </div>
            </Tabs.Content>
            <Tabs.Content value="favorites">
                Favorites
            </Tabs.Content>
            <Tabs.Content value="tags">
                Tags
            </Tabs.Content>
            <Tabs.Content value="moments">
                Moments
            </Tabs.Content>
            <Tabs.Content value="session">
                Sessions
            </Tabs.Content>
            <Tabs.Content value="search">
                Search
            </Tabs.Content>
        </Box>
    );
};

export default MainContent;
