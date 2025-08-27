import React from 'react';
import ErrorBanner from './ErrorBanner';

interface ModManagerWrapperProps {
  error: string | null;
  onDismissError: () => void;
  children: React.ReactNode;
}

const ModManagerWrapper: React.FC<ModManagerWrapperProps> = ({
  error,
  onDismissError,
  children
}) => {
  // Only show errors in the wrapper if they're not download/extraction errors
  // Those are now handled by GlobalErrorHandler
  const shouldShowError = error && 
    !error.includes('Downloaded file is not a valid ZIP') && 
    !error.includes('Failed to download') &&
    !error.includes('Failed to extract');

  return (
    <div className="h-full flex flex-col">
      {shouldShowError && (
        <div className="px-4 mt-4">
          <ErrorBanner
            error={error}
            onDismiss={onDismissError}
          />
        </div>
      )}
      <div className="flex flex-1 gap-4 p-4 h-[calc(100vh-2rem)] overflow-hidden">
        {children}
      </div>
    </div>
  );
};

export default ModManagerWrapper;
