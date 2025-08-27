export interface Mod {
  id: number;
  name: string;
  category: string;
  version: string;
  url: string | null;  // Changed to allow null
  filename: string | null;  // Changed to allow null
  newVersion?: string;
  shortDescription: string;
  description: string;
  isDownloaded: boolean;
  isEnabled?: boolean;
  sort_order: number;
}

export interface Profile {
  name: string;
  dcs_path: string;  // Changed from mod_path
  repo_url: string;
}

export interface Settings {
  dark_mode: 'System' | 'Light' | 'Dark';
  download_path: string;
  sideload_path: string;
  profiles: Profile[];
}