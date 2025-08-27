import React from 'react';
import { Button } from './ui/button';
import ProfileSettingsDialog from './ProfileSettingsDialog';
import type { Profile } from '../types/types';

interface ProfileListItemProps {
  profile: Profile;
  index: number;
  isSelected: boolean;
  onSelect: (index: number) => void;
  onUpdateProfile: (profile: Profile) => void;
  isNewProfile?: boolean;
  onSettingsDialogOpenChange?: (open: boolean) => void;
}

const ProfileListItem: React.FC<ProfileListItemProps> = ({
  profile,
  index,
  isSelected,
  onSelect,
  onUpdateProfile,
  isNewProfile,
  onSettingsDialogOpenChange,
}) => {
  return (
    <div
      className={`flex items-center gap-2 p-2 rounded ${
        isSelected ? 'bg-muted' : ''
      }`}
    >
      <Button
        variant="ghost"
        className="flex-1 justify-start h-8 font-normal"
        onClick={() => onSelect(index)}
      >
        {profile.name}
      </Button>
      <ProfileSettingsDialog
        profile={profile}
        onUpdateProfile={onUpdateProfile}
        forceOpen={isNewProfile}
        onOpenChange={onSettingsDialogOpenChange}
      />
    </div>
  );
};

export default ProfileListItem;