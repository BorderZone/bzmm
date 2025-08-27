import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';


interface DownloadState {
  downloading: Set<string>;
  extracting: Set<string>;
  enabling: Set<string>; // Track enabling state
  error: string | null;
  progress: Map<string, number>;
  addDownload: (modName: string) => void;
  removeDownload: (modName: string) => void;
  setExtracting: (modName: string) => void;
  removeExtracting: (modName: string) => void;
  setEnabling: (modName: string) => void; // Set mod as being enabled
  removeEnabling: (modName: string) => void; // Remove mod from enabling state
  setProgress: (modName: string, progress: number) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

export const useDownloadState = create<DownloadState>((set: any) => ({
  downloading: new Set<string>(),
  extracting: new Set<string>(),
  enabling: new Set<string>(),
  error: null,
  progress: new Map<string, number>(),
  addDownload: (modName: string) => set((state: DownloadState) => ({
    downloading: new Set(state.downloading).add(modName),
    progress: new Map(state.progress).set(modName, 0)
  })),
  removeDownload: (modName: string) => set((state: DownloadState) => {
    const downloading = new Set(state.downloading);
    const progress = new Map(state.progress);
    downloading.delete(modName);
    progress.delete(modName);
    return { downloading, progress };
  }),
  setExtracting: (modName: string) => set((state: DownloadState) => ({
    extracting: new Set(state.extracting).add(modName),
    downloading: new Set(Array.from(state.downloading).filter(m => m !== modName))
  })),
  removeExtracting: (modName: string) => set((state: DownloadState) => {
    const extracting = new Set(state.extracting);
    extracting.delete(modName);
    return { extracting };
  }),
  setEnabling: (modName: string) => set((state: DownloadState) => ({
    enabling: new Set(state.enabling).add(modName),
    // Remove from extraction state if it was there
    extracting: new Set(Array.from(state.extracting).filter(m => m !== modName))
  })),
  removeEnabling: (modName: string) => set((state: DownloadState) => {
    const enabling = new Set(state.enabling);
    enabling.delete(modName);
    return { enabling };
  }),
  setProgress: (modName: string, progress: number) => set((state: DownloadState) => ({
    progress: new Map(state.progress).set(modName, progress)
  })),
  setError: (error: string | null) => set({ error }),
  reset: () => set({
    downloading: new Set<string>(),
    extracting: new Set<string>(),
    enabling: new Set<string>(),
    error: null,
    progress: new Map<string, number>()
  })
}));

// Event interfaces
interface DownloadProgressEvent {
  modName: string;
  progressPercent: number;
}

interface ExtractionStatusEvent {
  mod_name: string;
  status: string;
}

interface ModEnablementEvent {
  mod_name: string;
  status: string;
}

interface DownloadErrorEvent {
  mod_name: string;
  error: string;
}

// Set up event listeners
listen('download-progress', (event: any) => {
  const payload = event.payload as DownloadProgressEvent;
  useDownloadState.getState().setProgress(payload.modName, payload.progressPercent);
});

listen('extraction-status', (event: any) => {
  const payload = event.payload as ExtractionStatusEvent;
  const state = useDownloadState.getState();
  
  if (payload.status === 'extracting') {
    state.setExtracting(payload.mod_name);
  } else if (payload.status === 'completed') {
    state.removeExtracting(payload.mod_name);
  }
});

// Listen for mod enablement events
listen('mod-enablement', (event: any) => {
  const payload = event.payload as ModEnablementEvent;
  const state = useDownloadState.getState();
  
  if (payload.status === 'enabling') {
    state.setEnabling(payload.mod_name);
  } else if (payload.status === 'completed' || payload.status === 'error') {
    state.removeEnabling(payload.mod_name);
  }
});

listen('download-error', (event: any) => {
  const payload = event.payload as DownloadErrorEvent;
  const state = useDownloadState.getState();
  state.removeDownload(payload.mod_name);
  state.removeExtracting(payload.mod_name);
  state.removeEnabling(payload.mod_name);
  
  // Set the error message to display to the user
  if (payload.error) {
    console.error(`Download error for ${payload.mod_name}: ${payload.error}`);
    state.setError(`Failed to download ${payload.mod_name.replace('.zip', '')}: ${payload.error}`);
    
    // Clear the error after 5 seconds
    setTimeout(() => {
      state.setError(null);
    }, 5000);
  }
});

export default useDownloadState;