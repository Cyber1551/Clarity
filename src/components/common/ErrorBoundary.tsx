import { Box, Button, Code, Heading, Text, VStack } from "@chakra-ui/react";
import { Component, type ErrorInfo, type ReactNode } from "react";
import { formatError } from "@/utils/format";
import { logger } from "@/utils/logger";

interface ErrorBoundaryProps {
    /** Logging scope and fallback heading. */
    name: string;
    children: ReactNode;
}

interface ErrorBoundaryState {
    error: Error | null;
    /** When true, the error message + stack panel is rendered below the buttons. */
    showDetails: boolean;
}

/**
 * Catches render-time errors anywhere in the React tree and shows a
 * fullscreen fallback with a centered summary, a Reload button, and a
 * collapsible "Show stack" panel for the technical details.
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    override state: ErrorBoundaryState = { error: null, showDetails: false };

    static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { error, showDetails: false };
    }

    override componentDidCatch(error: Error, info: ErrorInfo) {
        logger.error(`react-boundary:${this.props.name}`, error, {
            componentStack: info.componentStack,
        });
    }

    private handleReload = () => {
        window.location.reload();
    };

    private toggleDetails = () => {
        this.setState((s) => ({ showDetails: !s.showDetails }));
    };

    override render() {
        const { error, showDetails } = this.state;
        if (!error) return this.props.children;

        return (
            <Box
                minH="100vh"
                w="full"
                bg="bg.panel"
                color="fg"
                display="flex"
                alignItems="center"
                justifyContent="center"
                p={6}
                overflowY="auto"
                role="alert"
                aria-live="assertive"
                aria-labelledby="error-boundary-title"
                aria-describedby="error-boundary-desc"
            >
                <VStack gap={4} maxW="2xl" w="full" textAlign="center">
                    <Heading id="error-boundary-title" size="md">
                        &quot;{this.props.name}&quot; stopped working
                    </Heading>
                    <Text id="error-boundary-desc" fontSize="sm" color="fg.muted">
                        Clarity hit an error it couldn&apos;t recover from. Reload the app to start fresh.
                    </Text>

                    <Button onClick={this.handleReload}>Reload app</Button>

                    <Button size="sm" variant="ghost" onClick={this.toggleDetails}>
                        {showDetails ? "Hide stack" : "Show stack"}
                    </Button>

                    {showDetails && (
                        <Code
                            as="pre"
                            display="block"
                            fontSize="xs"
                            textAlign="left"
                            w="full"
                            p={3}
                            bg="bg.subtle"
                            borderRadius="md"
                            whiteSpace="pre-wrap"
                            wordBreak="break-word"
                            maxH="50vh"
                            overflowY="auto"
                        >
                            {formatError(error)}
                            {error.stack ? `\n\n${error.stack}` : ""}
                        </Code>
                    )}
                </VStack>
            </Box>
        );
    }
}
