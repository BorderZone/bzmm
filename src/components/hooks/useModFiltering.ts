import { useMemo } from 'react';
import type { Mod } from '../../types/types';

export function useModFiltering(
  mods: Mod[],
  searchQuery: string,
  selectedCategory: string
) {
  const categories = useMemo(() => {
    // Get unique category names with their corresponding sort_order
    const categoryMap = new Map<string, number>();
    categoryMap.set("All", -1); // Ensure "All" is always first

    mods.forEach(mod => {
      // We get the sort_order from any mod in the category since they should all have the same value
      if (!categoryMap.has(mod.category)) {
        categoryMap.set(mod.category, mod.sort_order || 0);
      }
    });

    // Convert to array and sort by sort_order
    return Array.from(categoryMap.entries())
      .sort((a, b) => a[1] - b[1])
      .map(([category]) => category);
  }, [mods]);

  const filteredMods = useMemo(() => 
    mods
      .filter(mod => {
        const matchesSearch = mod.name.toLowerCase().includes(searchQuery.toLowerCase());
        const matchesCategory = selectedCategory === "All" || mod.category === selectedCategory;
        return matchesSearch && matchesCategory;
      })
      .sort((a, b) => {
        // Sort by category first to ensure deprecated mods are at the top
        if (a.category === "Deprecated" && b.category !== "Deprecated") return -1;
        if (a.category !== "Deprecated" && b.category === "Deprecated") return 1;
        
        // Then sort by downloaded status
        if (a.isDownloaded && !b.isDownloaded) return -1;
        if (!a.isDownloaded && b.isDownloaded) return 1;
        
        // Then sort by category sort_order
        if (a.sort_order !== b.sort_order) {
          return (a.sort_order || 0) - (b.sort_order || 0);
        }

        // Finally sort alphabetically by name
        return a.name.localeCompare(b.name);
      }),
    [mods, searchQuery, selectedCategory]
  );

  return {
    categories,
    filteredMods
  };
}

export default useModFiltering;