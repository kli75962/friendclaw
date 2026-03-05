/** A single message in the Ollama chat conversation. */
export interface Message {
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
}

/** Payload emitted by the `ollama-stream` Tauri event for every token. */
export interface StreamPayload {
  content: string;
  done: boolean;
}

/** Emitted by `agent-status` when the LLM is executing a tool. */
export interface AgentStatusPayload {
  message: string;
}

/** Session config returned by the Rust `get_session` command. */
export interface SessionConfig {
  device: {
    device_id: string;
    device_type: 'android' | 'desktop';
    label: string;
  };
  hash_key: string;
  paired_devices: { device_id: string; address: string; label: string }[];
  bridge_port: number;
}
