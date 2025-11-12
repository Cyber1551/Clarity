import React from 'react';

type Props = {
    videoPath: string;
    title: string;
}

const VideoItem: React.FC<Props> = ({ title }) => {
    return (
        <div style={{ border: '1px solid #ccc', padding: '0.5rem', borderRadius: '4px' }}>
            <h4>{title}</h4>
        </div>
    );
};

export default VideoItem;
