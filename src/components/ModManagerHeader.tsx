import React from 'react';
import { Button } from './ui/button';
import { RefreshCw } from 'lucide-react';
import SettingsDialog from './SettingsDialog';

interface ModManagerHeaderProps {
  profileName: string;
  selectedCategory: string;
  filteredModsCount: number;
  onRefresh: () => void;
}

const ModManagerHeader: React.FC<ModManagerHeaderProps> = ({
  profileName,
  selectedCategory,
  onRefresh,
}) => {
  return (
    <div className="flex justify-between items-center mb-4">
      <h1 className="text-xl font-bold">
        {profileName} - {selectedCategory === "All" ? "All Mods" : selectedCategory}
      </h1>
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={onRefresh}
          title="Refresh mod list"
        >
          <RefreshCw className="h-4 w-4" />
        </Button>
        <SettingsDialog onSaved={onRefresh} />
      </div>
    </div>
  );
};

export default ModManagerHeader;