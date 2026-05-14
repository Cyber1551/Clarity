import { Button, Spinner, Text, VStack } from "@chakra-ui/react";
import { Folder } from "lucide-react";

interface ImportsSidebarProps {
    folders: string[];
    selectedFolder: string | null;
    isLoading: boolean;
    error: string | null;
    onSelect: (folder: string) => void;
}

export function ImportsSidebar({
    folders,
    selectedFolder,
    isLoading,
    error,
    onSelect,
}: ImportsSidebarProps) {
    return (
        <VStack
            w="250px"
            borderRightWidth="1px"
            borderColor="gray.100"
            align="stretch"
            p={4}
            gap={2}
            overflowY="auto"
        >
            <Text fontWeight="bold" fontSize="sm" color="gray.500" mb={2}>IMPORT SESSIONS</Text>
            {isLoading && folders.length === 0 && <Spinner size="sm" />}
            {folders.map(folder => {
                const isSelected = selectedFolder === folder;
                return (
                    <Button
                        key={folder}
                        variant="ghost"
                        size="sm"
                        justifyContent="flex-start"
                        bg={isSelected ? "blue.50" : "transparent"}
                        color={isSelected ? "blue.600" : "inherit"}
                        _hover={{ bg: isSelected ? "blue.50" : "gray.700" }}
                        fontWeight={isSelected ? "bold" : "normal"}
                        onClick={() => onSelect(folder)}
                    >
                        <Folder size={16} />
                        <Text fontSize="sm" truncate>
                            {folder}
                        </Text>
                    </Button>
                );
            })}
            {folders.length === 0 && !isLoading && (
                <Text fontSize="xs" color="gray.400">No imports found</Text>
            )}
            {error && <Text fontSize="xs" color="red.500">{error}</Text>}
        </VStack>
    );
}
