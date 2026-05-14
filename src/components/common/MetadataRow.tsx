import { HStack, Text } from "@chakra-ui/react";
import type { ReactNode } from "react";

interface MetadataRowProps {
    label: ReactNode;
    value: ReactNode;
    /** Override label/value font size (default "xs"). */
    size?: "2xs" | "xs" | "sm";
    /** Tooltip-friendly slot: render the value with a custom wrapper. */
    valueWrapper?: (value: ReactNode) => ReactNode;
}

/**
 * Standard "label on the left, value on the right" row used throughout
 * the viewer info pane and similar metadata displays.
 */
export function MetadataRow({ label, value, size = "xs", valueWrapper }: MetadataRowProps) {
    const labelColor = size === "2xs" ? "gray.500" : "gray.400";
    const valueNode = <Text fontSize={size}>{value}</Text>;
    return (
        <HStack justify="space-between">
            <Text fontSize={size} color={labelColor}>{label}</Text>
            {valueWrapper ? valueWrapper(valueNode) : valueNode}
        </HStack>
    );
}
