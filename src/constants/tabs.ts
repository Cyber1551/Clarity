export const TABS = [
    { value: "dashboard", label: "Dashboard" },
    { value: "imports", label: "Imports" },
    { value: "library", label: "Library" },
    { value: "people", label: "People" },
    //{ value: "explore", label: "Explore" },
    //{ value: "moments", label: "Moments" },
    //{ value: "session", label: "Sessions" },
] as const;

export type ActiveTab = (typeof TABS)[number]["value"];

export const DEFAULT_ACTIVE_TAB: ActiveTab = "library";

export function isActiveTab(value: string): value is ActiveTab {
    return TABS.some((t) => t.value === value);
}
