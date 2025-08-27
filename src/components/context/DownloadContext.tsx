import React, { createContext, useContext, useEffect, useReducer, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface DownloadStats {
  progress: number;
  downloadedBytes: number;
  totalBytes: number;
  speed: number;  // in bytes per second
  lastUpdateTime: number;
}

interface DownloadState {
  downloading: Set<string>;
  extracting: Set<string>;
  queued: Set<string>;
  progress: Map<string, DownloadStats>;
}

interface ModDownloadInfo {
  url: string;
  repoUrl: string;
}

type Action =
  | { type: 'ADD_QUEUED'; modName: string }
  | { type: 'REMOVE_QUEUED'; modName: string }
  | { type: 'ADD_DOWNLOAD'; modName: string; info?: ModDownloadInfo; markAsDownloading?: boolean }
  | { type: 'REMOVE_DOWNLOAD'; modName: string }
  | { type: 'SET_PROGRESS'; modName: string; progress: number; downloadedBytes: number; totalBytes: number }
  | { type: 'SET_EXTRACTING'; modName: string }
  | { type: 'REMOVE_EXTRACTING'; modName: string }
  | { type: 'RESET_MOD'; modName: string };

const initialState: DownloadState = {
  downloading: new Set(),
  extracting: new Set(),
  queued: new Set(),
  progress: new Map(),
};

function downloadReducer(state: DownloadState, action: Action): DownloadState {
  switch (action.type) {
    case 'ADD_QUEUED': {
      return {
        ...state,
        queued: new Set(state.queued).add(action.modName)
      };
    }

    case 'REMOVE_QUEUED': {
      const queued = new Set(state.queued);
      queued.delete(action.modName);
      return { ...state, queued };
    }

    case 'ADD_DOWNLOAD': {
      // If already downloading, don't add again (e.g. if startDownload is somehow called multiple times rapidly for same mod)
      if (state.downloading.has(action.modName)) {
        return state;
      }
      // Assuming action.markAsDownloading will always be true from the new startDownload function
      // And action.info will be provided.
      const initialStats: DownloadStats = {
        progress: 0,
        downloadedBytes: 0,
        totalBytes: 0,
        speed: 0,
        lastUpdateTime: Date.now()
      };
      return {
        ...state,
        downloading: new Set(state.downloading).add(action.modName),
        progress: new Map(state.progress).set(action.modName, initialStats)
      };
    }

    case 'REMOVE_DOWNLOAD': {
      const downloading = new Set(state.downloading);
      const progress = new Map(state.progress);
      const queued = new Set(state.queued);
      downloading.delete(action.modName);
      progress.delete(action.modName);
      queued.delete(action.modName);
      return { ...state, downloading, progress, queued };
    }

    case 'SET_PROGRESS': {
      const progressMap = new Map(state.progress);
      const now = Date.now();
      const prevStats = progressMap.get(action.modName) || {
        progress: 0,
        downloadedBytes: 0,
        totalBytes: 0,
        speed: 0,
        lastUpdateTime: now
      };
      const timeDiff = (now - prevStats.lastUpdateTime) / 1000;
      const bytesDiff = action.downloadedBytes - prevStats.downloadedBytes;
      const speed = timeDiff > 0.1 ? bytesDiff / timeDiff : prevStats.speed;
      const newStats: DownloadStats = {
        progress: action.progress,
        downloadedBytes: action.downloadedBytes,
        totalBytes: action.totalBytes,
        speed: speed,
        lastUpdateTime: now
      };
      progressMap.set(action.modName, newStats);
      return {
        ...state,
        progress: progressMap
      };
    }

    case 'SET_EXTRACTING': {
      const downloading = new Set(state.downloading);
      const queued = new Set(state.queued);
      downloading.delete(action.modName);
      queued.delete(action.modName);
      return {
        ...state,
        extracting: new Set(state.extracting).add(action.modName),
        downloading,
        queued
      };
    }

    case 'REMOVE_EXTRACTING': {
      const extracting = new Set(state.extracting);
      extracting.delete(action.modName);
      return { ...state, extracting };
    }

    case 'RESET_MOD': {
      const downloading = new Set(state.downloading);
      const extracting = new Set(state.extracting);
      const queued = new Set(state.queued);
      const progress = new Map(state.progress);

      // Clean up both .zip and non-.zip variants to handle naming inconsistencies
      const baseName = action.modName.endsWith('.zip') ? action.modName.slice(0, -4) : action.modName;
      const zipName = baseName + '.zip';

      downloading.delete(action.modName);
      downloading.delete(baseName);
      downloading.delete(zipName);

      extracting.delete(action.modName);
      extracting.delete(baseName);
      extracting.delete(zipName);

      queued.delete(action.modName);
      queued.delete(baseName);
      queued.delete(zipName);

      progress.delete(action.modName);
      progress.delete(baseName);
      progress.delete(zipName);

      return { ...state, downloading, extracting, queued, progress };
    }

    // START_QUEUE_PROCESSING and STOP_QUEUE_PROCESSING cases removed

    default:
      return state;
  }
}

interface DownloadContextType {
  state: DownloadState;
  dispatch: React.Dispatch<Action>;
  startDownload: (modName: string, url: string, repoUrl: string) => void;
  cancelDownload: (modName: string) => void;
}

const DownloadContext = createContext<DownloadContextType | null>(null);

export function DownloadProvider({ children }: { children: React.ReactNode }) {
  const [state, dispatch] = useReducer(downloadReducer, initialState);
  const modUrlMap = React.useRef(new Map<string, ModDownloadInfo>());

  const startDownload = useCallback(async (modName: string, url: string, repoUrl: string) => {
    console.log(`Attempting to queue download for ${modName} from ${url}`);

    // Check if already downloading or queued
    if (state.downloading.has(modName) || state.queued.has(modName)) {
      console.warn(`[startDownload] Mod ${modName} is already downloading or queued. Skipping.`);
      return;
    }

    const info: ModDownloadInfo = { url, repoUrl };
    modUrlMap.current.set(modName, info); // Store URL info before dispatching

    // Add to queued state first
    dispatch({ type: 'ADD_QUEUED', modName });

    try {
      console.log(`Queuing download for ${modName}`);
      await invoke('queue_download', {
        url,
        filename: modName,
        repoUrl
      });
      console.log(`Successfully queued download for ${modName}`);
    } catch (error) {
      console.error(`Error queuing download for ${modName}:`, error);
      modUrlMap.current.delete(modName); // Clean up URL info
      dispatch({ type: 'RESET_MOD', modName }); // Reset state for this mod
    }
  }, [state.downloading, state.queued]); // Updated dependencies

  const cancelDownload = useCallback(async (modName: string) => {
    console.log(`Attempting to cancel download for ${modName}`);

    try {
      await invoke('cancel_download', { filename: modName });
      console.log(`Successfully cancelled download for ${modName}`);
      
      // Reset state for this mod
      dispatch({ type: 'RESET_MOD', modName });
      
      // Clean up URL info
      modUrlMap.current.delete(modName);
      
    } catch (error) {
      console.error(`Error cancelling download for ${modName}:`, error);
    }
  }, []);

  useEffect(() => {
    const unsubscribe = Promise.all([
      listen('download-queued', (event) => {
        const modName = event.payload as string;
        console.log(`Backend confirmed download queued for: ${modName}`);
      }),

      listen('download-started', (event) => {
        const modName = event.payload as string;
        console.log(`Backend confirmed download started for: ${modName}`);

        // Move from queued to downloading
        dispatch({ type: 'REMOVE_QUEUED', modName });

        const info = modUrlMap.current.get(modName) || { url: '', repoUrl: '' };
        dispatch({ type: 'ADD_DOWNLOAD', modName, info, markAsDownloading: true });
      }),

      listen('download-progress', (event) => {
        const payload = event.payload as {
          modName: string;
          progressPercent: number;
          downloadedBytes: number;
          totalBytes: number;
        };

        dispatch({
          type: 'SET_PROGRESS',
          modName: payload.modName,
          progress: payload.progressPercent,
          downloadedBytes: payload.downloadedBytes,
          totalBytes: payload.totalBytes
        });
      }),

      listen('extraction-status', (event) => {
        const payload = event.payload as { mod_name: string; status: string };
        if (payload.status === 'extracting') {
          dispatch({ type: 'SET_EXTRACTING', modName: payload.mod_name });
        } else if (payload.status === 'completed') {
          dispatch({ type: 'REMOVE_EXTRACTING', modName: payload.mod_name });
          // No longer call processQueue here
        }
      }),

      listen('extraction-error', (event) => {
        const payload = event.payload as { mod_name: string; error: string };
        // Format the mod name to remove .zip extension if present
        const modName = payload.mod_name.endsWith('.zip')
          ? payload.mod_name.slice(0, -4)
          : payload.mod_name;

        // Display extraction error with consistent formatting
        window.dispatchEvent(new CustomEvent('show-error', {
          detail: {
            title: 'Extraction Error',
            message: `Failed to extract ${modName}: ${payload.error}`
          }
        }));

        console.error(`Extraction error for ${modName}:`, payload.error);

        // Reset states for this mod using the cleaned name
        dispatch({ type: 'RESET_MOD', modName: modName });
        // No longer call processQueue here
      }),

      listen('download-error', (event) => {
        const payload = event.payload as { mod_name: string, error: string };
        // Format the mod name to remove .zip extension if present
        const modName = payload.mod_name.endsWith('.zip')
          ? payload.mod_name.slice(0, -4)
          : payload.mod_name;

        // Display download error with consistent formatting
        window.dispatchEvent(new CustomEvent('show-error', {
          detail: {
            title: 'Download Error',
            message: `Failed to download ${modName}: ${payload.error}`
          }
        }));

        console.error(`Download error for ${modName}:`, payload.error);

        // Reset the download state for this mod using the potentially cleaned name
        dispatch({ type: 'RESET_MOD', modName: modName });
        // No longer call processQueue here
      }),

      listen('download-complete', () => {
        // No longer call processQueue here; download success implicitly frees up a slot if using maxConcurrent,
        // or simply means this one is done. The useEffect hook will handle queue processing.
        // Or, if specific mod info is needed here for cleanup, it should be added.
        // For now, just removing the direct processQueue call.
      }),

      listen('download-cancelled', (event) => {
        const modName = event.payload as string;
        console.log(`Download cancelled for: ${modName}`);
        
        // Reset all states for this mod
        dispatch({ type: 'RESET_MOD', modName });
      })
    ]);

    return () => {
      unsubscribe.then(unsubs => unsubs.forEach(u => u()));
    };
  }, []);

  return (
    <DownloadContext.Provider value={{ state, dispatch, startDownload, cancelDownload }}>
      {children}
    </DownloadContext.Provider>
  );
}

export function useDownloadContext() {
  const context = useContext(DownloadContext);
  if (!context) {
    throw new Error('useDownloadContext must be used within a DownloadProvider');
  }
  return context;
}
