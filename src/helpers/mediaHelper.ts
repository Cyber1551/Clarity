// Valid file extensions for images and videos.
const validImageExtensions = ['jpg', 'jpeg', 'png', 'gif', 'bmp', 'tiff', 'svg', 'webp'];
const validVideoExtensions = ['mp4', 'mov', 'avi', 'mkv', 'webm', 'flv', 'wmv'];

export function isImageFile(fileName: string): boolean {
    const ext = fileName.split('.').pop()?.toLowerCase();
    return ext ? validImageExtensions.includes(ext) : false;
}

export function isVideoFile(fileName: string): boolean {
    const ext = fileName.split('.').pop()?.toLowerCase();
    return ext ? validVideoExtensions.includes(ext) : false;
}
