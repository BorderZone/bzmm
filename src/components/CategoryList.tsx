import React from 'react';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { ChevronRight, Download, RotateCw } from 'lucide-react';
import { Mod } from '../types/types';
import { useCategoryActions } from './hooks/useCategoryActions';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from './ui/tooltip';

interface CategoryListProps {
  categories: string[];
  mods: Mod[];
  searchQuery: string;
  setSearchQuery: React.Dispatch<React.SetStateAction<string>>;
  selectedCategory: string;
  setSelectedCategory: React.Dispatch<React.SetStateAction<string>>;
  repoUrl: string;
}

const CategoryList: React.FC<CategoryListProps> = ({
  categories,
  mods,
  searchQuery,
  setSearchQuery,
  selectedCategory,
  setSelectedCategory,
  repoUrl,
}) => {
  const { downloadCategoryMods, updateCategoryMods } = useCategoryActions(repoUrl);

  const getCategoryMods = (categoryName: string): Mod[] => {
    return categoryName === 'All' 
      ? mods 
      : mods.filter(m => m.category === categoryName);
  };

  const handleDownloadCategory = (e: React.MouseEvent, category: string) => {
    e.stopPropagation();
    if (category === 'Sideloaded' || category === 'Deprecated') return; // Don't download sideloaded or deprecated mods
    
    const categoryMods = getCategoryMods(category);
    const count = downloadCategoryMods(categoryMods);
    console.log(`Queued ${count} mods for download from category ${category}`);
  };

  const handleUpdateCategory = (e: React.MouseEvent, category: string) => {
    e.stopPropagation();
    if (category === 'Sideloaded' || category === 'Deprecated') return; // Don't update sideloaded or deprecated mods
    
    const categoryMods = getCategoryMods(category);
    const count = updateCategoryMods(categoryMods);
    console.log(`Queued ${count} mods for update from category ${category}`);
  };

  const getDownloadableMods = (category: string): Mod[] => {
    if (category === 'Sideloaded' || category === 'Deprecated') return [];
    
    const categoryMods = getCategoryMods(category);
    return categoryMods.filter(mod => !mod.isDownloaded && mod.url);
  };

  const getUpdatableMods = (category: string): Mod[] => {
    if (category === 'Sideloaded' || category === 'Deprecated') return [];
    
    const categoryMods = getCategoryMods(category);
    return categoryMods.filter(mod => mod.isDownloaded && mod.newVersion && mod.url);
  };

  return (
    <div className="flex flex-col gap-2">
      <h2 className="font-semibold">Categories</h2>
      <Input
        placeholder="Search mods..."
        value={searchQuery}
        onChange={e => setSearchQuery(e.target.value)}
      />
      <div className="flex flex-col">
        {categories.map(category => {
          const downloadableMods = getDownloadableMods(category);
          const updatableMods = getUpdatableMods(category);
          
          return (
            <Button
              key={category}
              variant="ghost"
              className={`justify-start ${
                selectedCategory === category ? 'bg-muted' : ''
              }`}
              onClick={() => setSelectedCategory(category)}
            >
              <ChevronRight className="h-4 w-4 mr-2" />
              <span className="flex-1 text-left">{category}</span>
              <span className="flex items-center gap-2">
                {category !== 'Sideloaded' && category !== 'Deprecated' && downloadableMods.length > 0 && (
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 p-0"
                          onClick={(e) => handleDownloadCategory(e, category)}
                        >
                          <Download className="h-4 w-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>
                          {category === 'All' 
                            ? `Download ${downloadableMods.length} mods` 
                            : `Download ${downloadableMods.length} mods in this category`}
                        </p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                )}
                
                {category !== 'Sideloaded' && category !== 'Deprecated' && updatableMods.length > 0 && (
                  <TooltipProvider>
                    <Tooltip>
                      <TooltipTrigger asChild>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="h-8 w-8 p-0"
                          onClick={(e) => handleUpdateCategory(e, category)}
                        >
                          <RotateCw className="h-4 w-4" />
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>
                        <p>
                          {category === 'All' 
                            ? `Upgrade ${updatableMods.length} mods` 
                            : `Upgrade ${updatableMods.length} mods in this category`}
                        </p>
                      </TooltipContent>
                    </Tooltip>
                  </TooltipProvider>
                )}
                
                <span className="text-muted-foreground text-sm">
                  {category === 'All'
                    ? mods.length
                    : mods.filter(m => m.category === category).length}
                </span>
              </span>
            </Button>
          );
        })}
      </div>
    </div>
  );
};

export default CategoryList;
