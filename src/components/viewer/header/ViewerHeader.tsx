import { Box, Button, HStack, IconButton, Text } from "@chakra-ui/react";
import { ArrowLeft, PanelRight, Check } from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip";
import { type ViewerMode } from "@/stores/mediaStore";
import { type MediaDetail } from "@/types/mediaTypes";
import { FileNameRename } from "./FileNameRename";
import { LoveButton } from "./LoveButton";
import { SIDEBAR_WIDTH, HEADER_HEIGHT } from "../constants";

interface ViewerHeaderProps {
    detail: MediaDetail | null;
    mode: ViewerMode;
    currentIndex: number;
    totalCount: number;
    visible: boolean;
    sidebarOpen: boolean;
    isReviewed: boolean;
    onClose: () => void;
    onToggleSidebar: () => void;
    onMarkReviewed: () => void;
    onRename: (fileId: number, newName: string) => Promise<void>;
    onLoveToggle: () => void;
    onRenameActiveChange: (active: boolean) => void;
}

const ReviewedBadge = () => (
    <HStack gap={1} px={2} py={1} borderRadius="md" bg="whiteAlpha.100" flexShrink={0}>
        <Check size={14} color="var(--chakra-colors-green-400)" />
        <Text fontSize="xs" color="green.400">Reviewed</Text>
    </HStack>
);

const PositionCounter = ({ currentIndex, totalCount }: { currentIndex: number; totalCount: number }) => (
    <Text fontSize="xs" color="gray.500" flexShrink={0}>
        {currentIndex + 1} / {totalCount}
    </Text>
);

export function ViewerHeader({
    detail,
    mode,
    currentIndex,
    totalCount,
    visible,
    sidebarOpen,
    isReviewed,
    onClose,
    onToggleSidebar,
    onMarkReviewed,
    onRename,
    onLoveToggle,
    onRenameActiveChange,
}: ViewerHeaderProps) {
    const file = detail?.files?.[0];
    const fileName = file?.fileName ?? "";
    const formatLabel = file?.ext?.toUpperCase() ?? "";

    return (
        <HStack
            position="absolute"
            top={0}
            left={0}
            right={sidebarOpen ? SIDEBAR_WIDTH : 0}
            h={HEADER_HEIGHT}
            px={3}
            bg="rgba(0, 0, 0, 0.7)"
            backdropFilter="blur(12px)"
            color="white"
            zIndex={20}
            opacity={visible ? 1 : 0}
            pointerEvents={visible ? "auto" : "none"}
            transition="opacity 0.3s ease, right 0.25s ease"
            gap={2}
        >
            <Tooltip content="Close">
                <IconButton
                    aria-label="Close viewer"
                    variant="ghost"
                    size="sm"
                    color="white"
                    onClick={onClose}
                    _hover={{ bg: "whiteAlpha.200" }}
                >
                    <ArrowLeft size={18} />
                </IconButton>
            </Tooltip>

            <HStack flex={1} minW={0} gap={2}>
                <FileNameRename
                    fileName={fileName}
                    fileId={file?.id}
                    onSubmit={onRename}
                    onActiveChange={onRenameActiveChange}
                />
                {formatLabel && (
                    <Box
                        px={1.5}
                        py={0.5}
                        borderRadius="sm"
                        bg="whiteAlpha.150"
                        fontSize="2xs"
                        fontWeight="bold"
                        color="gray.400"
                        flexShrink={0}
                        letterSpacing="0.5px"
                    >
                        {formatLabel}
                    </Box>
                )}
                <LoveButton loved={Boolean(detail?.loved)} onToggle={onLoveToggle} />
            </HStack>

            {totalCount > 1 && <PositionCounter currentIndex={currentIndex} totalCount={totalCount} />}

            {mode === "import" && (
                isReviewed ? (
                    <ReviewedBadge />
                ) : (
                    <Tooltip content="Mark as reviewed (R)">
                        <Button
                            size="xs"
                            colorPalette="blue"
                            onClick={onMarkReviewed}
                            flexShrink={0}
                        >
                            <Check size={14} />
                            Mark Reviewed
                        </Button>
                    </Tooltip>
                )
            )}

            <Tooltip content="Info panel (\)">
                <IconButton
                    aria-label="Toggle info panel"
                    variant="ghost"
                    size="sm"
                    color={sidebarOpen ? "blue.400" : "white"}
                    onClick={onToggleSidebar}
                    _hover={{ bg: "whiteAlpha.200" }}
                >
                    <PanelRight size={18} />
                </IconButton>
            </Tooltip>
        </HStack>
    );
}
