import { useEffect, useState } from "react";
import { BlobWithMime } from "@/types/binaryTypes.ts";

type UseObjectUrlFromBlobOptions = {
    enabled?: boolean;
    onError?: (error: unknown) => void;
};

/**
 * Turns an async `{ blob, mimetype }` loader into an object URL with safe cleanup.
 * - Revokes old URLs on dependency changes/unmount.
 * - Prevents state updates after unmount.
 */
export function useObjectUrlFromBlob(
    load: () => Promise<BlobWithMime>,
    deps: readonly unknown[],
    options: UseObjectUrlFromBlobOptions = {},
) {
    const { enabled = true, onError } = options;
    const [url, setUrl] = useState<string | null>(null);

    useEffect(() => {
        let active = true;
        let objectUrl: string | null = null;

        async function run() {
            if (!enabled) {
                setUrl(null);
                return;
            }

            try {
                const { blob, mimetype } = await load();
                if (!active) return;

                const newUrl = URL.createObjectURL(new Blob([blob], { type: mimetype }));

                if (!active) {
                    URL.revokeObjectURL(newUrl);
                    return;
                }

                objectUrl = newUrl;
                setUrl(newUrl);
            } catch (e) {
                onError?.(e);
            }
        }

        void run();

        return () => {
            active = false;
            if (objectUrl) URL.revokeObjectURL(objectUrl);
        };
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [enabled, ...deps]);

    return url;
}