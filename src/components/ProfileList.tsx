import React, { useState, useEffect } from 'react';
import { Button } from './ui/button';
import { Plus } from 'lucide-react';
import type { Profile } from '../types/types';
import ProfileListItem from './ProfileListItem';
import DeleteProfileDialog from './DeleteProfileDialog';
import { useProfiles } from './hooks/useProfiles';

interface ProfileListProps {
  profiles: Profile[];
  currentProfileIndex: number;
  setCurrentProfileIndex: (index: number) => void;
  refreshSettings: () => void;
}

const ProfileList: React.FC<ProfileListProps> = ({
  profiles,
  currentProfileIndex,
  setCurrentProfileIndex,
  refreshSettings,
}) => {
  const [newProfileIndex, setNewProfileIndex] = useState<number | null>(null);

  const { addNewProfile, deleteProfile, updateProfile } = useProfiles(
    profiles,
    currentProfileIndex,
    setCurrentProfileIndex,
    refreshSettings
  );

  // Reset newProfileIndex when profiles array changes length
  useEffect(() => {
    if (newProfileIndex !== null && newProfileIndex >= profiles.length) {
      setNewProfileIndex(null);
    }
  }, [profiles.length]);

  const handleNewProfile = async () => {
    const index = await addNewProfile();
    // Set the new profile index after the profile has been created and loaded
    setNewProfileIndex(index);
  };

  const handleSettingsDialogOpenChange = (open: boolean) => {
    if (!open) {
      setNewProfileIndex(null);
    }
  };

  const currentProfile = profiles.length > 0 && currentProfileIndex >= 0 ? 
    profiles[currentProfileIndex] : 
    null;

  return (
    <div className="flex flex-col gap-2">
      <div className="flex justify-between items-center">
        <h2 className="font-semibold">Profiles</h2>
      </div>

      <div className="flex flex-col gap-1">
        {profiles.map((profile, index) => (
          <ProfileListItem
            key={index}
            profile={profile}
            index={index}
            isSelected={currentProfileIndex === index}
            onSelect={setCurrentProfileIndex}
            onUpdateProfile={updateProfile}
            isNewProfile={index === newProfileIndex}
            onSettingsDialogOpenChange={handleSettingsDialogOpenChange}
          />
        ))}
      </div>

      <div className="flex gap-2">
        <Button variant="outline" className="flex-1" onClick={handleNewProfile}>
          <Plus className="h-4 w-4 mr-1" />
          New
        </Button>
        {profiles.length > 0 && currentProfile && (
          <DeleteProfileDialog
            profileName={currentProfile.name}
            onDelete={deleteProfile}
          />
        )}
      </div>
    </div>
  );
};

export default ProfileList;