import { Box, HStack, Text, VStack } from "@chakra-ui/react";
import { type MediaFileRef } from "@/types/mediaTypes";
import { Tooltip } from "@/components/ui/tooltip";

interface FilesTabProps {
    files: MediaFileRef[];
}

// TODO(viewer): add a Tauri `open_directory_in_explorer(dirPath)` command
// and re-introduce a per-row "Show in Explorer" button. The previous button
// silently called `open_library_root` regardless of the row, which was misleading
export function FilesTab({ files }: FilesTabProps) {
    return (
        <VStack align="stretch" gap={2} flex={1} overflowY="auto" px={1}>
            {files.map((f) => (
                <HStack
                    key={f.id}
                    p={2.5}
                    bg="whiteAlpha.50"
                    borderRadius="md"
                    gap={2}
                    _hover={{ bg: "whiteAlpha.100" }}
                    transition="background 0.15s"
                >
                    <Box flex={1} minW={0}>
                        <Text fontSize="sm" truncate>
                            {f.fileName}.{f.ext}
                        </Text>
                        <Tooltip content={f.relPath} showArrow>
                            <Text fontSize="xs" color="gray.500" truncate cursor="default">
                                {f.dirPath}
                            </Text>
                        </Tooltip>
                    </Box>
                </HStack>
            ))}
        </VStack>
    );
}
