import { Text, VStack } from "@chakra-ui/react";
import { Subtitles } from "lucide-react";

export function SubtitlesTab() {
    return (
        <VStack align="center" justify="center" flex={1} gap={3} py={12} color="gray.500">
            <Subtitles size={32} />
            <Text fontSize="sm">No subtitles available</Text>
        </VStack>
    );
}
