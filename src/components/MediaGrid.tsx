// src/MediaGrid.tsx
import React from 'react';
import {MediaItem} from "@/types/media_item.ts";
import VideoItem from "@/components/VideoItem.tsx";
import { convertFileSrc } from '@tauri-apps/api/tauri';

interface MediaGridProps {
    mediaItems: MediaItem[];
}

const MediaGrid: React.FC<MediaGridProps> = ({ mediaItems }) => {
    return (
        <div
            className="media-grid"
            style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
                gap: '1rem',
            }}
        >
            {mediaItems.map((item) => (
                <div
                    key={item.path}
                    style={{
                        border: '1px solid #ccc',
                        borderRadius: '4px',
                        padding: '0.5rem',
                    }}
                >
                    <div style={{ height: '150px', overflow: 'hidden' }}>
                        {item.type === 'image' ? (
                            <img
                                src={convertFileSrc(item.path)}
                                alt={item.title}
                                style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                            />
                        ) : (
                            <VideoItem videoPath={item.path} title={item.title} />
                        )}
                    </div>
                    <div style={{ marginTop: '0.5rem' }}>
                        <h4 style={{ fontSize: '1rem', margin: '0 0 0.5rem' }}>
                            {item.title}
                        </h4>
                        <p style={{ margin: 0 }}>Type: {item.type}</p>
                        {item.type === 'video' && item.length && (
                            <p style={{ margin: 0 }}>Length: {item.length} sec</p>
                        )}
                    </div>
                </div>
            ))}
        </div>
    );
};

export default MediaGrid;
