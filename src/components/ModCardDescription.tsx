import React from 'react';

interface ModCardDescriptionProps {
  description: string;
}

const ModCardDescription: React.FC<ModCardDescriptionProps> = ({ description }) => {
  return (
    <div className="mt-2 pt-2 border-t border-border">
      <div className="font-mono text-sm whitespace-pre-wrap bg-muted text-muted-foreground rounded p-2">
        {description}
      </div>
    </div>
  );
};

export default ModCardDescription;