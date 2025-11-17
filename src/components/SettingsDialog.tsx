import { Button, Dialog, Input, Portal, Spinner, Stack, Text } from "@chakra-ui/react";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { useConfigStore } from "@/stores/configStore.ts";

export const SettingsDialog = () => {
    const isSettingsDialogOpen = useInterfaceStore(s => s.isSettingsDialogOpen);
    const setSettingsDialogOpen = useInterfaceStore(s => s.setSettingsDialogOpen);

    const config = useConfigStore(s => s.config);
    const isLoading = useConfigStore(s => s.isLoading);
    const error = useConfigStore(s => s.error);
    const pickLibraryRoot = useConfigStore(s => s.pickLibraryRoot);

    return (
        <Dialog.Root
            placement="center"
            motionPreset="slide-in-bottom"
            open={isSettingsDialogOpen}
            onOpenChange={({ open }) => setSettingsDialogOpen(open)}
        >
            <Portal>
                <Dialog.Backdrop />
                <Dialog.Positioner>
                    <Dialog.Content maxW="lg">
                        <Dialog.Header>
                            <Dialog.Title>Settings</Dialog.Title>
                        </Dialog.Header>

                        <Dialog.Body>
                            <Stack gap={4}>
                                <Text fontSize="sm" color="fg.muted">
                                    Configure your media library settings.
                                </Text>

                                <Stack gap={2}>
                                    <Text as="label" fontSize="sm" fontWeight="medium">
                                        Library Folder
                                    </Text>

                                    <Stack direction="row" gap={2}>
                                        <Input
                                            id="library-folder"
                                            value={
                                                config?.libraryRoot ||
                                                "No library selected. Choose a folder to continue."
                                            }
                                            disabled={true}
                                            size="sm"
                                        />
                                        <Button
                                            size="sm"
                                            variant="outline"
                                            onClick={async () => await pickLibraryRoot()}
                                        >
                                            {isLoading ? <Spinner /> : "Change..."}
                                        </Button>
                                    </Stack>

                                    {error && (
                                        <Text fontSize="xs" color="red.500">
                                            {error}
                                        </Text>
                                    )}

                                    <Text fontSize="xs" color="fg.muted">
                                        This is the root folder used for your media library. All
                                        scans, tags, thumbnails, and hard-linked views (by tag / set)
                                        will live under this directory.
                                    </Text>
                                </Stack>
                            </Stack>
                        </Dialog.Body>

                        <Dialog.Footer>
                            <Dialog.ActionTrigger asChild>
                                <Button variant="outline">Close</Button>
                            </Dialog.ActionTrigger>
                        </Dialog.Footer>
                    </Dialog.Content>
                </Dialog.Positioner>
            </Portal>
        </Dialog.Root>
    )
}

