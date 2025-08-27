import React, { useState } from 'react';
import { Button } from './ui/button';
import { Switch } from './ui/switch';
import {
  ChevronUp,
  ChevronDown,
  ArrowUpCircle,
  Download,
  Trash2,
  X
} from 'lucide-react';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from './ui/tooltip';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "./ui/alert-dialog";
import type { Mod, Profile } from '../types/types';

interface ModCardActionsProps {
  mod: Mod;
  profile: Profile;
  isExpanded: boolean;
  isDownloading: boolean;
  isExtracting: boolean;
  isQueued?: boolean;
  onUpdate: () => void;
  onDownload: () => void;
  onToggle: () => void;
  onDelete?: () => void;
  onCancel?: () => void;
  onExpandClick: () => void;
}

const ModCardActions: React.FC<ModCardActionsProps> = ({
  mod,
  profile,
  isExpanded,
  isDownloading,
  isExtracting,
  isQueued = false,
  onUpdate,
  onDownload,
  onToggle,
  onDelete,
  onCancel,
  onExpandClick,
}) => {
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const dcsPathSet = profile.dcs_path && profile.dcs_path.length > 0;

  // const getButtonStatus = () => {
  //   if (isExtracting) return 'Extracting...';
  //   if (isDownloading) return 'Downloading...';
  //   if (isQueued) return 'Queued';
  //   return null;
  // };

  // const buttonStatus = getButtonStatus();
  const showStatusButton = isDownloading || isExtracting || isQueued;

  const handleDelete = () => {
    setShowDeleteDialog(false);
    if (onDelete) {
      onDelete();
    }
  };

  return (
    <>
      <div className="flex items-center gap-2">
        {/* Show download button if not downloaded or cancel button if processing */}
        {(!mod.isDownloaded && mod.url) ? (
          <Button
            size="sm"
            onClick={showStatusButton ? onCancel : onDownload}
            variant={showStatusButton ? "destructive" : "default"}
          >
            {showStatusButton ? (
              <X className="h-4 w-4 mr-1" />
            ) : (
              <Download className="h-4 w-4 mr-1" />
            )}
            {showStatusButton ? 'Cancel' : 'Download'}
          </Button>
        ) : null}

        {/* Show update button if an update is available or cancel button if processing */}
        {mod.isDownloaded && mod.newVersion && (
          <Button
            size="sm"
            variant={showStatusButton ? "destructive" : "outline"}
            onClick={showStatusButton ? onCancel : onUpdate}
          >
            {showStatusButton ? (
              <X className="h-4 w-4 mr-1" />
            ) : (
              <ArrowUpCircle className="h-4 w-4 mr-1" />
            )}
            {showStatusButton ? 'Cancel' : `Upgrade to v${mod.newVersion}`}
          </Button>
        )}

        {/* Show cancel button during extraction when no other buttons are shown */}
        {isExtracting && !(!mod.isDownloaded && mod.url) && !(mod.isDownloaded && mod.newVersion) && (
          <Button
            size="sm"
            variant="destructive"
            onClick={onCancel}
          >
            <X className="h-4 w-4 mr-1" />
            Cancel
          </Button>
        )}

        {/* Show enable/disable toggle if mod is downloaded and not currently being processed */}
        {mod.isDownloaded && !isExtracting && !isDownloading && !isQueued && (
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">
              {!dcsPathSet ? (
                "Set Profile's DCS Path to Enable"
              ) : (
                mod.isEnabled ? "Enabled" : "Disabled"
              )}
            </span>
            <Switch
              checked={mod.isEnabled ?? false}
              onCheckedChange={onToggle}
              disabled={!dcsPathSet || isExtracting || isDownloading || isQueued}
            />
          </div>
        )}

        {/* Delete button - Only for downloaded, disabled mods that aren't sideloaded or processing */}
        {mod.isDownloaded && !mod.isEnabled &&
          mod.category !== "Sideloaded" &&
          !isExtracting && !isDownloading && !isQueued && onDelete && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setShowDeleteDialog(true)}
                    className="h-8 w-8 p-0 text-destructive hover:bg-destructive"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Delete mod</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}

        {/* Expand/Collapse button */}
        <Button
          variant="ghost"
          size="sm"
          onClick={onExpandClick}
          className="h-8 w-8 p-0"
        >
          {isExpanded ? (
            <ChevronUp className="h-4 w-4" />
          ) : (
            <ChevronDown className="h-4 w-4" />
          )}
        </Button>
      </div>

      {/* Delete confirmation dialog */}
      <AlertDialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle className="text-foreground">Delete Mod</AlertDialogTitle>
            <AlertDialogDescription className="text-muted-foreground">
              Are you sure you want to delete {mod.name}?
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel className="bg-background text-foreground hover:bg-muted">Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
};

export default ModCardActions;
