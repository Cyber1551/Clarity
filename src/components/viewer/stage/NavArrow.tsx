import { Box } from "@chakra-ui/react";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { MouseEvent } from "react";

interface NavArrowProps {
    direction: "prev" | "next";
    onClick: () => void;
    /** Override the default `bottom` so video controls remain visible. */
    bottomOffset?: string | number;
    /** z-index override; defaults to 2 (over images), pass 10 for video. */
    zIndex?: number;
}

export function NavArrow({
    direction,
    onClick,
    bottomOffset = 0,
    zIndex = 2,
}: NavArrowProps) {
    const isLeft = direction === "prev";
    const handleClick = (e: MouseEvent) => {
        e.stopPropagation();
        onClick();
    };

    return (
        <Box
            position="absolute"
            top="0"
            bottom={bottomOffset}
            {...(isLeft ? { left: 0 } : { right: 0 })}
            w="60px"
            display="flex"
            alignItems="center"
            justifyContent="center"
            cursor="pointer"
            opacity={0}
            _hover={{ opacity: 1 }}
            transition="opacity 0.2s"
            onClick={handleClick}
            zIndex={zIndex}
            aria-label={isLeft ? "Previous" : "Next"}
            role="button"
        >
            <Box
                bg="blackAlpha.600"
                borderRadius="full"
                p={2}
                color="white"
                _hover={{ bg: "blackAlpha.800" }}
                transition="background 0.15s"
            >
                {isLeft ? <ChevronLeft size={24} /> : <ChevronRight size={24} />}
            </Box>
        </Box>
    );
}
