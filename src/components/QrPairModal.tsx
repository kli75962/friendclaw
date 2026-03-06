import { useEffect, useState } from 'react';
import { QrCode, Camera } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { scan, Format } from '@tauri-apps/plugin-barcode-scanner';
import { Modal } from './Modal';
import { useSession } from '../hooks/useSession';

interface QrPairModalProps {
  onClose: () => void;
}

export function QrPairModal({ onClose }: QrPairModalProps) {
  const { session, refresh } = useSession();
  const isAndroid = session?.device.device_type === 'android';

  return (
    <Modal title={isAndroid ? 'Scan QR to Link' : 'Show QR to Link'} onClose={onClose}>
      {isAndroid ? (
        <ScanView onPaired={() => { refresh(); onClose(); }} />
      ) : (
        <ShowQrView />
      )}
    </Modal>
  );
}

/** Desktop: display QR code SVG for the phone to scan. */
function ShowQrView() {
  const [svg, setSvg] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    invoke<string>('get_qr_pair_svg')
      .then(setSvg)
      .catch((e) => setError(String(e)));
  }, []);

  if (error) {
    return <p className="text-xs text-red-400 text-center py-4">❌ {error}</p>;
  }

  if (!svg) {
    return (
      <div className="flex items-center justify-center py-8">
        <QrCode size={48} className="text-gray-600 animate-pulse" />
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-4">
      <div
        className="bg-white rounded-xl p-3"
        dangerouslySetInnerHTML={{ __html: svg }}
      />
      <p className="text-xs text-gray-500 text-center">
        Open the app on your phone and scan this QR code to link.
      </p>
    </div>
  );
}

/** Android: scan QR code and auto-pair. */
function ScanView({ onPaired }: { onPaired: () => void }) {
  const [status, setStatus] = useState<'idle' | 'scanning' | 'pairing' | 'done' | 'error'>('idle');
  const [error, setError] = useState('');

  async function handleScan() {
    setStatus('scanning');
    setError('');
    try {
      const result = await scan({ formats: [Format.QRCode], windowed: false });
      const raw = result.content;

      let parsed: { address: string; hash_key: string };
      try {
        parsed = JSON.parse(raw);
      } catch {
        throw new Error('Invalid QR code — not a valid pairing code.');
      }

      if (!parsed.address || !parsed.hash_key) {
        throw new Error('Invalid QR code — missing address or hash key.');
      }

      setStatus('pairing');

      // Atomic: peer is verified, hash key set, and device saved in one command.
      // If the ping fails, the local hash key is left unchanged.
      await invoke('pair_from_qr', { address: parsed.address, hashKey: parsed.hash_key });

      setStatus('done');
      onPaired();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setStatus('error');
    }
  }

  return (
    <div className="flex flex-col items-center gap-4">
      {status === 'idle' && (
        <button
          onClick={handleScan}
          className="flex items-center gap-2 px-6 py-3 rounded-xl bg-purple-600 text-sm font-medium text-white hover:bg-purple-500 transition-colors"
        >
          <Camera size={16} />
          Scan QR Code
        </button>
      )}

      {status === 'scanning' && (
        <p className="text-sm text-gray-400 animate-pulse">Opening camera…</p>
      )}

      {status === 'pairing' && (
        <p className="text-sm text-gray-400 animate-pulse">Linking device…</p>
      )}

      {status === 'error' && (
        <div className="flex flex-col items-center gap-3">
          <p className="text-xs text-red-400 text-center">❌ {error}</p>
          <button
            onClick={handleScan}
            className="flex items-center gap-2 px-5 py-2 rounded-xl bg-[#2C2C2C] text-sm text-gray-200 hover:bg-[#383838] transition-colors"
          >
            <Camera size={14} />
            Try again
          </button>
        </div>
      )}

      <p className="text-xs text-gray-500 text-center">
        Point your camera at the QR code shown on your PC.
      </p>
    </div>
  );
}
