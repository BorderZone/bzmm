import React, { useEffect, useState } from 'react';
import ErrorBanner from './ErrorBanner';

interface ErrorData {
  id: string;
  title: string;
  message: string;
  timestamp: number;
}

// Helper function to create a unique message signature to identify duplicates
const createMessageSignature = (title: string, message: string): string => {
  return `${title}:${message.split('\n')[0]}`;
};

const GlobalErrorHandler: React.FC = () => {
  const [errors, setErrors] = useState<ErrorData[]>([]);
  // Keep track of message signatures to avoid duplicates
  const [messageSignatures, setMessageSignatures] = useState<Set<string>>(new Set());

  useEffect(() => {
    // Listen for error events
    const handleError = (event: Event) => {
      const customEvent = event as CustomEvent<{ title: string; message: string }>;
      const title = customEvent.detail.title;
      const message = customEvent.detail.message;
      
      // Create a signature for this message
      const signature = createMessageSignature(title, message);
      
      // Check if we already have this error message
      if (messageSignatures.has(signature)) {
        console.log('Skipping duplicate error:', signature);
        return;
      }
      
      const newError = {
        id: `error-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`,
        title,
        message,
        timestamp: Date.now(),
      };
      
      // Add signature to the set
      setMessageSignatures(prev => new Set(prev).add(signature));
      
      // Add error to the list
      setErrors((prev) => [...prev, newError]);
      
      // Log the error to console
      console.log(`Error: ${newError.title} - ${newError.message}`);
    };

    window.addEventListener('show-error', handleError);

    return () => {
      window.removeEventListener('show-error', handleError);
    };
  }, [messageSignatures]);

  // Auto-dismiss errors after 30 seconds
  useEffect(() => {
    if (errors.length === 0) return;
    
    const timer = setInterval(() => {
      const now = Date.now();
      const errorsToRemove = new Set<string>();
      
      // Find errors to remove based on timestamp
      errors.forEach(error => {
        if (now - error.timestamp >= 30000) {
          const signature = createMessageSignature(error.title, error.message);
          errorsToRemove.add(signature);
        }
      });
      
      // Remove the errors
      if (errorsToRemove.size > 0) {
        setErrors(prev => 
          prev.filter(error => {
            const isOld = now - error.timestamp >= 30000;
            if (isOld) {
              // Also remove signature
              const signature = createMessageSignature(error.title, error.message);
              setMessageSignatures(prev => {
                const newSet = new Set(prev);
                newSet.delete(signature);
                return newSet;
              });
            }
            return !isOld;
          })
        );
      }
    }, 5000);
    
    return () => clearInterval(timer);
  }, [errors]);

  const dismissError = (id: string) => {
    setErrors((prev) => {
      const errorToRemove = prev.find(error => error.id === id);
      if (errorToRemove) {
        // Remove signature from the set
        const signature = createMessageSignature(errorToRemove.title, errorToRemove.message);
        setMessageSignatures(prev => {
          const newSet = new Set(prev);
          newSet.delete(signature);
          return newSet;
        });
      }
      return prev.filter((error) => error.id !== id);
    });
  };

  if (errors.length === 0) {
    return null;
  }

  return (
    <div className="fixed top-4 right-4 z-50 max-w-md space-y-2">
      {errors.map((error) => (
        <ErrorBanner
          key={error.id}
          error={error.message}
          title={error.title}
          onDismiss={() => dismissError(error.id)}
          className="shadow-lg"
        />
      ))}
    </div>
  );
};

export default GlobalErrorHandler;
