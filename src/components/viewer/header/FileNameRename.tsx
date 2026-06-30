import { Input, Text } from "@chakra-ui/react";
import { useCallback, useEffect, useRef, useState, type KeyboardEvent } from "react";

interface FileNameRenameProps {
    /** Current display name, no extension. */
    name: string;
    onSubmit: (newName: string) => Promise<void>;
    /** Lets the parent pause header autohide while a rename is in progress. */
    onActiveChange?: (active: boolean) => void;
}

/** Click-to-edit label for the item's logical (display-name) rename. */
export function FileNameRename({
    name,
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
        setValue(name);
        setRenaming(true);
    }, [name]);

    const cancel = useCallback(() => {
        setRenaming(false);
    }, []);

    const submit = useCallback(async () => {
        if (!value.trim() || value === name) {
            cancel();
            return;
        }
        await onSubmit(value.trim());
        setRenaming(false);
    }, [value, name, cancel, onSubmit]);

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
            {name || "Untitled"}
        </Text>
    );
}
