import { Box, HStack, Text, VStack } from "@chakra-ui/react";
import { Tooltip } from "@/components/ui/tooltip";
import type { LucideIcon } from "lucide-react";

interface ComingSoonRowProps {
    /** Section heading. */
    heading: string;
    /** Empty-state body text. */
    emptyText: string;
    icon: LucideIcon;
    actionLabel: string;
}

/**
 * Placeholder section used by the viewer info pane for not-yet-shipped features
 * Renders heading + empty-state + disabled action with a "Coming soon" tooltip.
 */
export function ComingSoonRow({ heading, emptyText, icon: Icon, actionLabel }: ComingSoonRowProps) {
    return (
        <VStack align="stretch" gap={1.5}>
            <Text fontSize="xs" color="gray.400" fontWeight="medium">{heading}</Text>
            <Text fontSize="xs" color="gray.600">{emptyText}</Text>
            <Tooltip content="Coming soon">
                <Box>
                    <HStack
                        gap={1}
                        fontSize="xs"
                        color="gray.600"
                        cursor="not-allowed"
                        opacity={0.5}
                    >
                        <Icon size={12} />
                        <Text>{actionLabel}</Text>
                    </HStack>
                </Box>
            </Tooltip>
        </VStack>
    );
}
