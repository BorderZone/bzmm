import React from 'react';

interface LoadingProps {
  message?: string;
}

export const LoadingState: React.FC<LoadingProps> = ({ message = 'Loading...' }) => (
  <div className="w-full h-full flex items-center justify-center">
    {message}
  </div>
);

export default LoadingState;