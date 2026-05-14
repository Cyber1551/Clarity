import { Button, Dialog, Portal } from "@chakra-ui/react";
import type { ReactNode } from "react";

interface AppDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    title: ReactNode;
    children: ReactNode;
    /** Optional footer content. If omitted, a single "Close" button is rendered. */
    footer?: ReactNode;
    size?: Dialog.ContentProps["maxW"];
    maxH?: Dialog.ContentProps["maxH"];
    bodyProps?: Dialog.BodyProps;
    placement?: "center" | "top" | "bottom";
    motionPreset?: "slide-in-bottom" | "slide-in-top" | "scale" | "none";
}

const defaultFooter = (
    <Dialog.ActionTrigger asChild>
        <Button variant="outline">Close</Button>
    </Dialog.ActionTrigger>
);

export function AppDialog({
    open,
    onOpenChange,
    title,
    children,
    footer,
    size = "lg",
    maxH,
    bodyProps,
    placement = "center",
    motionPreset = "slide-in-bottom",
}: AppDialogProps) {
    return (
        <Dialog.Root
            placement={placement}
            motionPreset={motionPreset}
            open={open}
            onOpenChange={(e) => onOpenChange(e.open)}
        >
            <Portal>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content maxW={size} maxH={maxH}>
                        <Dialog.Header>
                            <Dialog.Title>{title}</Dialog.Title>
                        </Dialog.Header>
                        <Dialog.Body {...bodyProps}>{children}</Dialog.Body>
                        <Dialog.Footer>{footer ?? defaultFooter}</Dialog.Footer>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    );
}
