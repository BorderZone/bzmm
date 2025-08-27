import { invoke } from '@tauri-apps/api/core';
import type { Profile } from '../../types/types';

export function useProfiles(
  profiles: Profile[],
  currentProfileIndex: number,
  setCurrentProfileIndex: (index: number) => void,
  refreshSettings: () => void
) {
  const addNewProfile = async () => {
    const newProfileIndex = profiles.length;
    const newProfile: Profile = {
      name: `Profile ${newProfileIndex + 1}`,
      dcs_path: '',
      repo_url: 'https://repo.borderzone.ca/BZ_Saved_Mods.xml', // Add default repo URL
    };
    
    await invoke('update_profile', { 
      index: newProfileIndex,
      profile: newProfile
    });
    
    await refreshSettings();
    // Set the current profile index to the new profile
    setCurrentProfileIndex(newProfileIndex);
    
    // Return the index of the new profile
    return newProfileIndex;
  };

  const deleteProfile = async () => {
    try {
      await invoke('delete_profile', {
        index: currentProfileIndex
      });
      // First refresh settings to get the updated profiles list
      await refreshSettings();
      // Only set index to 0 if there are any profiles left
      if (profiles.length > 1) {
        setCurrentProfileIndex(0);
      }
    } catch (error) {
      console.error('Failed to delete profile:', error);
    }
  };

  const updateProfile = async (updatedProfile: Profile) => {
    await invoke('update_profile', {
      index: currentProfileIndex,
      profile: updatedProfile
    });
    await refreshSettings();
  };

  return {
    addNewProfile,
    deleteProfile,
    updateProfile
  };
}

export default useProfiles;
