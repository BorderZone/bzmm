import { Mod, Profile } from '../types/types';

export const initialMods: Mod[] = [
  {
    id: 1,
    name: "SPECIAL MISSION PACK 1 - KOREA",
    category: "Missions",
    version: "1.0.0",
    url: "http://example.com/mod1.zip",
    filename: "mod1.zip",
    newVersion: "1.0.1",
    shortDescription: "Complete mission pack including new scenarios and units",
    description: `- FW-190A8 (KPAAF, CAPTURED)
- MOSQUITO FB.MKVI (USSR)
- P-51D (KOREA VAR-9, USAF 12TH FBS, 39TH FG, 67TH FBS)
- BOEING B-29 SUPERFORTRESS

- WIRTS ARTY MOD
- FRENCH PACK V45 MOD`,
    isDownloaded: true,
    sort_order: 1
  },
  {
    id: 2,
    name: "Enhanced Graphics Pack",
    category: "Visual",
    version: "2.1.0",
    url: "http://example.com/mod2.zip",
    filename: "mod2.zip",
    shortDescription: "Improved textures and effects",
    description: `- Improved texture resolution for all ground units
- Enhanced particle effects for explosions and smoke
- New weather effects including rain and snow
- Optimized for performance with minimal FPS impact
- Compatible with all other visual mods`,
    isDownloaded: false,
    sort_order: 2
  },
  {
    id: 3,
    name: "Historical Units Expansion",
    category: "Units",
    version: "3.0.1",
    url: "http://example.com/mod3.zip",
    filename: "mod3.zip",
    newVersion: "3.1.0",
    shortDescription: "Adds historically accurate unit variants",
    description: `- Adds 25 new historically accurate unit variants
- Updated camouflage patterns based on archive photos
- Includes units from lesser-known military operations
- Full documentation included for historical context
- Compatible with base game version 1.5 and above`,
    isDownloaded: true,
    sort_order: 3
  }
];

export const initialProfiles: Profile[] = [
  {
    name: "Default",
    dcs_path: "",
    repo_url: "http://repo.borderzone.ca/mods.xml",
  },
  {
    name: "Minimal",
    dcs_path: "",
    repo_url: "http://repo.borderzone.ca/mods.xml",
  }
];