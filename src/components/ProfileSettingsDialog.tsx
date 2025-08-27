import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Settings } from 'lucide-react';
import { Profile } from '../types/types';
import FormField from './FormField';
import DirectoryPicker from './DirectoryPicker';
import { useProfileSettings } from './hooks/useProfileSettings';

interface ProfileSettingsDialogProps {
  profile: Profile;
  onUpdateProfile: (updatedProfile: Profile) => void;
  forceOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
}

const ProfileSettingsDialog: React.FC<ProfileSettingsDialogProps> = ({
  profile,
  onUpdateProfile,
  forceOpen,
  onOpenChange,
}) => {
  const [internalOpen, setInternalOpen] = useState(false);

  const { 
    settings, 
    setSettings, 
    handleDcsDirectoryChange,
    handleSave,
    handleCancel 
  } = useProfileSettings(profile, onUpdateProfile);

  useEffect(() => {
    if (forceOpen !== undefined) {
      setInternalOpen(forceOpen);
    }
  }, [forceOpen]);

  useEffect(() => {
    setSettings({
      name: profile.name,
      dcs_path: profile.dcs_path,
      repo_url: profile.repo_url
    });
  }, [profile]);

  const onSave = () => {
    handleSave();
    setInternalOpen(false);
    onOpenChange?.(false);
  };

  const onCancel = () => {
    handleCancel();
    setInternalOpen(false);
    onOpenChange?.(false);
  };

  const handleOpenChange = (open: boolean) => {
    setInternalOpen(open);
    onOpenChange?.(open);
  };

  return (
    <Dialog open={internalOpen} onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <Button variant="ghost" size="icon" className="h-8 w-8">
          <Settings className="h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[650px]">
        <DialogHeader>
          <DialogTitle>Profile Settings</DialogTitle>
          <DialogDescription>
            Configure profile-specific settings.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-6 py-6">
          <FormField 
            label="Name" 
            htmlFor="profile-name"
            tooltip="A descriptive name for this profile. Use a name that helps you identify the purpose of the profile."
          >
            <Input
              id="profile-name"
              value={settings.name}
              onChange={e => setSettings(prev => ({ ...prev, name: e.target.value }))}
              className="text-foreground"
            />
          </FormField>
          <FormField 
            label="Repository URL" 
            htmlFor="repo-url"
            tooltip="URL to the mod repository XML file. This is where the mod manager will download the list of available mods from."
          >
            <Input
              id="repo-url"
              value={settings.repo_url}
              onChange={e => setSettings(prev => ({ ...prev, repo_url: e.target.value }))}
              placeholder="Enter URL"
              className="text-foreground"
            />
          </FormField>
          <FormField 
            label="DCS Path" 
            htmlFor="dcs-path"
            tooltip="Path where mods will be activated into. This will be either your DCS Saved Games Directory, or your DCS install directory."
          >
            <DirectoryPicker
              id="dcs-path"
              value={settings.dcs_path}
              onBrowse={handleDcsDirectoryChange}
            />
          </FormField>
        </div>
        <DialogFooter>
          <Button type="button" variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit" onClick={onSave}>
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export default ProfileSettingsDialog;
