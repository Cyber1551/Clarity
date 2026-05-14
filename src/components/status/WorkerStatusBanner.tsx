import { useState } from "react";
import { Box, Button, HStack, Text, VStack } from "@chakra-ui/react";
import { AlertTriangle } from "lucide-react";
import { useWorkerStatusStore } from "@/stores/workerStatusStore";
import { useTauriEvent } from "@/hooks/useTauriEvent";
import { type WorkerStalledPayload } from "@/types/eventTypes";
import { WorkerErrorDialog } from "./WorkerErrorDialog";

interface StatusMessages {
    inline: string;
    modal: string;
}

function getStatusMessages(state: {
    retrying: boolean;
    lastRetryFailed: boolean;
    consecutiveFailures: number;
}): StatusMessages {
    if (state.retrying) {
        return {
            inline: "Retrying now\u2026",
            modal: "Retrying now. Waiting for the worker to recover\u2026",
        };
    }
    const plural = state.consecutiveFailures === 1 ? "" : "s";
    if (state.lastRetryFailed) {
        return {
            inline: "Retry didn't resolve the issue. The worker is still failing.",
            modal: `The retry attempt did not resolve the issue. The worker has failed ${state.consecutiveFailures} consecutive time${plural} and will keep retrying with backoff in the background.`,
        };
    }
    return {
        inline: "Something went wrong while processing background jobs. New imports may not be indexed until this is resolved.",
        modal: `The job worker has failed ${state.consecutiveFailures} consecutive time${plural}. It will keep retrying with backoff in the background. Click Retry to attempt immediately.`,
    };
}

export function WorkerStatusBanner() {
    const stalled = useWorkerStatusStore((s) => s.stalled);
    const lastError = useWorkerStatusStore((s) => s.lastError);
    const consecutiveFailures = useWorkerStatusStore((s) => s.consecutiveFailures);
    const retrying = useWorkerStatusStore((s) => s.retrying);
    const lastRetryFailed = useWorkerStatusStore((s) => s.lastRetryFailed);
    const onStalled = useWorkerStatusStore((s) => s.onStalled);
    const onRecovered = useWorkerStatusStore((s) => s.onRecovered);
    const retry = useWorkerStatusStore((s) => s.retry);

    const [detailsOpen, setDetailsOpen] = useState(false);

    useTauriEvent<WorkerStalledPayload>("worker-stalled", (e) => onStalled(e.payload));
    useTauriEvent("worker-recovered", () => {
        onRecovered();
        setDetailsOpen(false);
    });

    if (!stalled) return null;

    const messages = getStatusMessages({ retrying, lastRetryFailed, consecutiveFailures });
    const handleRetry = () => void retry();

    return (
        <>
            <Box
                position="fixed"
                bottom={4}
                right={4}
                zIndex={1500}
                bg="red.900"
                color="red.50"
                borderWidth="1px"
                borderColor="red.700"
                borderRadius="md"
                boxShadow="lg"
                px={4}
                py={3}
                maxW="sm"
            >
                <VStack align="stretch" gap={2}>
                    <HStack gap={2} align="center">
                        <AlertTriangle size={18} />
                        <Text fontSize="sm" fontWeight="semibold">
                            Background processing stalled
                        </Text>
                    </HStack>
                    <Text fontSize="xs" color="red.200">
                        {messages.inline}
                    </Text>
                    <HStack gap={2} justify="flex-end">
                        <Button
                            size="xs"
                            variant="ghost"
                            colorPalette="red"
                            onClick={() => setDetailsOpen(true)}
                        >
                            Details
                        </Button>
                        <Button
                            size="xs"
                            variant="outline"
                            colorPalette="red"
                            loading={retrying}
                            loadingText="Retrying"
                            onClick={handleRetry}
                        >
                            Retry
                        </Button>
                    </HStack>
                </VStack>
            </Box>

            <WorkerErrorDialog
                open={detailsOpen}
                onOpenChange={setDetailsOpen}
                message={messages.modal}
                lastError={lastError}
                retrying={retrying}
                onRetry={handleRetry}
            />
        </>
    );
}
