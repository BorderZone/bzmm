import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Mod, Settings } from '../../types/types';

const LOADING_DELAY = 500; // ms before showing loading state

interface ModsResult {
  categories: any[];
  error: string | null;
}

function formatErrorMessage(error: unknown): string {
  const errorStr = error instanceof Error ? error.toString() : String(error);
  console.log('Formatting error message from:', errorStr);
  if (errorStr.includes('missing field')) {
    return `Failed to load settings: Your settings file appears to be outdated. It's missing required fields.
Please delete your settings file and restart the application, or contact support for help with migration.
Technical details: ${errorStr}`;
  }
  return `Failed to load settings: ${errorStr}`;
}

export function useModData(setGlobalError: (error: string | null) => void) {
  const [mods, setMods] = useState<Mod[]>([]);
  const [settings, setSettings] = useState<Settings | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [showLoading, setShowLoading] = useState(false);
  const [downloadedMods, setDownloadedMods] = useState<Set<string>>(new Set());
  const [currentProfileIndex, setCurrentProfileIndex] = useState<number>(0);
  const [modsError, setModsError] = useState<string | null>(null);

  const transformMods = async (categories: any[], downloadedModsSet: Set<string>, profileName: string) => {
    const enabledMods = new Set(await invoke<string[]>('get_enabled_mods', { profileName }));
    
    return categories.flatMap((category: any, categoryIndex: number) => {
      if (!category?.mods || !Array.isArray(category.mods)) {
        console.warn('Invalid category data:', category);
        return [];
      }
      
      return category.mods.map((mod: any, modIndex: number) => {
        console.log('Processing mod:', mod); // Keep this for debugging
        const transformedMod = {
          id: categoryIndex * 1000 + modIndex,
          name: mod.name,
          category: category.name || 'Uncategorized',
          version: mod.version || '0.0.0',
          newVersion: mod.newVersion || undefined,
          url: mod.url || null,
          filename: mod.name ? `${mod.name}.zip` : null,
          shortDescription: mod.description?.split('\n')[0] || '',
          description: mod.description || '',
          isDownloaded: downloadedModsSet.has(mod.name) || downloadedModsSet.has(mod.name + '.zip'),
          isEnabled: enabledMods.has(mod.name),
          sort_order: category.sort_order || 0,
        };
        return transformedMod;
      });
    });
  };

  const loadSettings = async () => {
    try {
      const settingsData = await invoke<Settings>('get_settings');
      setSettings(settingsData);
      setGlobalError(null);
    } catch (error) {
      const formattedError = formatErrorMessage(error);
      setGlobalError(formattedError);
      setSettings(null);
    }
  };

  const loadMods = async (profileIndex?: number) => {
    if (profileIndex !== undefined) {
      setCurrentProfileIndex(profileIndex);
    }
    
    try {
      setIsLoading(true);
      
      const currentProfile = settings?.profiles[profileIndex ?? currentProfileIndex];
      if (!currentProfile) {
        throw new Error('No profile selected');
      }

      console.log(`Reloading mods for profile: ${currentProfile.name}, index: ${profileIndex ?? currentProfileIndex}`);
      
      const [modsResult, downloaded] = await Promise.all([
        invoke<ModsResult>('get_mods', { 
          profileIndex: profileIndex ?? currentProfileIndex 
        }),
        invoke<string[]>('get_downloaded_mods')
      ]);
      
      const downloadedModsSet = new Set(downloaded.flatMap(name => {
        const baseName = name.endsWith('.zip') ? name.slice(0, -4) : name;
        return [baseName, baseName + '.zip'];
      }));
      
      const transformedMods = await transformMods(modsResult.categories, downloadedModsSet, currentProfile.name);
      
      setMods(transformedMods);
      setDownloadedMods(downloadedModsSet);
      
      if (modsResult.error) {
        setModsError(modsResult.error);
      } else {
        setModsError(null);
      }
    } catch (error) {
      console.error('Critical error loading mods:', error);
      setMods([]);
      setDownloadedMods(new Set());
      setModsError(formatErrorMessage(error));
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    let timer: number;
    if (isLoading) {
      timer = window.setTimeout(() => setShowLoading(true), LOADING_DELAY);
    } else {
      setShowLoading(false);
    }
    return () => {
      if (timer) window.clearTimeout(timer);
    };
  }, [isLoading]);

  useEffect(() => {
    loadSettings();
  }, [setGlobalError]);

  useEffect(() => {
    if (settings) {
      loadMods();
    }
  }, [settings, currentProfileIndex]);

  return {
    mods,
    setMods,
    settings,
    loading: showLoading,
    downloadedMods,
    setDownloadedMods,
    loadSettings,
    loadMods,
    currentProfileIndex,
    setCurrentProfileIndex,
    modsError
  };
}

export default useModData;