import { invoke } from '@tauri-apps/api/core';
import { ArrowLeft, ChevronDown, Check, Save } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { GenerateHashModal } from './GenerateHashModal';
import { LinkHashModal } from './LinkHashModal';

// ── Types ───────────────────────────────────────────────────────────────────────────────

type MemoryFile = 'core.md' | 'conversations.jsonl';

interface SettingsScreenProps {
  model: string;
  availableModels: string[];
  onModelChange: (model: string) => void;
  onBack: () => void;
}

// ── Memory file tab config ─────────────────────────────────────────────────────────

const MEMORY_TABS: { file: MemoryFile; label: string; desc: string }[] = [
  {
    file: 'core.md',
    label: 'Core',
    desc: 'Injected into every prompt — keep short. User preferences, name, language.',
  },
  {
    file: 'conversations.jsonl',
    label: 'Recall',
    desc: 'Last 50 conversation summaries (JSONL). Used by the LLM to search past sessions.',
  },
];

/** Full-screen settings page styled like a native mobile settings app. */
export function SettingsScreen({ model, availableModels, onModelChange, onBack }: SettingsScreenProps) {
  const [modelOpen, setModelOpen] = useState(false);
  const [showLinkHash, setShowLinkHash] = useState(false);
  const [showGenerateHash, setShowGenerateHash] = useState(false);

  // Memory tab state
  const [activeTab, setActiveTab] = useState<MemoryFile>('core.md');
  const [fileContent, setFileContent] = useState('');
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Load file content when tab changes
  useEffect(() => {
    setDirty(false);
    setSaveMsg('');
    invoke<string>('get_memory_file', { filename: activeTab })
      .then((content) => setFileContent(content))
      .catch(() => setFileContent(''));
  }, [activeTab]);

  async function handleSave() {
    setSaving(true);
    try {
      await invoke('set_memory_file', { filename: activeTab, content: fileContent });
      setDirty(false);
      setSaveMsg('Saved');
      setTimeout(() => setSaveMsg(''), 2000);
    } catch {
      setSaveMsg('Error saving');
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col h-screen bg-[#131314] text-[#E3E3E3]">
      {showLinkHash && <LinkHashModal onClose={() => setShowLinkHash(false)} />}
      {showGenerateHash && <GenerateHashModal onClose={() => setShowGenerateHash(false)} />}
      {/* Header */}
      <div className="flex items-center gap-3 px-2 py-3 border-b border-[#2C2C2C]">
        <button
          onClick={onBack}
          className="p-2 hover:bg-[#2C2C2C] rounded-full transition-colors"
        >
          <ArrowLeft size={22} className="text-gray-400" />
        </button>
        <h1 className="text-lg font-semibold">Settings</h1>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 overflow-y-auto px-4 custom-scrollbar">
        <div className="max-w-2xl mx-auto pb-12">
          {/* Section: Model */}
          <div className="mt-6">
            <p className="px-2 pb-2 text-xs font-semibold text-gray-400 uppercase tracking-widest">
              Model
            </p>

            <div className="bg-[#1E1F20] border border-[#2C2C2C] rounded-2xl overflow-hidden shadow-sm">
              <button
                onClick={() => setModelOpen((v) => !v)}
                className="w-full flex items-center justify-between px-4 py-4 text-sm hover:bg-[#252526] transition-colors"
              >
                <span className="text-gray-200 font-medium">Active model</span>
                <span className="flex items-center gap-2 text-gray-400 font-mono text-xs bg-[#2C2C2C]/50 px-2 py-1 rounded-md">
                  <span className="truncate max-w-[160px]">{model || 'None'}</span>
                  <ChevronDown
                    size={14}
                    className={`shrink-0 transition-transform ${modelOpen ? 'rotate-180' : ''}`}
                  />
                </span>
              </button>

              {modelOpen && (
                <div className="bg-[#1A1A1B]">
                  <div className="h-px bg-[#2C2C2C] mx-4" />
                  {availableModels.length === 0 ? (
                    <p className="px-4 py-4 text-sm text-gray-500 text-center">
                      No models found — is Ollama running?
                    </p>
                  ) : (
                    <div className="py-2">
                      {availableModels.map((m) => (
                        <button
                          key={m}
                          onClick={() => { onModelChange(m); setModelOpen(false); }}
                          className="w-full flex items-center justify-between px-4 py-3 text-sm font-mono transition-colors hover:bg-[#252526]/80"
                        >
                          <span className={m === model ? 'text-purple-400 font-semibold' : 'text-gray-400 hover:text-gray-300'}>{m}</span>
                          {m === model && <Check size={16} className="text-purple-400 shrink-0" />}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </div>

            <p className="px-2 pt-2 text-xs text-gray-500">
              Models are loaded from your local Ollama instance.
            </p>
          </div>

          {/* Section: Session */}
          <div className="mt-8">
            <p className="px-2 pb-2 text-xs font-semibold text-gray-400 uppercase tracking-widest">
              Session
            </p>
            <div className="bg-[#1E1F20] border border-[#2C2C2C] rounded-2xl overflow-hidden shadow-sm">
              <button
                onClick={() => setShowLinkHash(true)}
                className="w-full flex items-center justify-between px-4 py-4 text-sm hover:bg-[#252526] transition-colors group"
              >
                <span className="text-gray-200 font-medium group-hover:text-white transition-colors">Link hash key</span>
                <span className="text-xs text-gray-500 group-hover:text-gray-400 transition-colors bg-[#2C2C2C]/30 px-2 py-1 rounded">Paste key to join</span>
              </button>
              <div className="h-px bg-[#2C2C2C] mx-4" />
              <button
                onClick={() => setShowGenerateHash(true)}
                className="w-full flex items-center justify-between px-4 py-4 text-sm hover:bg-[#252526] transition-colors group"
              >
                <span className="text-gray-200 font-medium group-hover:text-white transition-colors">Generate hash key</span>
                <span className="text-xs text-gray-500 group-hover:text-gray-400 transition-colors bg-[#2C2C2C]/30 px-2 py-1 rounded">Create shareable key</span>
              </button>
            </div>
            <p className="px-2 pt-2 text-xs text-gray-500">
              Share your key to link devices and sync sessions.
            </p>
          </div>

          {/* Section: Memory */}
          <div className="mt-8 mb-4 flex flex-col">
            {/* Header row */}
            <div className="flex items-center justify-between px-2 pb-2">
              <p className="text-xs font-semibold text-gray-400 uppercase tracking-widest">
                Memory
              </p>
              {dirty && (
                <button
                  onClick={handleSave}
                  disabled={saving}
                  className="flex items-center gap-1.5 text-xs font-medium text-blue-400 bg-blue-500/10 hover:bg-blue-500/20 px-2.5 py-1 rounded-full transition-colors disabled:opacity-50"
                >
                  <Save size={12} />
                  {saving ? 'Saving…' : 'Save changes'}
                </button>
              )}
              {saveMsg && !dirty && (
                <span className="text-xs font-medium text-green-400 bg-green-500/10 px-2.5 py-1 rounded-full">{saveMsg}</span>
              )}
            </div>

            <div className="bg-[#1E1F20] border border-[#2C2C2C] rounded-2xl overflow-hidden shadow-sm flex flex-col">
              {/* Tab bar */}
              <div className="flex border-b border-[#2C2C2C] bg-[#1a1b1c]">
                {MEMORY_TABS.map(({ file, label }) => (
                  <button
                    key={file}
                    onClick={() => setActiveTab(file)}
                    className={`flex-1 px-4 py-3 text-sm font-medium transition-colors relative ${
                      activeTab === file
                        ? 'text-purple-400'
                        : 'text-gray-500 hover:text-gray-300 hover:bg-[#252526]/50'
                    }`}
                  >
                    {label}
                    {activeTab === file && (
                      <div className="absolute bottom-0 left-0 right-0 h-0.5 bg-purple-500" />
                    )}
                  </button>
                ))}
              </div>

              {/* Description */}
              <div className="px-4 py-3 bg-[#1e1f20] border-b border-[#2C2C2C]/50">
                <p className="text-xs text-gray-400 leading-relaxed">
                  {MEMORY_TABS.find((t) => t.file === activeTab)?.desc}
                </p>
              </div>

              {/* Editable textarea */}
              <div className="p-4">
                <textarea
                  ref={textareaRef}
                  value={fileContent}
                  onChange={(e) => { setFileContent(e.target.value); setDirty(true); setSaveMsg(''); }}
                  className="w-full h-64 bg-[#141415] border border-[#2C2C2C] rounded-xl px-4 py-3 text-sm font-mono text-gray-300 focus:outline-none focus:border-purple-500/50 focus:ring-1 focus:ring-purple-500/50 resize-none leading-relaxed transition-all shadow-inner custom-scrollbar"
                  spellCheck={false}
                  placeholder={`No content in ${activeTab}`}
                />
              </div>
            </div>

            <p className="px-2 pt-3 text-xs text-gray-500 pb-8">
              The LLM can read and write these files using the{' '}
              <span className="font-mono bg-[#2C2C2C]/50 px-1 py-0.5 rounded text-gray-400">memory</span> tool during any conversation.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
