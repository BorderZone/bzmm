import React from 'react';
import type { Mod } from '../types/types';
import { AlertTriangle } from 'lucide-react';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "./ui/tooltip";

interface ModCardHeaderProps {
  mod: Mod;
}

const ModCardHeader: React.FC<ModCardHeaderProps> = ({ mod }) => {
  const isDeprecated = mod.category === "Deprecated";

  return (
    <div className="flex-1">
      <div className="flex items-center gap-2">
        <h3 className="text-lg font-semibold">
          {mod.name}
        </h3>
        <span className="text-sm text-muted-foreground">
          v{mod.version}
        </span>
        {isDeprecated && (
          <div className="flex items-center gap-1">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <AlertTriangle className="h-4 w-4 text-yellow-500" />
                </TooltipTrigger>
                <TooltipContent>
                  <p>This mod is deprecated (no longer in the repository)</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <span className="text-xs font-semibold bg-yellow-500/20 text-yellow-700 dark:text-yellow-400 px-1.5 py-0.5 rounded">DEPRECATED</span>
          </div>
        )}
      </div>
      <p className="text-sm text-muted-foreground">
        {mod.shortDescription}
      </p>
    </div>
  );
};

export default ModCardHeader;