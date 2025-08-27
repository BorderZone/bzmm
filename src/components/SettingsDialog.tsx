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
import { Settings } from 'lucide-react';
import SettingsForm from './SettingsForm';
import { useSettings } from './hooks/useSettings';
import { useAppVersion } from './hooks/useAppVersion';

interface SettingsDialogProps {
  onSaved?: () => void;
}

const SettingsDialog = React.forwardRef<HTMLButtonElement, SettingsDialogProps>(({ onSaved }, ref) => {
  const [isOpen, setIsOpen] = useState(false);
  const { version } = useAppVersion();
  const {
    settings,
    isLoading,
    loadSettings,
    handleDownloadPathChange,
    handleSideloadPathChange,
    handleSave,
    handleCancel
  } = useSettings(onSaved);

  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen, loadSettings]);

  const onSave = async () => {
    await handleSave();
    setIsOpen(false);
  };

  const onCancel = () => {
    handleCancel();
    setIsOpen(false);
  };

  if (isLoading) {
    return null;
  }

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button ref={ref} variant="ghost" size="icon" className="h-8 w-8">
          <Settings className="h-4 w-4" />
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[650px]">
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Configure application settings.
          </DialogDescription>
        </DialogHeader>
        <div className="my-6">
          <SettingsForm
            downloadPath={settings.download_path}
            sideloadPath={settings.sideload_path}
            onDownloadPathChange={handleDownloadPathChange}
            onSideloadPathChange={handleSideloadPathChange}
          />
        </div>
        <div className="flex flex-col-reverse sm:flex-row sm:justify-between sm:items-center">
          <p className="text-xs text-muted-foreground mt-4 sm:mt-0">Version: {version}</p>
          <DialogFooter className="sm:space-x-2">
            <Button type="button" variant="secondary" onClick={onCancel}>
              Cancel
            </Button>
            <Button type="submit" onClick={onSave}>
              Save
            </Button>
          </DialogFooter>
        </div>
      </DialogContent>
    </Dialog>
  );
});

export default SettingsDialog;
