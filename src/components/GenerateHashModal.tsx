import { useState } from 'react';
import { Copy, Check } from 'lucide-react';
import { Modal } from './Modal';
import { useSession } from '../hooks/useSession';

interface GenerateHashModalProps {
  onClose: () => void;
}

type Step = 'view' | 'confirm' | 'done';

/** Mask a key: show first 5 chars then asterisks. */
function maskKey(key: string): string {
  if (key.length <= 5) return key;
  return key.slice(0, 5) + '*'.repeat(Math.min(key.length - 5, 20));
}

/** Generate a cryptographically random passphrase (UUID v4 without dashes). */
function generatePassphrase(): string {
  return crypto.randomUUID().replace(/-/g, '');
}

/** Popup to view the current hash key and optionally generate a new one. */
export function GenerateHashModal({ onClose }: GenerateHashModalProps) {
  const { session, setPassphrase } = useSession();
  const [step, setStep] = useState<Step>('view');
  const [newPassphrase, setNewPassphrase] = useState('');
  const [loading, setLoading] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState('');

  const currentKey = session?.hash_key ?? '';

  async function handleConfirmGenerate() {
    setLoading(true);
    setError('');
    const passphrase = generatePassphrase();
    try {
      await setPassphrase(passphrase);
      setNewPassphrase(passphrase);
      setStep('done');
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleCopy() {
    await navigator.clipboard.writeText(newPassphrase);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <Modal title="Hash Key" onClose={onClose}>
      {/* ── View: show current key ── */}
      {step === 'view' && (
        <div className="flex flex-col gap-5">
          <div>
            <p className="text-xs text-gray-500 mb-1.5">Current key</p>
            <p className="font-mono text-sm text-gray-200 bg-[#131314] border border-[#2C2C2C] rounded-xl px-4 py-3 tracking-widest">
              {currentKey ? maskKey(currentKey) : <span className="text-gray-600">Not set</span>}
            </p>
          </div>

          <button
            onClick={() => setStep('confirm')}
            className="w-full py-2.5 rounded-xl bg-[#2C2C2C] text-sm text-gray-200 hover:bg-[#383838] transition-colors"
          >
            Generate new key
          </button>
        </div>
      )}

      {/* ── Confirm: warn user ── */}
      {step === 'confirm' && (
        <div className="flex flex-col gap-5">
          <div className="rounded-xl bg-yellow-500/10 border border-yellow-500/20 px-4 py-3">
            <p className="text-xs text-yellow-400 leading-relaxed">
              ⚠️ Generating a new key will abandon the current one.
              Any device or Discord user linked with the old key will be disconnected.
            </p>
          </div>

          {error && (
            <p className="text-xs text-red-400">❌ {error}</p>
          )}

          <div className="flex gap-3">
            <button
              onClick={() => setStep('view')}
              disabled={loading}
              className="flex-1 py-2.5 rounded-xl bg-[#2C2C2C] text-sm text-gray-300 hover:bg-[#383838] transition-colors disabled:opacity-40"
            >
              No
            </button>
            <button
              onClick={handleConfirmGenerate}
              disabled={loading}
              className="flex-1 py-2.5 rounded-xl bg-purple-600 text-sm font-medium text-white hover:bg-purple-500 transition-colors disabled:opacity-40"
            >
              {loading ? 'Generating…' : 'Yes, generate'}
            </button>
          </div>
        </div>
      )}

      {/* ── Done: show new passphrase ── */}
      {step === 'done' && (
        <div className="flex flex-col gap-5">
          <div>
            <p className="text-xs text-gray-500 mb-1.5">Your new key — share this to link devices</p>
            <p className="font-mono text-xs text-green-300 bg-[#131314] border border-[#2C2C2C] rounded-xl px-4 py-3 break-all leading-relaxed">
              {newPassphrase}
            </p>
          </div>

          <p className="text-xs text-gray-500">
            In Discord, type{' '}
            <span className="font-mono text-gray-400">!link {newPassphrase}</span>{' '}
            to link your account.
          </p>

          <button
            onClick={handleCopy}
            className="w-full flex items-center justify-center gap-2 py-2.5 rounded-xl bg-[#2C2C2C] text-sm text-gray-200 hover:bg-[#383838] transition-colors"
          >
            {copied ? (
              <>
                <Check size={14} className="text-green-400" />
                <span className="text-green-400">Copied!</span>
              </>
            ) : (
              <>
                <Copy size={14} />
                Copy key
              </>
            )}
          </button>

          <button
            onClick={onClose}
            className="w-full py-2 text-xs text-gray-600 hover:text-gray-400 transition-colors"
          >
            Close
          </button>
        </div>
      )}
    </Modal>
  );
}
