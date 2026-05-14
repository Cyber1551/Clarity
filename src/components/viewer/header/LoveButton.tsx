import { IconButton } from "@chakra-ui/react";
import { Heart } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";

interface LoveButtonProps {
    loved: boolean;
    onToggle: () => void;
}

/**
 * Heart toggle in the viewer header. Reflects the `loved` flag on the
 * current media detail and delegates the mutation to its parent.
 */
export function LoveButton({ loved, onToggle }: LoveButtonProps) {
    return (
        <Tooltip content="Love (F)">
            <IconButton
                aria-label="Toggle love"
                variant="ghost"
                size="xs"
                color={loved ? "red.400" : "gray.600"}
                onClick={onToggle}
                _hover={{ color: loved ? "red.300" : "gray.400" }}
                flexShrink={0}
            >
                <Heart size={15} fill={loved ? "currentColor" : "none"} />
            </IconButton>
        </Tooltip>
    );
}
