import { useEffect } from 'react';
import type { Mod } from '../../types/types';
import { invoke } from '@tauri-apps/api/core';
import { Settings } from '../../types/types';
import { listen } from '@tauri-apps/api/event';
import { useDownloadContext } from '../context/DownloadContext';

export function useModActions(
  mods: Mod[],
  setMods: React.Dispatch<React.SetStateAction<Mod[]>>,
  setDownloadedMods: React.Dispatch<React.SetStateAction<Set<string>>>,
  settings: Settings | null,
  currentProfileIndex: number,
  loadMods?: (profileIndex?: number) => Promise<void>
) {
  const { startDownload } = useDownloadContext();

  // Listen for download errors and reset mod state
  useEffect(() => {
    const unlistenError = listen('download-error', (event) => {
      const payload = event.payload as { mod_name: string };
      const modName = payload.mod_name.replace('.zip', '');
      
      // Reset the mod's download state
      setMods(currentMods => 
        currentMods.map(m =>
          m.name === modName ? { ...m, isDownloaded: false } : m
        )
      );
    });

    // Listen for download completion
    const unlistenComplete = listen('download-complete', (event) => {
      const modName = event.payload as string;
      // Update the mod's downloaded state
      const cleanName = modName.replace('.zip', '');
      
      setMods(currentMods => 
        currentMods.map(m =>
          m.name === cleanName ? { ...m, isDownloaded: true } : m
        )
      );
      
      setDownloadedMods(current => new Set([...current, cleanName]));
    });

    return () => {
      unlistenError.then(unsubscribe => unsubscribe());
      unlistenComplete.then(unsubscribe => unsubscribe());
    };
  }, []);

  const handleDownload = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    if (!mod) return;

    // Create filename from mod name
    const filename = `${mod.name}.zip`;

    try {
      console.log(`Starting download for ${mod.name} from URL: ${mod.url}`);
      if (!mod.url || !mod.url.startsWith("http")) {
        throw new Error(`Invalid URL for mod ${mod.name}: ${mod.url || 'empty'}`);
      }

      const currentProfile = settings?.profiles[currentProfileIndex];
      if (!currentProfile) {
        console.error('No profile selected, cannot start download.');
        return; // Or throw an error
      }
      const repoUrl = currentProfile.repo_url || ""; // Use empty string as fallback
      
      // Start the download directly
      startDownload(filename, mod.url, repoUrl);
      
      // We'll update the download state when we receive events from backend
    } catch (error) {
      console.error('Failed to start mod download:', error);
      throw error;
    }
  };

  const handleUpdate = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    if (!mod || !settings) return;
    
    if (!mod.url) {
      throw new Error('Mod URL is missing');
    }
    
    if (!mod.url.startsWith("http")) {
      throw new Error(`Invalid URL for mod ${mod.name}: ${mod.url}`);
    }

    const currentProfile = settings.profiles[currentProfileIndex];
    if (!currentProfile) {
      throw new Error('No profile selected');
    }

    try {
      // Create filename for the update
      const filename = `${mod.name}.zip`;
      
      // Start the download for the update directly
      console.log(`Starting update for ${mod.name} from URL: ${mod.url}`);
      const repoUrl = currentProfile.repo_url || ""; // Use empty string as fallback
      startDownload(filename, mod.url, repoUrl);

      // Listen for download completion to refresh the mod list
      const unlistenComplete = await listen('download-complete', async () => {
        console.log(`Download completed, refreshing mod list`);
        if (loadMods) {
          await loadMods(currentProfileIndex);
        }
        unlistenComplete();
      });
      
      // The backend will handle extraction and replacing the old version
    } catch (error) {
      console.error('Failed to update mod:', error);
      throw error;
    }
  };

  const handleToggle = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    if (!mod || !settings) return;

    // Don't allow enabling if the mod isn't downloaded
    if (!mod.isDownloaded) {
      throw new Error('Cannot enable mod that is not downloaded. Please download the mod first.');
    }

    const currentProfile = settings.profiles[currentProfileIndex];
    if (!currentProfile) {
      throw new Error('No profile selected');
    }

    try {
      if (mod.isEnabled) {
        await invoke('disable_mod', { 
          modName: mod.name,
          profileName: currentProfile.name
        });
      } else {
        await invoke('enable_mod', { 
          modName: mod.name,
          profileName: currentProfile.name
        });
      }
      
      setMods(mods.map(m =>
        m.id === modId ? { ...m, isEnabled: !m.isEnabled } : m
      ));
    } catch (error) {
      console.error('Failed to toggle mod:', error);
      throw error;
    }
  };

  const handleDelete = async (modId: number) => {
    const mod = mods.find(m => m.id === modId);
    if (!mod || !settings) return;

    const currentProfile = settings.profiles[currentProfileIndex];
    if (!currentProfile) {
      throw new Error('No profile selected');
    }

    try {
      await invoke('delete_mod', { 
        modName: mod.name,
        profileName: currentProfile.name
      });
      
      // Update downloaded mods set immediately for quick UI feedback
      setDownloadedMods(prevDownloaded => {
        const newSet = new Set(prevDownloaded);
        newSet.delete(mod.name);
        return newSet;
      });
      
      // Reload the entire mod list to ensure deleted mod reappears with download button
      if (loadMods) {
        // If we have access to the loadMods function from useModData, use it
        await loadMods(currentProfileIndex);
      } else {
        console.log("No loadMods function provided, falling back to manual reload");
        // If loadMods function isn't available, reload mods manually
        const modsResult = await invoke<any>('get_mods', { 
          profileIndex: currentProfileIndex 
        });
        
        // Get current downloaded mods
        const downloaded = await invoke<string[]>('get_downloaded_mods');
        const downloadedModsSet = new Set(downloaded.flatMap(name => {
          const baseName = name.endsWith('.zip') ? name.slice(0, -4) : name;
          return [baseName, baseName + '.zip'];
        }));
        
        // Get enabled mods
        const enabledMods = new Set(await invoke<string[]>('get_enabled_mods', { 
          profileName: currentProfile.name 
        }));
        
        // Transform mods with updated state
        const transformedMods = modsResult.categories.flatMap((category: any, categoryIndex: number) => {
          if (!category?.mods || !Array.isArray(category.mods)) {
            return [];
          }
          
          return category.mods.map((mod: any, modIndex: number) => ({
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
          }));
        });
        
        // Update mods list with fresh data
        setMods(transformedMods);
      }
    } catch (error) {
      console.error('Failed to delete mod:', error);
      throw error;
    }
  };

  return {
    handleDownload,
    handleUpdate,
    handleToggle,
    handleDelete
  };
}

export default useModActions;
