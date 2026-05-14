import { Box, Button, Spinner, Text } from "@chakra-ui/react";

interface StageStatusProps {
    loading?: boolean;
    error?: string | null;
    onRetry?: () => void;
}

/**
 * Shared loading / error / retry placeholder for the viewer stage area.
 * Returns `null` when neither loading nor errored
 */
export function StageStatus({ loading, error, onRetry }: StageStatusProps) {
    if (loading) {
        return (
            <Box flex={1} display="flex" alignItems="center" justifyContent="center" bg="black">
                <Spinner size="xl" color="white" />
            </Box>
        );
    }

    if (error) {
        return (
            <Box
                flex={1}
                display="flex"
                flexDirection="column"
                alignItems="center"
                justifyContent="center"
                bg="black"
                gap={3}
            >
                <Text color="red.400" fontSize="sm">{error}</Text>
                {onRetry && (
                    <Button size="sm" variant="subtle" colorPalette="gray" onClick={onRetry}>
                        Retry
                    </Button>
                )}
            </Box>
        );
    }

    return null;
}
