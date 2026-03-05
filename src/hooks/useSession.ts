import { invoke } from '@tauri-apps/api/core';
import { useCallback, useEffect, useState } from 'react';
import type { SessionConfig } from '../types';

export function useSession() {
  const [session, setSession] = useState<SessionConfig | null>(null);

  const refresh = useCallback(async () => {
    try {
      const s = await invoke<SessionConfig>('get_session');
      setSession(s);
    } catch {
      // session not available on this build
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  /** Set the session hash key (must be the 64-char hex key from the app). */
  const setHashKey = useCallback(async (hashKey: string): Promise<SessionConfig> => {
    const s = await invoke<SessionConfig>('set_session_hash_key', { hashKey });
    setSession(s);
    return s;
  }, []);

  return { session, refresh, setHashKey };
}
