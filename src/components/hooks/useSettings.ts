import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface SettingsData {
  download_path: string;
  sideload_path: string;
}

const DEFAULT_SETTINGS: SettingsData = {
  download_path: '',
  sideload_path: '',
};

export function useSettings(onSaved?: () => void) {
  const [settings, setSettings] = useState<SettingsData>(DEFAULT_SETTINGS);
  const [isLoading, setIsLoading] = useState(false);

  const loadSettings = useCallback(async () => {
    try {
      setIsLoading(true);
      const savedSettings = await invoke<SettingsData>('get_settings');
      setSettings(savedSettings);
    } catch (error) {
      console.error('Failed to load settings:', error);
      setSettings(DEFAULT_SETTINGS);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleSettingsPathChange = async (key: 'download_path' | 'sideload_path') => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      
      if (selected) {
        setSettings(prev => ({ ...prev, [key]: selected as string }));
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  const handleDownloadPathChange = () => handleSettingsPathChange('download_path');
  const handleSideloadPathChange = () => handleSettingsPathChange('sideload_path');

  const handleSave = async () => {
    try {
      // Save download_path
      await invoke('update_settings', { 
        update: { key: 'download_path', value: settings.download_path }
      });
      
      // Save sideload_path
      await invoke('update_settings', { 
        update: { key: 'sideload_path', value: settings.sideload_path }
      });
      
      onSaved?.();
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  };

  const handleCancel = () => {
    setSettings(DEFAULT_SETTINGS);
  };

  return {
    settings,
    isLoading,
    loadSettings,
    handleDownloadPathChange,
    handleSideloadPathChange,
    handleSave,
    handleCancel
  };
}

export default useSettings;