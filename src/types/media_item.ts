import {Bookmark} from "@/types/bookmark.ts";

export type MediaItem = {
    path: string;
    title: string;
    type: "image" | "video";
    thumbnail: string | null;
    length: number | null; // Duration for videos
    tags: string[]; // An array of tags
    bookmarks: Bookmark[]; // For videos, store bookmarks
};
