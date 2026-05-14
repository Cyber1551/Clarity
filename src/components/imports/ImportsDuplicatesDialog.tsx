import { Text, VStack } from "@chakra-ui/react";
import { AppDialog } from "@/components/common";
import { type ImportSkippedItem } from "@/types/importTypes";
import { ImportsDuplicateRow } from "./ImportsDuplicateRow";

interface ImportsDuplicatesDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    skippedItems: ImportSkippedItem[];
    onJump: (item: ImportSkippedItem) => void;
}

export function ImportsDuplicatesDialog({
    open,
    onOpenChange,
    skippedItems,
    onJump,
}: ImportsDuplicatesDialogProps) {
    return (
        <AppDialog
            open={open}
            onOpenChange={onOpenChange}
            title="Duplicates skipped"
            maxH="90vh"
            bodyProps={{ overflowY: "auto" }}
        >
            {skippedItems.length ? (
                <VStack align="stretch" gap={4}>
                    {skippedItems.map(item => (
                        <ImportsDuplicateRow
                            key={`${item.mediaId}-${item.contentHash}`}
                            item={item}
                            onJump={onJump}
                        />
                    ))}
                </VStack>
            ) : (
                <Text fontSize="sm" color="gray.500">No duplicates to show.</Text>
            )}
        </AppDialog>
    );
}
