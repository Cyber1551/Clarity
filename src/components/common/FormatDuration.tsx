import { formatDuration } from "@/utils/format";

interface FormatDurationProps {
    value: number;
    style?: "long" | "short" | "narrow" | "digital";
}

export function FormatDuration({ value, style }: FormatDurationProps) {
    return <>{formatDuration(value, style ?? "narrow")}</>;
}
