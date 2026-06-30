import { Button, Field, Input, Spinner, Stack, Text } from "@chakra-ui/react";
import { useInterfaceStore } from "@/stores/interfaceStore.ts";
import { useConfigStore } from "@/stores/configStore.ts";
import { AppDialog } from "@/components/common";
import { useRebuildLibrary } from "@/queries/library/useRebuildLibrary";

export const SettingsDialog = () => {
    const isSettingsDialogOpen = useInterfaceStore(s => s.isSettingsDialogOpen);
    const setSettingsDialogOpen = useInterfaceStore(s => s.setSettingsDialogOpen);

    const config = useConfigStore(s => s.config);
    const isLoading = useConfigStore(s => s.isLoading);
    const error = useConfigStore(s => s.error);
    const pickLibraryRoot = useConfigStore(s => s.pickLibraryRoot);

    const rebuild = useRebuildLibrary();

    return (
        <AppDialog
            open={isSettingsDialogOpen}
            onOpenChange={setSettingsDialogOpen}
            title="Settings"
        >
            <Stack gap={4}>
                <Text fontSize="sm" color="fg.muted">
                    Configure your media library settings.
                </Text>

                <Field.Root invalid={Boolean(error)}>
                    <Field.Label fontSize="sm" fontWeight="medium">
                        Library Folder
                    </Field.Label>

                    <Stack direction="row" gap={2} w="full">
                        <Input
                            value={
                                config?.libraryRoot ??
                                "No library selected. Choose a folder to continue."
                            }
                            disabled
                            size="sm"
                        />
                        <Button
                            size="sm"
                            variant="outline"
                            onClick={() => void pickLibraryRoot()}
                        >
                            {isLoading ? <Spinner /> : "Change..."}
                        </Button>
                    </Stack>

                    {error && <Field.ErrorText fontSize="xs">{error}</Field.ErrorText>}

                    <Field.HelperText fontSize="xs" color="fg.muted">
                        This is the root folder used for your media library. All
                        imports, thumbnails, and hard-linked views
                        will live under this directory.
                    </Field.HelperText>
                </Field.Root>

                <Field.Root>
                    <Field.Label fontSize="sm" fontWeight="medium">
                        Library Projection
                    </Field.Label>

                    <Button
                        size="sm"
                        variant="outline"
                        alignSelf="flex-start"
                        onClick={() => rebuild.mutate()}
                        loading={rebuild.isPending}
                        loadingText="Rebuilding..."
                    >
                        Rebuild library
                    </Button>

                    <Field.HelperText fontSize="xs" color="fg.muted">
                        Deletes and regenerates the entire <strong>Library</strong> folder tree from the database. 
                        Use this if the folders ever drift out of sync. Your originals in <strong> .objects</strong> are never touched.
                    </Field.HelperText>
                </Field.Root>
            </Stack>
        </AppDialog>
    );
};
