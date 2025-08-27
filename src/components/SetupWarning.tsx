import React from 'react';
import { Card, CardContent, CardFooter, CardHeader, CardTitle, CardDescription } from './ui/card';
import { Button } from './ui/button';
import { AlertCircle } from 'lucide-react';

interface SetupWarningProps {
  title: string;
  description: string;
  actionLabel: string;
  onAction: () => void;
}

const SetupWarning: React.FC<SetupWarningProps> = ({
  title,
  description,
  actionLabel,
  onAction
}) => {
  return (
    <Card className="max-w-md mx-auto mt-8 border-amber-500 border-2">
      <CardHeader className="flex flex-row items-center gap-2">
        <AlertCircle className="h-6 w-6 text-amber-500" />
        <CardTitle className="text-lg">{title}</CardTitle>
      </CardHeader>
      <CardContent>
        <CardDescription className="text-base whitespace-pre-line text-foreground/80">
          {description}
        </CardDescription>
      </CardContent>
      <CardFooter>
        <Button onClick={onAction} className="w-full">
          {actionLabel}
        </Button>
      </CardFooter>
    </Card>
  );
};

export default SetupWarning;
