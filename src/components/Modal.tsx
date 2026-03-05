import type { ReactNode } from 'react';
import { X } from 'lucide-react';

interface ModalProps {
  title: string;
  onClose: () => void;
  children: ReactNode;
}

/** Reusable centered popup overlay. */
export function Modal({ title, onClose, children }: ModalProps) {
  return (
    <div
      style={{ zIndex: 60 }}
      className="fixed inset-0 flex items-center justify-center bg-black/60 px-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-sm bg-[#1E1F20] border border-[#2C2C2C] rounded-2xl shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-5 py-4 border-b border-[#2C2C2C]">
          <span className="text-sm font-semibold text-[#E3E3E3]">{title}</span>
          <button
            onClick={onClose}
            className="p-1 rounded-full hover:bg-[#2C2C2C] transition-colors"
          >
            <X size={16} className="text-gray-400" />
          </button>
        </div>

        {/* Body */}
        <div className="px-5 py-5">{children}</div>
      </div>
    </div>
  );
}
