import { createRoot } from "react-dom/client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";
import "./index.css";
import App from "./App.tsx";
import { Provider } from "@/components/ui/provider.tsx";
import { ErrorBoundary } from "@/components/common";
import { logger } from "@/utils/logger";
import { notify } from "@/utils/notify";

// Catch-all for synchronous errors that escape React (timers, listeners, non-React libraries, etc.).
// Render-time errors are still handled by `ErrorBoundary`; this fires for everything else.
window.addEventListener("error", (e) => {
    logger.error("window-error", e.error ?? e.message);
});

// Promise rejections without a `.catch` land here.
// We log + toast because by definition the user-initiated path that triggered them is no longer in scope to surface the failure inline.
window.addEventListener("unhandledrejection", (e) => {
    logger.error("unhandled-rejection", e.reason);
    notify.error("Something went wrong", e.reason);
});

// Single QueryClient instance for the app lifetime.
const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            retry: 1,
            refetchOnWindowFocus: false,
            staleTime: 30_000,
        },
        mutations: {
            onError: (e) => notify.error("Action failed", e),
        },
    },
});

createRoot(document.getElementById("root")!).render(
    <QueryClientProvider client={queryClient}>
        <Provider>
            <ErrorBoundary name="root">
                <App />
            </ErrorBoundary>
        </Provider>
        {import.meta.env.DEV && <ReactQueryDevtools buttonPosition="bottom-left" />}
    </QueryClientProvider>
);
