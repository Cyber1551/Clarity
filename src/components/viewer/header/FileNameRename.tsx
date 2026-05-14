import { Input, Text } from "@chakra-ui/react";
import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from "react";

interface FileNameRenameProps {
    fileName: string;
    /** Underlying media file id. When null the rename UI is read-only. */
    fileId: number | null | undefined;
    /** Renames the file via the appropriate backend mutation. */
    onSubmit: (fileId: number, newName: string) => Promise<void>;
    /** Notifies parents whether a rename is currently in progress (for autohide pause). */
    onActiveChange?: (active: boolean) => void;
}

/**
 * Click-to-edit filename label. Manages its own rename state machine and
 * delegates the actual mutation to its parent.
 */
export function FileNameRename({
    fileName,
    fileId,
    onSubmit,
    onActiveChange,
}: FileNameRenameProps) {
    const [renaming, setRenaming] = useState(false);
    const [value, setValue] = useState("");
    const inputRef = useRef<HTMLInputElement>(null);

    useEffect(() => {
        if (renaming && inputRef.current) {
            inputRef.current.focus();
            inputRef.current.select();
        }
    }, [renaming]);

    useEffect(() => {
        onActiveChange?.(renaming);
    }, [renaming, onActiveChange]);

    const start = useCallback(() => {
        setValue(fileName);
        setRenaming(true);
    }, [fileName]);

    const cancel = useCallback(() => {
        setRenaming(false);
    }, []);

    const submit = useCallback(async () => {
        if (!fileId || !value.trim() || value === fileName) {
            cancel();
            return;
        }
        await onSubmit(fileId, value.trim());
        setRenaming(false);
    }, [fileId, value, fileName, cancel, onSubmit]);

    const handleKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
        if (e.key === "Enter") {
            e.preventDefault();
            void submit();
        } else if (e.key === "Escape") {
            e.preventDefault();
            cancel();
        }
        e.stopPropagation();
    }, [submit, cancel]);

    if (renaming) {
        return (
            <Input
                ref={inputRef}
                value={value}
                onChange={(e) => setValue(e.target.value)}
                onKeyDown={handleKeyDown}
                onBlur={() => void submit()}
                size="sm"
                variant="flushed"
                color="white"
                fontSize="sm"
                fontWeight="medium"
                maxW="300px"
                _focus={{ borderColor: "blue.400" }}
            />
        );
    }

    return (
        <Text
            fontSize="sm"
            fontWeight="medium"
            truncate
            cursor="pointer"
            _hover={{ color: "blue.300" }}
            transition="color 0.15s"
            onClick={start}
            title="Click to rename"
        >
            {fileName || "Untitled"}
        </Text>
    );
}
