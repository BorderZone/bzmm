# BorderZone Mod Manager - Mod Enablement Process

## Overview

The BZMM mod enablement process involves merging mod content into the DCS directory structure using a combination of directory creation, symlinks, and lua file patching. This document details how mods are enabled and disabled.

## Directory Structure

A typical mod directory structure looks like:

```
ModName/                          # Base mod directory
├── README.txt                    # Required file
├── VERSION.txt                   # Required file, contains version string
└── ModName/                      # Main subdirectory (same name as parent)
    ├── Mods/                     # 2nd level directory
    │   ├── aircraft/            # 4rd level directory
    │   │   ├── Aircraft1/       # 4th level directory
    │   │   └── Aircraft2/       # 4th level directory
    │   └── tech/                # 3rd level directory
    └── Liveries/                # Another 2nd level directory
        └── Aircraft1/           # 3rd level directory
```

## Directory Levels

The process treats different directory levels uniquely:

1. **2nd and 3rd Level** (e.g., 2nd: "Mods"/"Liveries", 3rd: "aircraft"/"tech")
   - Created as regular directories in DCS directory if they don't exist
   - Any existing directories are preserved
   - Files at these levels are ignored
   - Case insensitive matching (e.g., "Mods" matches "mods", "aircraft" matches "Aircraft")

2. **4th Level and Below** (e.g., individual aircraft directories, files)
   - Directories: Created as symlinks pointing to mod directory
   - Lua Files: Either symlinked or patched if the file already exists
   - Other Files: Created as symlinks, unless file exists (then error)

## Enabling Process

When a mod is enabled:

1. Verify mod structure:
   - Check for README.txt and VERSION.txt
   - Verify main subdirectory exists with same name as mod

2. Create ENABLING file in mod directory

3. Process directories recursively:
   - Create 2nd and 3rd level directories as needed
   - At 4th level and below:
     - For directories: Create symlinks to mod directory
     - For .lua files: Create symlink if file doesn't exist, patch if it does
     - For other files: Create symlink if file doesn't exist, error if it does

4. Remove ENABLING file and create ENABLED file

### Lua File Patching

When a lua file needs to be patched:
```lua
-- Original content
... existing content ...

-- This was added automatically by BorderZone Mod Manager. DO NOT EDIT! --
-- {"mod_name": "ModName", "version": "1.0.0"}
... mod content ...
-- This was added automatically by BorderZone Mod Manager. DO NOT EDIT! --
```

## Disabling Process

When a mod is disabled:

1. Verify mod structure and ENABLED file exists

2. Process directories recursively:
   - Only process 4th level and below
   - Remove symlinks that point to this mod
   - Remove lua patches for this mod
   - Remove empty directories

3. Remove ENABLED file

## Error Handling

The process handles several error cases:

- Directory Structure Errors: Missing required files or incorrect structure
- File Conflicts: Non-lua file already exists where symlink would be created
- Lua Patch Errors: Issues during lua file patching
- Enablement State Errors: Mod currently being enabled/disabled

## State Files

- `ENABLED-{profile}.txt`: Empty file indicating mod is enabled for profile
- `ENABLING-{profile}.txt`: Temporary file during enablement process
- If ENABLING exists, mod is considered disabled, but files should be cleaned up.

## Design Notes

- The process is idempotent - running enable multiple times has same effect as running once
- All state is stored either in the mod directory (ENABLED files) or DCS directory (symlinks, patches)
- Easy to recover from interruption by checking ENABLED/ENABLING state
- Supports multiple profiles by using profile-specific ENABLED files
- Case insensitive matching is used throughout to support Windows filesystems
- Directory names are preserved in their original case when creating new directories