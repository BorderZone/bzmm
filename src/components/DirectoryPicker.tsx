import React from 'react';
import { Button } from './ui/button';
import { Input } from './ui/input';

interface DirectoryPickerProps {
  value: string;
  onBrowse: () => void;
  id?: string;
}

const DirectoryPicker: React.FC<DirectoryPickerProps> = ({ 
  value, 
  onBrowse, 
  id 
}) => {
  return (
    <div className="flex items-center gap-2">
      <Input
        id={id}
        value={value}
        readOnly
        className="flex-1 text-foreground focus:ring-0 cursor-text overflow-x-auto"
      />
      <Button 
        type="button" 
        variant="secondary"
        onClick={onBrowse}
      >
        Browse
      </Button>
    </div>
  );
};

export default DirectoryPicker;