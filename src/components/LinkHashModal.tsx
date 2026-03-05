import { useState } from 'react';
import { Modal } from './Modal';
import { useSession } from '../hooks/useSession';

interface LinkHashModalProps {
  onClose: () => void;
}

/** Popup that lets the user paste the system-generated hash key to link this device. */
export function LinkHashModal({ onClose }: LinkHashModalProps) {
  const { setHashKey } = useSession();
  const [value, setValue] = useState('');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');
  const [errorMsg, setErrorMsg] = useState('');

  const isValidKey = (v: string) => v.length === 64 && /^[0-9a-f]+$/i.test(v);

  async function handleLink() {
    const trimmed = value.trim();
    if (!isValidKey(trimmed)) {
      setErrorMsg('Must be the 64-character hex key shown in the app.');
      setStatus('error');
      return;
    }
    setStatus('loading');
    try {
      await setHashKey(trimmed);
      setStatus('success');
    } catch (e) {
      setErrorMsg(e instanceof Error ? e.message : String(e));
      setStatus('error');
    }
  }

  return (
    <Modal title="Link Hash Key" onClose={onClose}>
      {status === 'success' ? (
        <div className="flex flex-col items-center gap-4">
          <div className="flex items-center justify-center w-12 h-12 rounded-full bg-green-500/15">
            <span className="text-2xl">✅</span>
          </div>
          <p className="text-sm text-green-400 font-medium text-center">
            Linked successfully!
          </p>
          <p className="text-xs text-gray-500 text-center">
            This device now shares the session key you entered.
          </p>
          <button
            onClick={onClose}
            className="w-full py-2.5 rounded-xl bg-[#2C2C2C] text-sm text-gray-300 hover:bg-[#383838] transition-colors"
          >
            Done
          </button>
        </div>
      ) : (
        <div className="flex flex-col gap-4">
          <p className="text-xs text-gray-400">
            Paste the 64-character hash key shown under{' '}
            <span className="font-mono text-gray-500">Generate hash key</span> on another device.
          </p>

          <input
            autoFocus
            type="text"
            value={value}
            onChange={(e) => { setValue(e.target.value.toLowerCase()); setStatus('idle'); }}
            onKeyDown={(e) => { if (e.key === 'Enter') handleLink(); }}
            placeholder="64-character hex key…"
            className="w-full bg-[#131314] border border-[#2C2C2C] rounded-xl px-4 py-3 text-sm text-gray-200 placeholder:text-gray-600 focus:outline-none focus:border-[#444] font-mono"
          />

          {status === 'error' && (
            <p className="text-xs text-red-400">❌ {errorMsg || 'Failed to link. Try again.'}</p>
          )}

          <button
            onClick={handleLink}
            disabled={!isValidKey(value.trim()) || status === 'loading'}
            className="w-full py-2.5 rounded-xl bg-purple-600 text-sm font-medium text-white hover:bg-purple-500 transition-colors disabled:opacity-40"
          >
            {status === 'loading' ? 'Linking…' : 'Link'}
          </button>
        </div>
      )}
    </Modal>
  );
}
