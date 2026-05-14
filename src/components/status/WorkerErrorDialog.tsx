import { Box, Button, Text, VStack } from "@chakra-ui/react";
import { AppDialog } from "@/components/common";

interface WorkerErrorDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    /** Long-form message describing the current state (composed by the parent). */
    message: string;
    lastError: string | null;
    retrying: boolean;
    onRetry: () => void;
}

export function WorkerErrorDialog({
    open,
    onOpenChange,
    message,
    lastError,
    retrying,
    onRetry,
}: WorkerErrorDialogProps) {
    return (
        <AppDialog
            open={open}
            onOpenChange={onOpenChange}
            title="Background processing error"
            size="2xl"
            footer={
                <>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        Close
                    </Button>
                    <Button
                        variant="solid"
                        colorPalette="red"
                        loading={retrying}
                        loadingText="Retrying"
                        onClick={onRetry}
                    >
                        Retry
                    </Button>
                </>
            }
        >
            <VStack align="stretch" gap={3}>
                <Text fontSize="sm" color="fg.muted">{message}</Text>
                <Box
                    as="pre"
                    bg="bg.muted"
                    borderWidth="1px"
                    borderColor="border"
                    borderRadius="md"
                    p={3}
                    fontSize="xs"
                    fontFamily="mono"
                    whiteSpace="pre-wrap"
                    wordBreak="break-word"
                    maxH="60vh"
                    overflowY="auto"
                >
                    {lastError ?? "No error message available."}
                </Box>
            </VStack>
        </AppDialog>
    );
}
