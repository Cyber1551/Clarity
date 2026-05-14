import { Box, Button, Code, HStack, Heading, Text, VStack } from "@chakra-ui/react";
import { Component, type ErrorInfo, type ReactNode } from "react";
import { formatError } from "@/utils/format";

/**
 * Scope of the boundary, used to decide which recovery actions and which presentation are appropriate. Pick the smallest scope you can
 * Nested boundaries degrade gracefully and keep the rest of the app usable.
 *   - "root"  : app-wide catastrophic fallback. Primary action: Reload.
 *   - "route" : a top-level view/modal. Primary action: caller-supplied
 *               reset (e.g. close the modal / go back), Reload as secondary.
 *   - "leaf"  : a small panel/widget. Renders an inline placeholder with
 *               a "Try again" link so the surrounding UI stays usable.
 */
export type ErrorBoundaryLevel = "root" | "route" | "leaf";

interface ErrorBoundaryProps {
    /** Boundary scope. Defaults to "root". */
    level?: ErrorBoundaryLevel;
    /** Optional friendly headline shown to users. */
    title?: string;
    /** Optional supporting text shown below the headline. */
    description?: string;
    /**
     * Custom recovery callback. For "route" boundaries this powers the
     * primary action button (e.g. close the broken modal). The boundary
     * always resets its internal error state before invoking it.
     */
    onReset?: () => void;
    /** Label for the primary recovery action on "route" boundaries. */
    resetLabel?: string;
    /** Full custom fallback. When provided, overrides the built-in UI. */
    fallback?: (error: Error, reset: () => void) => ReactNode;
    children: ReactNode;
}

interface ErrorBoundaryState {
    error: Error | null;
    showDetails: boolean;
}

/**
 * Catches render-time errors in the React tree and presents a recoverable
 * fallback so a single misbehaving component doesn't take down the whole app.
 *   - Users never see the raw error message as the primary content; the
 *     technical details live behind a "Show details" disclosure.
 *   - The recovery action is matched to the boundary's scope (see {@link ErrorBoundaryLevel}).
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
    state: ErrorBoundaryState = { error: null, showDetails: false };

    static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
        return { error };
    }

    componentDidCatch(error: Error, info: ErrorInfo) {
        console.error("ErrorBoundary captured", error, info);
    }

    private reset = () => this.setState({ error: null, showDetails: false });

    private handleReload = () => {
        window.location.reload();
    };

    private handleReset = () => {
        // Always clear our own state first so the children get a chance to
        // remount cleanly once the caller's reset runs.
        this.reset();
        this.props.onReset?.();
    };

    private toggleDetails = () => {
        this.setState((s) => ({ showDetails: !s.showDetails }));
    };

    render() {
        const { error, showDetails } = this.state;
        if (!error) return this.props.children;

        if (this.props.fallback) {
            return this.props.fallback(error, this.reset);
        }

        const level = this.props.level ?? "root";

        if (level === "leaf") {
            return (
                <VStack align="stretch" gap={1.5} py={2} w="full">
                    <Text fontSize="xs" color="fg.muted">
                        {this.props.description ?? "This section couldn't load."}
                    </Text>
                    <HStack gap={2}>
                        <Button size="xs" variant="ghost" onClick={this.reset}>
                            Try again
                        </Button>
                        <Button size="xs" variant="ghost" onClick={this.toggleDetails}>
                            {showDetails ? "Hide details" : "Show details"}
                        </Button>
                    </HStack>
                    {showDetails && this.renderDetails(error)}
                </VStack>
            );
        }

        const title =
            this.props.title ??
            (level === "route" ? "This view ran into a problem" : "Something went wrong");
        const description =
            this.props.description ??
            (level === "route"
                ? "You can close this view and keep using the app. Your library and files are unaffected."
                : "Clarity ran into an unexpected problem. Your library and files are unaffected.");

        // Render as a fixed full-screen overlay so the fallback covers whatever
        // was on screen when the error occurred (rather than flowing in below
        // sibling content like the media grid).
        return (
            <Box
                position="fixed"
                inset={0}
                zIndex="modal"
                bg="black"
                display="flex"
                alignItems="center"
                justifyContent="center"
                p={6}
                overflowY="auto"
            >
                <VStack gap={4} maxW="lg" textAlign="center" w="full">
                    <Heading size="md">{title}</Heading>
                    <Text fontSize="sm" color="fg.muted">
                        {description}
                    </Text>

                    <HStack gap={2}>
                        {level === "route" && this.props.onReset && (
                            <Button onClick={this.handleReset}>
                                {this.props.resetLabel ?? "Close"}
                            </Button>
                        )}
                        <Button
                            variant={level === "route" && this.props.onReset ? "outline" : "solid"}
                            onClick={this.handleReload}
                        >
                            Reload app
                        </Button>
                    </HStack>

                    <Button size="xs" variant="ghost" onClick={this.toggleDetails}>
                        {showDetails ? "Hide technical details" : "Show technical details"}
                    </Button>
                    {showDetails && this.renderDetails(error)}
                </VStack>
            </Box>
        );
    }

    private renderDetails(error: Error) {
        return (
            <Code
                as="pre"
                display="block"
                fontSize="xs"
                textAlign="left"
                w="full"
                p={3}
                bg="blackAlpha.400"
                borderRadius="md"
                whiteSpace="pre-wrap"
                wordBreak="break-word"
                maxH="40vh"
                overflowY="auto"
            >
                {formatError(error)}
                {error.stack ? `\n\n${error.stack}` : ""}
            </Code>
        );
    }
}
