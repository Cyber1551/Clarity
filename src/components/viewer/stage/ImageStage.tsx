import { Box, Image } from "@chakra-ui/react";
import { NavArrow } from "./NavArrow";
import { StageStatus } from "./StageStatus";

interface ImageStageProps {
    src: string;
    alt: string;
    loading: boolean;
    error: string | null;
    canGoPrev: boolean;
    canGoNext: boolean;
    onPrev: () => void;
    onNext: () => void;
    onRetry: () => void;
    transitioning: boolean;
}

export function ImageStage({
    src,
    alt,
    loading,
    error,
    canGoPrev,
    canGoNext,
    onPrev,
    onNext,
    onRetry,
    transitioning,
}: ImageStageProps) {
    if (loading || error) {
        return <StageStatus loading={loading} error={error} onRetry={onRetry} />;
    }

    return (
        <Box
            flex={1}
            position="relative"
            display="flex"
            alignItems="center"
            justifyContent="center"
            bg="black"
            overflow="hidden"
        >
            <Image
                src={src}
                alt={alt}
                maxH="100%"
                maxW="100%"
                objectFit="contain"
                opacity={transitioning ? 0 : 1}
                transition="opacity 0.15s ease"
            />
            {canGoPrev && <NavArrow direction="prev" onClick={onPrev} />}
            {canGoNext && <NavArrow direction="next" onClick={onNext} />}
        </Box>
    );
}
