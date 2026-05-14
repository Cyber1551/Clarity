import { HStack, IconButton, Text } from "@chakra-ui/react";
import { Tooltip } from "@/components/ui/tooltip";
import type { LucideIcon } from "lucide-react";

interface RatingRowProps {
    label: string;
    icon: LucideIcon;
    /** The currently selected rating value (0 = none). */
    value: number;
    /** Maximum number of buttons to render. Defaults to 3. */
    max?: number;
    /** Color tokens for the active and hover states. */
    colorActive: string;
    colorHover: string;
    onChange: (newValue: number) => void;
    /** Optional override for the inactive icon color. */
    colorInactive?: string;
    /** Width of the label column. */
    labelWidth?: string;
}

/**
 * Generic "label + N clickable icons" rating row. Used for both the
 * Favorite (star) and Quality (gem) controls in the viewer info tab.
 *
 * Clicking the currently selected value clears the rating (sets it to 0).
 */
export function RatingRow({
    label,
    icon: Icon,
    value,
    max = 3,
    colorActive,
    colorHover,
    colorInactive = "gray.600",
    labelWidth = "50px",
    onChange,
}: RatingRowProps) {
    return (
        <HStack gap={2} align="center">
            <Text fontSize="2xs" color="gray.500" w={labelWidth} flexShrink={0}>
                {label}
            </Text>
            <HStack gap={0.5}>
                {Array.from({ length: max }, (_, i) => i + 1).map((step) => {
                    const isActive = value >= step;
                    const newValue = value === step ? 0 : step;
                    return (
                        <Tooltip key={step} content={`${label} ${step}`}>
                            <IconButton
                                aria-label={`${label} ${step}`}
                                variant="ghost"
                                size="xs"
                                color={isActive ? colorActive : colorInactive}
                                onClick={() => onChange(newValue)}
                                _hover={{ color: colorHover }}
                            >
                                <Icon size={14} fill={isActive ? "currentColor" : "none"} />
                            </IconButton>
                        </Tooltip>
                    );
                })}
            </HStack>
        </HStack>
    );
}
