import React from 'react';
import { ThemeToggle } from './theme/theme-toggle';
import FormField from './FormField';
import DirectoryPicker from './DirectoryPicker';

interface SettingsFormProps {
  downloadPath: string;
  sideloadPath: string;
  onDownloadPathChange: () => void;
  onSideloadPathChange: () => void;
}

const SettingsForm: React.FC<SettingsFormProps> = ({
  downloadPath,
  sideloadPath,
  onDownloadPathChange,
  onSideloadPathChange,
}) => {
  return (
    <div className="grid gap-4 py-4">
      <FormField 
        label="Theme" 
        htmlFor="theme"
        tooltip="Switch between light and dark mode for the application interface."
      >
        <ThemeToggle />
      </FormField>

      <FormField 
        label="Download Path" 
        htmlFor="download-path"
        tooltip="Directory where mod files will be downloaded and decompressed into for storage."
      >
        <DirectoryPicker
          id="download-path"
          value={downloadPath}
          onBrowse={onDownloadPathChange}
        />
      </FormField>

      <FormField 
        label="Sideload Path" 
        htmlFor="sideload-path"
        tooltip="Directory where manually downloaded mods are stored for installation. Use this for mods not available in the repository."
      >
        <DirectoryPicker
          id="sideload-path"
          value={sideloadPath}
          onBrowse={onSideloadPathChange}
        />
      </FormField>
    </div>
  );
};

export default SettingsForm;
