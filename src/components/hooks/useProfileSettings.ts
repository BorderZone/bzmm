import { useState, useEffect } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import type { Profile } from '../../types/types';

interface ProfileSettingsData {
  name: string;
  dcs_path: string;
  repo_url: string;
}

export function useProfileSettings(
  profile: Profile,
  onUpdateProfile: (updatedProfile: Profile) => void
) {
  const [settings, setSettings] = useState<ProfileSettingsData>({
    name: profile.name,
    dcs_path: profile.dcs_path || '',
    repo_url: profile.repo_url || 'https://repo.borderzone.ca/BZ_Saved_Mods.xml',
  });

  useEffect(() => {
    setSettings({
      name: profile.name,
      dcs_path: profile.dcs_path || '',
      repo_url: profile.repo_url || 'https://repo.borderzone.ca/BZ_Saved_Mods.xml',
    });
  }, [profile]);

  const handleDcsDirectoryChange = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected) {
        setSettings(prev => ({ ...prev, dcs_path: selected as string }));
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  const handleSave = async () => {
    onUpdateProfile({
      ...profile,
      name: settings.name,
      dcs_path: settings.dcs_path,
      repo_url: settings.repo_url,
    });
  };

  const handleCancel = () => {
    setSettings({
      name: profile.name,
      dcs_path: profile.dcs_path || '',
      repo_url: profile.repo_url || 'https://repo.borderzone.ca/BZ_Saved.xml',
    });
  };

  return {
    settings,
    setSettings,
    handleDcsDirectoryChange,
    handleSave,
    handleCancel
  };
}

export default useProfileSettings;
