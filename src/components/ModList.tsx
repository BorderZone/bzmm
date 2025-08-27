import React from 'react';
import { Mod, Profile } from '../types/types';
import ModCard from './ModCard';
import styles from './ModList.module.css';

interface ModListProps {
  profile: Profile;
  mods: Mod[];
  title: string;
  onDownload: (modId: number) => void;
  onUpdate: (modId: number) => void;
  onToggle: (modId: number) => void;
  onDelete?: (modId: number) => void;
  className?: string;
}

const ModList: React.FC<ModListProps> = ({
  profile,
  mods,
  title,
  onDownload,
  onUpdate,
  onToggle,
  onDelete,
  className = "",
}) => {
  if (mods.length === 0) return null;

  return (
    <>
      <h2 className="font-medium text-sm text-gray-500 mb-2">{title}</h2>
      <div className={`grid gap-2 ${styles.modList} ${className}`}>
        {mods.map(mod => (
          <ModCard
            key={mod.id}
            profile={profile}
            mod={mod}
            onDownload={onDownload}
            onUpdate={onUpdate}
            onToggle={onToggle}
            onDelete={onDelete}
            className={styles.modCard}
          />
        ))}
      </div>
    </>
  );
};

export default ModList;