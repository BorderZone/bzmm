import React, { useState } from 'react';
import ModCardHeader from './ModCardHeader';
import ModCardActions from './ModCardActions';
import ModCardDescription from './ModCardDescription';
import ModCardProgress from './ModCardProgress';
import { useDownloadContext } from './context/DownloadContext';
import styles from './ModList.module.css';
import type { Mod, Profile } from '../types/types';

interface ModCardProps {
  mod: Mod;
  profile: Profile;
  onDownload: (modId: number) => void;
  onUpdate: (modId: number) => void;
  onToggle: (modId: number) => void;
  onDelete?: (modId: number) => void;
  className?: string;
}

const ModCard: React.FC<ModCardProps> = ({ 
  mod,
  profile,
  onDownload, 
  onUpdate, 
  onToggle,
  onDelete,
  className = ""
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);
  const [wasActive, setWasActive] = useState(false);

  const { state, cancelDownload } = useDownloadContext();
  // Add support for both .zip and unchanged filename formats
  const zipFilename = `${mod.name}.zip`;
  const plainFilename = mod.name;
  
  // Check for the mod in both formats
  const isDownloading = state.downloading.has(zipFilename) || state.downloading.has(plainFilename);
  const isExtracting = state.extracting.has(zipFilename) || state.extracting.has(plainFilename);
  const isQueued = state.queued.has(zipFilename) || state.queued.has(plainFilename);

  // Debug logging
  console.log('ModCard received mod:', {
    name: mod.name,
    version: mod.version,
    newVersion: mod.newVersion,
    isDownloading,
    isExtracting,
    isQueued
  });

  const handleToggle = () => {
    setIsAnimating(true);
    setWasActive(false);
    
    setTimeout(() => {
      onToggle(mod.id);
      setIsAnimating(false);
    }, 300);
  };

  const handleDownload = () => {
    console.log('Starting download for mod:', mod.name);
    onDownload(mod.id);
  };

  const handleUpdate = () => {
    console.log('Starting update for mod:', mod.name);
    onUpdate(mod.id);
  };
  
  const handleDelete = () => {
    if (onDelete) {
      onDelete(mod.id);
    }
  };

  const handleCancel = () => {
    console.log('Cancelling download for mod:', mod.name);
    const filename = `${mod.name}.zip`;
    cancelDownload(filename);
  };

  // Add special styling for deprecated mods
  const isDeprecated = mod.category === "Deprecated";
  
  const cardClassName = `
    bg-card text-card-foreground border rounded-lg p-3
    ${className}
    ${isAnimating ? (wasActive ? styles.modCardExit : styles.modCardEnter) : ''}
    ${isDeprecated ? 'border-yellow-500 border-2' : ''}
  `.trim();

  // Show progress for any download, extraction, or update operation
  const showProgress = isDownloading || isExtracting || isQueued;
  
  // Get download stats - try both filename formats
  const downloadStats = state.progress.get(zipFilename) || state.progress.get(plainFilename);
  
  // Use undefined fallbacks to avoid errors
  const progress = downloadStats?.progress ?? 0;
  const downloadedBytes = downloadStats?.downloadedBytes;
  const totalBytes = downloadStats?.totalBytes;
  const downloadSpeed = downloadStats?.speed ?? 0;

  return (
    <div key={mod.id} className={cardClassName}>
      <div className="flex items-start gap-4">
        <ModCardHeader mod={mod} />
        <ModCardActions
          mod={mod}
          profile={profile}
          isExpanded={isExpanded}
          onUpdate={handleUpdate}
          onDownload={handleDownload}
          onToggle={handleToggle}
          onDelete={onDelete ? handleDelete : undefined}
          onCancel={handleCancel}
          onExpandClick={() => setIsExpanded(!isExpanded)}
          isDownloading={isDownloading}
          isExtracting={isExtracting}
          isQueued={isQueued}
        />
      </div>

      {showProgress && (
        <ModCardProgress 
          modFileName={zipFilename}
          progress={progress}
          downloadedBytes={downloadedBytes}
          totalBytes={totalBytes}
          downloadSpeed={downloadSpeed}
          isExtracting={isExtracting}
          isQueued={isQueued}
        />
      )}
      {isExpanded && (
        <ModCardDescription description={mod.description} />
      )}
    </div>
  );
};

export default ModCard;
