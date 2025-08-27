import React from 'react';
import { Label } from './ui/label';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
  TooltipProvider
} from './ui/tooltip';
import { HelpCircle } from 'lucide-react';

interface FormFieldProps {
  label: string;
  htmlFor: string;
  children: React.ReactNode;
  tooltip?: string;
}

const FormField: React.FC<FormFieldProps> = ({ 
  label, 
  htmlFor, 
  children,
  tooltip
}) => {
  return (
    <div className="grid grid-cols-12 items-center gap-4">
      <div className="col-span-3 flex items-center justify-end gap-1 text-foreground">
        <Label htmlFor={htmlFor} className="text-right">
          {label}
        </Label>
        {tooltip && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <HelpCircle className="h-4 w-4 text-muted-foreground cursor-help" />
              </TooltipTrigger>
              <TooltipContent className="max-w-xs">
                <p>{tooltip}</p>
              </TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </div>
      <div className="col-span-9">
        {children}
      </div>
    </div>
  );
};

export default FormField;
