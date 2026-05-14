import React from "react";
import { Box } from "@chakra-ui/react";
import { NavArrow } from "./NavArrow";
import { StageStatus } from "./StageStatus";

interface VideoStageProps {
    src: string;
    loading: boolean;
    canGoPrev: boolean;
    canGoNext: boolean;
    onPrev: () => void;
    onNext: () => void;
}

export function VideoStage({ src, loading, canGoPrev, canGoNext, onPrev, onNext }: VideoStageProps) {
    if (loading) {
        return <StageStatus loading />;
    }

    return (
        <Box flex={1} position="relative" bg="black" overflow="hidden">
            <video
                src={src}
                controls
                autoPlay={false}
                onContextMenu={(e: React.MouseEvent) => e.preventDefault()}
                style={{ width: "100%", height: "100%" }}
            />
            {canGoPrev && <NavArrow direction="prev" onClick={onPrev} bottomOffset="60px" zIndex={10} />}
            {canGoNext && <NavArrow direction="next" onClick={onNext} bottomOffset="60px" zIndex={10} />}
        </Box>
    );
}
