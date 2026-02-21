export interface ImportSkippedItem {
    mediaId: number;
    contentHash: string;
    fileName: string;
    originalImportFolder: string | null;
    originalRelPath: string | null;
    existingDirPath: string | null;
    existingRelPath: string | null;
}

export interface ImportResult {
    folderName: string;
    importedCount: number;
    skippedCount: number;
    skippedItems: ImportSkippedItem[];
}
