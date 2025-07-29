import {Bookmark} from "@/types/bookmark.ts";

export type MediaItem = {
    path: string;
    title: string;
    type: "image" | "video";
    thumbnail_base64: string;
    length: number | null; // Duration for videos
    tags: string[]; // An array of tags
    bookmarks: Bookmark[]; // For videos, store bookmarks
};
