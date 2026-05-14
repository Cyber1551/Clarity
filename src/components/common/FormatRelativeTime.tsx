import { formatRelativeTime } from "@/utils/format";

interface FormatRelativeTimeProps {
    value: number;
}

export function FormatRelativeTime({ value }: FormatRelativeTimeProps) {
    return <>{formatRelativeTime(value)}</>;
}
