export interface JobCompletedPayload {
    jobType: string;
    mediaId: number | null;
    fileId: number | null;
    relPath: string | null;
    status: string;
}

export interface WorkerStalledPayload {
    errorMessage: string;
    consecutiveFailures: number;
}
