export type JobCompletedPayload = {
    jobType: string;
    mediaId: number | null;
    fileId: number | null;
    relPath: string | null;
    status: string;
};

export type WorkerStalledPayload = {
    errorMessage: string;
    consecutiveFailures: number;
};
