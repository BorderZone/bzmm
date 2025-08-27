import React, { useState, useEffect, useRef } from 'react';
import ModManagerContent from './ModManagerContent';
import ModManagerSidebar from './ModManagerSidebar';
import ModManagerWrapper from './ModManagerWrapper';
import { useModFiltering } from './hooks/useModFiltering';
import { useModActions } from './hooks/useModActions';
import { useModData } from './hooks/useModData';
import { useDownloadState } from './hooks/useDownloadState';
import { useProfiles } from './hooks/useProfiles';
import CriticalErrorBanner from './CriticalErrorBanner';
import SettingsDialog from './SettingsDialog';
import ProfileSettingsDialog from './ProfileSettingsDialog';

const ModManager: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [selectedCategory, setSelectedCategory] = useState<string>("All");
  const [globalError, setGlobalError] = useState<string | null>(null);
  const settingsDialogRef = useRef<HTMLButtonElement>(null);
  const [newProfileIndex, setNewProfileIndex] = useState<number | null>(null);
  const [profileSettingsOpen, setProfileSettingsOpen] = useState(false);
  
  // Get download state
  const downloadState = useDownloadState();
  
  // Use either global error or download error
  const displayError = downloadState.error || globalError;
  
  const { 
    mods,
    setMods,
    settings,
    loading,
    setDownloadedMods,
    loadSettings,
    loadMods,
    currentProfileIndex,
    setCurrentProfileIndex,
    modsError,
  } = useModData(setGlobalError);
  
  const { 
    addNewProfile,
    updateProfile
  } = useProfiles(
    settings?.profiles || [],
    currentProfileIndex,
    setCurrentProfileIndex,
    loadSettings
  );

  // Debug log for error state
  useEffect(() => {
    if (globalError) {
      console.log('Global error state:', globalError);
    }
  }, [globalError]);

  // Ensure currentProfileIndex is valid whenever settings change
  useEffect(() => {
    if (settings && currentProfileIndex >= settings.profiles.length) {
      setCurrentProfileIndex(0);
    }
  }, [settings, currentProfileIndex]);

  // Load mods whenever profile changes
  useEffect(() => {
    if (settings) {
      loadMods(currentProfileIndex);
    }
  }, [currentProfileIndex]);

  const { 
    handleDownload, 
    handleUpdate, 
    handleToggle,
    handleDelete
  } = useModActions(
    mods,
    setMods,
    setDownloadedMods,
    settings,
    currentProfileIndex,
    loadMods
  );

  const { categories, filteredMods } = useModFiltering(
    mods, 
    searchQuery, 
    selectedCategory
  );

  const formatDetailedError = (operation: string, mod: string | undefined, error: unknown) => {
    const errorString = error instanceof Error ? error.toString() : String(error);
    const details = [];
    
    // Add operation and mod info
    details.push(`Operation: ${operation}`);
    if (mod) {
      details.push(`Mod: ${mod}`);
    }
    
    // Add technical error details
    details.push(`Error: ${errorString}`);
    
    // Special handling for specific error types
    if (errorString.includes('permission denied')) {
      details.push('Note: This may be due to file system permissions. Check that the application has write access to the mod directory.');
    } else if (errorString.includes('already exists')) {
      details.push('Note: A file or directory with this name already exists. Try removing it first.');
    } else if (errorString.includes('network')) {
      details.push('Note: This appears to be a network error. Check your internet connection and any VPN settings.');
    }
    
    return details.join('\n');
  };

  // Wrap the handlers to catch errors
  const onDownload = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    try {
      await handleDownload(modId);
      setGlobalError(null);
    } catch (err) {
      setGlobalError(formatDetailedError('Download', mod?.name, err));
    }
  };

  const onUpdate = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    try {
      await handleUpdate(modId);
      setGlobalError(null);
    } catch (err) {
      setGlobalError(formatDetailedError('Update', mod?.name, err));
    }
  };

  const onToggle = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    try {
      await handleToggle(modId);
      setGlobalError(null);
    } catch (err) {
      setGlobalError(formatDetailedError('Toggle', mod?.name, err));
    }
  };
  
  const onDelete = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    try {
      await handleDelete(modId);
      setGlobalError(null);
    } catch (err) {
      setGlobalError(formatDetailedError('Delete', mod?.name, err));
    }
  };

  // Show critical error banner when there's a settings error
  if (!settings && globalError) {
    return (
      <div className="h-screen flex justify-center items-center p-4">
        <div className="max-w-2xl w-full">
          <CriticalErrorBanner
            error={globalError}
            title="Settings Error"
            onRetry={loadSettings}
          />
        </div>
      </div>
    );
  }

  // Handler to create a profile and open settings dialog
  const createAndConfigureProfile = async () => {
    try {
      const index = await addNewProfile();
      setNewProfileIndex(index);
      setProfileSettingsOpen(true);
      // Clear any error messages when creating a profile
      setGlobalError(null);
      return index;
    } catch (err) {
      setGlobalError(formatDetailedError('Create Profile', undefined, err));
      return -1;
    }
  };

  // Handler to open settings dialog programmatically
  const openSettingsDialog = () => {
    settingsDialogRef.current?.click();
    // Clear any error messages when opening settings
    setGlobalError(null);
  };

  // Get the currently selected profile or null if none exists
  const currentProfile = settings?.profiles.length && currentProfileIndex >= 0 ? 
    settings.profiles[currentProfileIndex] : 
    null;

  // Determine if we should show error or warnings
  const showError = !loading && modsError && settings &&
    !(settings.profiles.length === 0 || (settings.profiles.length > 0 && settings.download_path === ""));

  return (
    <ModManagerWrapper
      error={displayError}
      onDismissError={() => {
        setGlobalError(null);
        downloadState.setError(null);
      }}
    >
      {settings ? (
        <>
          <ModManagerSidebar
            profiles={settings.profiles}
            currentProfileIndex={settings.profiles.length > 0 ? currentProfileIndex : -1}
            setCurrentProfileIndex={setCurrentProfileIndex}
            refreshSettings={loadSettings}
            categories={categories}
            mods={mods}
            searchQuery={searchQuery}
            setSearchQuery={setSearchQuery}
            selectedCategory={selectedCategory}
            setSelectedCategory={setSelectedCategory}
          />

          <ModManagerContent
            profile={settings.profiles.length > 0 ? settings.profiles[currentProfileIndex] : { name: '', dcs_path: '', repo_url: '' }}
            settings={settings}
            profileName={settings.profiles.length > 0 ? settings.profiles[currentProfileIndex].name : 'No Profile'}
            selectedCategory={selectedCategory}
            filteredMods={filteredMods}
            error={showError ? modsError : null}
            loading={loading}
            onRefresh={() => loadMods(currentProfileIndex)}
            onDownload={onDownload}
            onUpdate={onUpdate}
            onToggle={onToggle}
            onDelete={onDelete}
            onAddProfile={createAndConfigureProfile}
            onOpenSettings={openSettingsDialog}
          />

          <div className="hidden">
            <SettingsDialog 
              ref={settingsDialogRef}
              onSaved={loadSettings}
            />
          </div>

          {currentProfile && newProfileIndex !== null && (
            <ProfileSettingsDialog 
              profile={settings.profiles[newProfileIndex]}
              onUpdateProfile={updateProfile}
              forceOpen={profileSettingsOpen}
              onOpenChange={(open) => {
                setProfileSettingsOpen(open);
                if (!open) setNewProfileIndex(null);
              }}
            />
          )}
        </>
      ) : null}
    </ModManagerWrapper>
  );
};

export default ModManager;