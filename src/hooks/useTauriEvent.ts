import { useEffect, useRef } from "react";
import { listen, type EventCallback, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Subscribe to a Tauri event for the lifetime of the component.
 *
 * The handler is stored in a ref so it can be updated on every render without
 * causing the underlying `listen()` subscription to tear down and re-register.
 * This means callers don't need to memoize their handler or worry about
 * stale closures.
 */
export function useTauriEvent<T = unknown>(
    event: string,
    handler: EventCallback<T>,
): void {
    const handlerRef = useRef(handler);
    useEffect(() => {
        handlerRef.current = handler;
    });

    useEffect(() => {
        let unlisten: UnlistenFn | null = null;
        let cancelled = false;

        void listen<T>(event, (e) => {
            handlerRef.current(e);
        }).then((fn) => {
            if (cancelled) {
                fn();
            } else {
                unlisten = fn;
            }
        });

        return () => {
            cancelled = true;
            if (unlisten) unlisten();
        };
    }, [event]);
}
