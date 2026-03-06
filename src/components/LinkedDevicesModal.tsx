import { Smartphone, Monitor, Trash2, Plus, Copy, Check, QrCode } from 'lucide-react';
import { Modal } from './Modal';
import { QrPairModal } from './QrPairModal';
import { useSession } from '../hooks/useSession';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import type { SessionConfig } from '../types';

interface LinkedDevicesModalProps {
  onClose: () => void;
}

export function LinkedDevicesModal({ onClose }: LinkedDevicesModalProps) {
  const { session, refresh } = useSession();
  const [removing, setRemoving] = useState<string | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const [address, setAddress] = useState('');
  const [addStatus, setAddStatus] = useState<'idle' | 'loading' | 'error'>('idle');
  const [addError, setAddError] = useState('');
  const [localAddress, setLocalAddress] = useState('');
  const [copied, setCopied] = useState(false);
  const [showQr, setShowQr] = useState(false);

  useEffect(() => {
    invoke<string>('get_local_address').then(setLocalAddress).catch(() => {});
  }, []);

  if (!session) return null;

  async function handleCopyAddress() {
    await navigator.clipboard.writeText(localAddress);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  async function handleRemove(deviceId: string) {
    setRemoving(deviceId);
    try {
      await invoke('remove_paired_device', { deviceId });
      await refresh();
    } finally {
      setRemoving(null);
    }
  }

  async function handleAddPeer() {
    const trimmed = address.trim();
    if (!trimmed) return;
    setAddStatus('loading');
    setAddError('');
    try {
      await invoke<SessionConfig>('discover_and_pair', { address: trimmed });
      await refresh();
      setShowAdd(false);
      setAddress('');
      setAddStatus('idle');
    } catch (e) {
      setAddError(e instanceof Error ? e.message : String(e));
      setAddStatus('error');
    }
  }

  const peers = session.paired_devices;

  return (
    <Modal title="Linked Devices" onClose={onClose}>
      <div className="flex flex-col gap-3">
        {/* This device */}
        <div className="flex items-center gap-3 px-4 py-3 rounded-xl bg-[#1E1F20] border border-[#2C2C2C]">
          <Monitor size={16} className="text-purple-400 shrink-0" />
          <div className="flex-1 min-w-0">
            <p className="text-sm text-gray-200 font-medium truncate">{session.device.label}</p>
            {localAddress && (
              <button
                onClick={handleCopyAddress}
                className="flex items-center gap-1 text-xs text-gray-500 font-mono hover:text-gray-300 transition-colors mt-0.5"
              >
                {copied ? <Check size={10} className="text-green-400" /> : <Copy size={10} />}
                <span className={copied ? 'text-green-400' : ''}>{localAddress}</span>
              </button>
            )}
          </div>
          <span className="text-xs text-purple-400 bg-purple-500/10 px-2 py-0.5 rounded-full shrink-0">This device</span>
        </div>

        {/* Paired peers */}
        {peers.length === 0 ? (
          <p className="text-xs text-gray-500 text-center py-4">
            No paired devices yet. Share your hash key to link another device.
          </p>
        ) : (
          peers.map(peer => (
            <div key={peer.device_id} className="flex items-center gap-3 px-4 py-3 rounded-xl bg-[#1E1F20] border border-[#2C2C2C]">
              <Smartphone size={16} className="text-green-400 shrink-0" />
              <div className="flex-1 min-w-0">
                <p className="text-sm text-gray-200 font-medium truncate">{peer.label || peer.device_id}</p>
                <p className="text-xs text-gray-500 font-mono truncate">{peer.address}</p>
              </div>
              <button
                onClick={() => handleRemove(peer.device_id)}
                disabled={removing === peer.device_id}
                className="p-1.5 rounded-lg hover:bg-red-500/10 text-gray-600 hover:text-red-400 transition-colors disabled:opacity-40 shrink-0"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))
        )}

        {/* Hash key */}
        <div className="mt-1 px-4 py-3 rounded-xl bg-[#131314] border border-[#2C2C2C]">
          <p className="text-xs text-gray-500 mb-1">Shared hash key</p>
          <p className="text-xs font-mono text-gray-400 break-all leading-relaxed">
            {session.hash_key || <span className="text-gray-600">Not set</span>}
          </p>
        </div>

        {/* QR pairing */}
        <button
          onClick={() => setShowQr(true)}
          className="flex items-center justify-center gap-2 w-full py-2.5 rounded-xl bg-purple-600 text-sm font-medium text-white hover:bg-purple-500 transition-colors"
        >
          <QrCode size={15} />
          {session.device.device_type === 'android' ? 'Scan QR to link' : 'Show QR to link'}
        </button>

        {/* Add peer manually */}
        {showAdd ? (
          <div className="flex flex-col gap-2 px-4 py-3 rounded-xl bg-[#1E1F20] border border-[#2C2C2C]">
            <p className="text-xs text-gray-400">
              Enter the peer's address (e.g. <span className="font-mono text-gray-500">192.168.1.5:9876</span>)
            </p>
            {session.device.device_type === 'android' && (
              <p className="text-xs text-yellow-500/80">
                On an emulator, use <span className="font-mono">10.0.2.2:9876</span> to reach the PC host.
              </p>
            )}
            <input
              autoFocus
              type="text"
              value={address}
              onChange={(e) => { setAddress(e.target.value); setAddStatus('idle'); }}
              onKeyDown={(e) => { if (e.key === 'Enter') handleAddPeer(); }}
              placeholder="ip:port"
              className="w-full bg-[#131314] border border-[#2C2C2C] rounded-xl px-3 py-2 text-sm text-gray-200 placeholder:text-gray-600 focus:outline-none focus:border-[#444] font-mono"
            />
            {addStatus === 'error' && (
              <p className="text-xs text-red-400">❌ {addError}</p>
            )}
            <div className="flex gap-2">
              <button
                onClick={() => { setShowAdd(false); setAddress(''); setAddStatus('idle'); }}
                className="flex-1 py-2 rounded-xl bg-[#2C2C2C] text-xs text-gray-400 hover:bg-[#383838] transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleAddPeer}
                disabled={!address.trim() || addStatus === 'loading'}
                className="flex-1 py-2 rounded-xl bg-purple-600 text-xs font-medium text-white hover:bg-purple-500 transition-colors disabled:opacity-40"
              >
                {addStatus === 'loading' ? 'Connecting…' : 'Add device'}
              </button>
            </div>
          </div>
        ) : (
          <button
            onClick={() => setShowAdd(true)}
            className="flex items-center justify-center gap-2 w-full py-2.5 rounded-xl border border-dashed border-[#2C2C2C] text-xs text-gray-500 hover:text-gray-300 hover:border-[#444] transition-colors"
          >
            <Plus size={13} />
            Add device by address
          </button>
        )}
      </div>

      {showQr && <QrPairModal onClose={() => { setShowQr(false); refresh(); }} />}
    </Modal>
  );
}
