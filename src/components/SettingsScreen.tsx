import { invoke } from '@tauri-apps/api/core';
import { ArrowLeft, ChevronDown, Check, Save } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';

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
      <div className="flex-1 min-h-0 overflow-y-auto">

        {/* Section: Model */}
        <div className="mt-6">
          <p className="px-5 pb-2 text-xs font-semibold text-gray-500 uppercase tracking-widest">
            Model
          </p>

          <div className="bg-[#1E1F20] border-y border-[#2C2C2C]">
            <button
              onClick={() => setModelOpen((v) => !v)}
              className="w-full flex items-center justify-between px-5 py-4 text-sm hover:bg-[#252526] transition-colors"
            >
              <span className="text-gray-300">Active model</span>
              <span className="flex items-center gap-2 text-gray-400 font-mono text-xs">
                <span className="truncate max-w-[160px]">{model || 'None'}</span>
                <ChevronDown
                  size={15}
                  className={`shrink-0 transition-transform ${modelOpen ? 'rotate-180' : ''}`}
                />
              </span>
            </button>

            {modelOpen && (
              <>
                <div className="border-t border-[#2C2C2C]" />
                {availableModels.length === 0 ? (
                  <p className="px-5 py-4 text-sm text-gray-500">
                    No models found — is Ollama running?
                  </p>
                ) : (
                  availableModels.map((m, i) => (
                    <button
                      key={m}
                      onClick={() => { onModelChange(m); setModelOpen(false); }}
                      className={`w-full flex items-center justify-between px-5 py-3.5 text-sm font-mono transition-colors hover:bg-[#252526] ${
                        i < availableModels.length - 1 ? 'border-b border-[#2C2C2C]' : ''
                      }`}
                    >
                      <span className={m === model ? 'text-purple-400' : 'text-gray-300'}>{m}</span>
                      {m === model && <Check size={14} className="text-purple-400 shrink-0" />}
                    </button>
                  ))
                )}
              </>
            )}
          </div>

          <p className="px-5 pt-2 text-xs text-gray-600">
            Models are loaded from your local Ollama instance.
          </p>
        </div>

        {/* Section: Memory */}
        <div className="mt-6 mb-8 flex flex-col">
          {/* Header row */}
          <div className="flex items-center justify-between px-5 pb-2">
            <p className="text-xs font-semibold text-gray-500 uppercase tracking-widest">
              Memory
            </p>
            {dirty && (
              <button
                onClick={handleSave}
                disabled={saving}
                className="flex items-center gap-1.5 text-xs text-blue-400 hover:text-blue-300 transition-colors disabled:opacity-50"
              >
                <Save size={12} />
                {saving ? 'Saving…' : 'Save'}
              </button>
            )}
            {saveMsg && !dirty && (
              <span className="text-xs text-green-400">{saveMsg}</span>
            )}
          </div>

          {/* Tab bar */}
          <div className="flex border-b border-[#2C2C2C] bg-[#1E1F20]">
            {MEMORY_TABS.map(({ file, label }) => (
              <button
                key={file}
                onClick={() => setActiveTab(file)}
                className={`px-4 py-2.5 text-xs font-medium border-b-2 transition-colors ${
                  activeTab === file
                    ? 'border-purple-500 text-purple-400'
                    : 'border-transparent text-gray-500 hover:text-gray-300'
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          {/* Description */}
          <p className="px-5 py-2 text-xs text-gray-600">
            {MEMORY_TABS.find((t) => t.file === activeTab)?.desc}
          </p>

          {/* Editable textarea */}
          <div className="px-5">
            <textarea
              ref={textareaRef}
              value={fileContent}
              onChange={(e) => { setFileContent(e.target.value); setDirty(true); setSaveMsg(''); }}
              className="w-full h-64 bg-[#1A1A1B] border border-[#2C2C2C] rounded-lg px-4 py-3 text-xs font-mono text-gray-300 focus:outline-none focus:border-[#444] resize-none leading-relaxed"
              spellCheck={false}
              placeholder={`No content in ${activeTab}`}
            />
          </div>

          <p className="px-5 pt-2 text-xs text-gray-600">
            The LLM can read and write these files using the{' '}
            <span className="font-mono text-gray-500">memory</span> tool during any conversation.
          </p>
        </div>

      </div>
    </div>
  );
}
