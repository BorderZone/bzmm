import React from 'react';
import ModList from './ModList';
import ModManagerHeader from './ModManagerHeader';
import LoadingState from './LoadingStates';
import ErrorBanner from './ErrorBanner';
import SetupWarning from './SetupWarning';
import type { Mod, Profile, Settings } from '../types/types';

interface ModManagerContentProps {
  profile: Profile;
  settings: Settings;
  profileName: string;
  selectedCategory: string;
  filteredMods: Mod[];
  error: string | null;
  loading: boolean;
  onRefresh: () => void;
  onDownload: (modId: number) => void;
  onUpdate: (modId: number) => void;
  onToggle: (modId: number) => void;
  onDelete?: (modId: number) => void;
  onAddProfile: () => Promise<number>;
  onOpenSettings: () => void;
}

const ModManagerContent: React.FC<ModManagerContentProps> = ({
  profile,
  settings,
  profileName,
  selectedCategory,
  filteredMods,
  error,
  loading,
  onRefresh,
  onDownload,
  onUpdate,
  onToggle,
  onDelete,
  onAddProfile,
  onOpenSettings,
}) => {
  const activeMods = filteredMods.filter(mod => mod.isEnabled);
  const inactiveMods = filteredMods.filter(mod => !mod.isEnabled);

  return (
    <main className="flex-1 flex flex-col overflow-hidden">
      <ModManagerHeader
        profileName={profileName}
        selectedCategory={selectedCategory}
        filteredModsCount={filteredMods.length}
        onRefresh={onRefresh}
      />
      <div className="flex-1 overflow-y-auto pt-4 px-4">
        {loading && (
          <div className="h-full flex items-center justify-center">
            <LoadingState message="Loading mods..." />
          </div>
        )}
        
        {error && (
          <div className="min-w-[200px] max-w-full">
            <ErrorBanner
              error={error}
              title="Error Loading Mods"
            >
              <div className="mt-2 text-sm opacity-70">
                Please check your repository URL in the profile settings.
              </div>
            </ErrorBanner>
          </div>
        )}

        {!loading && settings.profiles.length === 0 && (
          <SetupWarning
            title="No Profile Configured"
            description="You need to create a profile to manage your mods. A profile contains your DCS installation path and repository URL."
            actionLabel="Create Profile"
            onAction={onAddProfile}
          />
        )}

        {!loading && settings.profiles.length > 0 && settings.download_path === "" && (
          <SetupWarning
            title="Download Path Not Configured"
            description="You need to specify a download path for storing mods. This is where all mod files will be downloaded and organized."
            actionLabel="Configure Download Path"
            onAction={onOpenSettings}
          />
        )}

        {!loading && settings.profiles.length > 0 && settings.download_path !== "" && (
          <>
            {activeMods.length > 0 && (
              <ModList
                profile={profile}
                mods={activeMods}
                title="Active in this Profile"
                onDownload={onDownload}
                onUpdate={onUpdate}
                onToggle={onToggle}
                onDelete={onDelete}
                className="mb-4"
              />
            )}
            {inactiveMods.length > 0 && (
              <ModList
                profile={profile}
                mods={inactiveMods}
                title="Available Mods"
                onDownload={onDownload}
                onUpdate={onUpdate}
                onToggle={onToggle}
                onDelete={onDelete}
              />
            )}
          </>
        )}
      </div>
    </main>
  );
};

export default ModManagerContent;