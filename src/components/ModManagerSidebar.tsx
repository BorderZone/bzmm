import React from 'react';
import ProfileList from './ProfileList';
import CategoryList from './CategoryList';
import type { Profile, Mod } from '../types/types';

interface ModManagerSidebarProps {
  profiles: Profile[];
  currentProfileIndex: number;
  setCurrentProfileIndex: (index: number) => void;
  refreshSettings: () => Promise<void>;
  categories: string[];
  mods: Mod[];  // Fixed any[] to Mod[]
  searchQuery: string;
  setSearchQuery: React.Dispatch<React.SetStateAction<string>>;
  selectedCategory: string;
  setSelectedCategory: React.Dispatch<React.SetStateAction<string>>;
}

const ModManagerSidebar: React.FC<ModManagerSidebarProps> = ({
  profiles,
  currentProfileIndex,
  setCurrentProfileIndex,
  refreshSettings,
  categories,
  mods,
  searchQuery,
  setSearchQuery,
  selectedCategory,
  setSelectedCategory,
}) => {
  // Get the current profile's repo URL
  const currentProfile = profiles[currentProfileIndex];
  const repoUrl = currentProfile?.repo_url || "";

  return (
    <aside className="w-64 flex flex-col gap-4 overflow-hidden">
      <div className="space-y-4">
        <ProfileList
          profiles={profiles}
          currentProfileIndex={currentProfileIndex}
          setCurrentProfileIndex={setCurrentProfileIndex}
          refreshSettings={refreshSettings}
        />
        <CategoryList
          categories={categories}
          mods={mods}
          searchQuery={searchQuery}
          setSearchQuery={setSearchQuery}
          selectedCategory={selectedCategory}
          setSelectedCategory={setSelectedCategory}
          repoUrl={repoUrl}
        />
      </div>
    </aside>
  );
};

export default ModManagerSidebar;