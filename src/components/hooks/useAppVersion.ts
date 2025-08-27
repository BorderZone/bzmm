import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AppVersion {
  version: string;
}

export function useAppVersion() {
  const [version, setVersion] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const getVersion = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await invoke<AppVersion>('get_app_version');
      setVersion(result.version);
      setError(null);
    } catch (err) {
      setError(`Failed to get app version: ${err}`);
      console.error(err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    getVersion();
  }, [getVersion]);

  return { version, isLoading, error };
}
