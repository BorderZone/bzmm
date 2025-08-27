import React from 'react';
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { XCircle } from "lucide-react";

interface CriticalErrorBannerProps {
  error: string;
  title: string;
  onRetry: () => void;
  children?: React.ReactNode;
  className?: string;
}

const CriticalErrorBanner: React.FC<CriticalErrorBannerProps> = ({ 
  error, 
  title,
  onRetry,
  children,
  className = "" 
}) => {
  return (
    <Alert 
      variant="destructive" 
      className={`mb-4 bg-red-900/20 border-red-900/50 text-red-50 ${className}`}
    >
      <XCircle className="h-4 w-4" />
      <AlertTitle className="text-red-50">
        {title}
      </AlertTitle>
      <AlertDescription className="mt-2 text-red-100 whitespace-pre-line">
        {error}
        {children}
        <button
          onClick={onRetry}
          className="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90"
        >
          Try Again
        </button>
      </AlertDescription>
    </Alert>
  );
};

export default CriticalErrorBanner;