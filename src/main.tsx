import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.tsx";
import { Provider } from "@/components/ui/provider.tsx";
import { ErrorBoundary } from "@/components/common";

if (typeof Intl.DurationFormat === "undefined") {
    await import("@formatjs/intl-durationformat/polyfill.js");
}

createRoot(document.getElementById("root")!).render(
    <Provider>
        <ErrorBoundary>
            <App />
        </ErrorBoundary>
    </Provider>
);
