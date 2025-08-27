import React from 'react';
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { XCircle } from "lucide-react";

interface ErrorBannerProps {
  error: string;
  title?: string;
  onDismiss?: () => void;
  children?: React.ReactNode;
  className?: string;
}

const ErrorBanner: React.FC<ErrorBannerProps> = ({ 
  error, 
  title = "Error",
  onDismiss, 
  children,
  className = "" 
}) => {
  return (
    <Alert 
      variant="destructive" 
      className={`mb-4 bg-red-900 border-red-900/50 text-red-50 ${className}`}
    >
      <XCircle className="h-4 w-4" />
      <AlertTitle className="flex justify-between items-center text-red-50">
        {title}
        {onDismiss && (
          <button 
            onClick={onDismiss}
            className="text-sm text-red-200 hover:text-red-100"
          >
            Dismiss
          </button>
        )}
      </AlertTitle>
      <AlertDescription className="mt-2 text-red-100 whitespace-pre-line">
        {error}
        {error.toLowerCase().includes('url') && (
          <div className="mt-2">
            You can fix this in the Settings dialog accessed through the gear icon next to "Default - All Mods".
          </div>
        )}
        {children}
      </AlertDescription>
    </Alert>
  );
};

export default ErrorBanner;