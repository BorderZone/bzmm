import { useCallback } from 'react';
import { Mod } from '../../types/types';
import { useDownloadContext } from '../context/DownloadContext';

export function useCategoryActions(repoUrl: string) {
  const { startDownload } = useDownloadContext();

  const downloadCategoryMods = useCallback((mods: Mod[]) => {
    const modsToDownload = mods.filter(mod => !mod.isDownloaded && mod.url);

    // Start downloads for each mod
    modsToDownload.forEach(mod => {
      if (mod.url) {
        const filename = `${mod.name}.zip`;
        startDownload(filename, mod.url, repoUrl);
      }
    });

    return modsToDownload.length;
  }, [startDownload, repoUrl]);

  const updateCategoryMods = useCallback((mods: Mod[]) => {
    const modsToUpdate = mods.filter(mod =>
      mod.isDownloaded && mod.newVersion && mod.url
    );

    // Start updates for each mod (by starting a download)
    modsToUpdate.forEach(mod => {
      if (mod.url) {
        const filename = `${mod.name}.zip`;
        startDownload(filename, mod.url, repoUrl);
      }
    });

    return modsToUpdate.length;
  }, [startDownload, repoUrl]);

  return {
    downloadCategoryMods,
    updateCategoryMods
  };
}

export default useCategoryActions;
