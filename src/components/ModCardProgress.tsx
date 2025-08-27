import React from 'react';
import { Progress } from './ui/progress';
import { cn } from "../lib/utils";

interface ModCardProgressProps {
  modFileName?: string;
  progress: number;
  downloadedBytes?: number;
  totalBytes?: number;
  downloadSpeed?: number;
  isExtracting: boolean;
  isQueued?: boolean;
  onComplete?: () => void;
}

// Helper function to format bytes to KB, MB, GB
const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 B';
  
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${sizes[i]}`;
};

// Helper function to format download speed
const formatSpeed = (bytesPerSecond: number): string => {
  if (bytesPerSecond === 0) return '';
  
  if (bytesPerSecond < 1024) {
    return `${Math.round(bytesPerSecond)} B/s`;
  } else if (bytesPerSecond < 1024 * 1024) {
    return `${(bytesPerSecond / 1024).toFixed(1)} KB/s`;
  } else {
    return `${(bytesPerSecond / (1024 * 1024)).toFixed(1)} MB/s`;
  }
};

const ModCardProgress: React.FC<ModCardProgressProps> = ({ 
  progress,
  downloadedBytes,
  totalBytes,
  downloadSpeed = 0,
  isExtracting,
  isQueued = false
}) => {
  let statusText = '';
  
  if (isQueued) {
    statusText = 'Queued...';
  } else if (isExtracting) {
    statusText = 'Extracting...';
  } else {
    statusText = `${Math.round(progress)}% Complete`;
  }

  // Format the download size display
  const sizeDisplay = downloadedBytes !== undefined && totalBytes !== undefined 
    ? `${formatBytes(downloadedBytes)} / ${formatBytes(totalBytes)}`
    : '';
    
  // Format the download speed
  const speedDisplay = downloadSpeed > 0 ? ` • ${formatSpeed(downloadSpeed)}` : '';

  return (
    <div className="mt-3">
      <div className="flex justify-between text-xs text-muted-foreground mb-1">
        <span>{statusText}</span>
        {!isExtracting && !isQueued && progress > 0 && (
          <span>
            {sizeDisplay}{speedDisplay}
          </span>
        )}
      </div>
      <Progress 
        value={isExtracting || isQueued ? undefined : progress}
        className={cn(
          "h-1",
          (isExtracting || isQueued) && "progress-indeterminate"
        )}
      />
    </div>
  );
}

export default ModCardProgress;
