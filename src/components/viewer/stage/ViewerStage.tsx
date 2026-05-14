import { type MediaDetail } from "@/types/mediaTypes";
import { ImageStage } from "./ImageStage";
import { VideoStage } from "./VideoStage";

interface ViewerStageProps {
    detail: MediaDetail | null;
    src: string;
    loading: boolean;
    error: string | null;
    canGoPrev: boolean;
    canGoNext: boolean;
    onPrev: () => void;
    onNext: () => void;
    onRetry: () => void;
    transitioning: boolean;
}

/**
 * Picks the right stage component for the focused media item.
 * Falls back to the image stage for unknown/unset media types so we
 * always render a friendly loading or error state.
 */
export function ViewerStage({
    detail,
    src,
    loading,
    error,
    canGoPrev,
    canGoNext,
    onPrev,
    onNext,
    onRetry,
    transitioning,
}: ViewerStageProps) {
    if (detail?.mediaType === "video") {
        return (
            <VideoStage
                src={src}
                loading={loading}
                canGoPrev={canGoPrev}
                canGoNext={canGoNext}
                onPrev={onPrev}
                onNext={onNext}
            />
        );
    }

    return (
        <ImageStage
            src={src}
            alt={detail?.contentHash ?? ""}
            loading={loading}
            error={error}
            canGoPrev={canGoPrev}
            canGoNext={canGoNext}
            onPrev={onPrev}
            onNext={onNext}
            onRetry={onRetry}
            transitioning={transitioning}
        />
    );
}
