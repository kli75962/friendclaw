// Auto-generated agent registry (skills + tools) embedded at compile time.
include!(concat!(env!("OUT_DIR"), "/agent_registry.rs"));

use serde_json::Value;

/// All SKILL.md files joined as a single text block for the system prompt.
/// Each file already contains agentskills.io-spec frontmatter (name + description)
/// followed by full markdown instructions — no extra headers needed.
pub fn build_skills_prompt() -> String {
    let mut out = String::new();
    for (i, s) in SKILLS.iter().enumerate() {
        if i > 0 { out.push_str("\n\n---\n\n"); }
        out.push_str(s.content);
    }
    out
}

/// All tool JSON schemas parsed and collected for Ollama's `tools` field.
pub fn load_tool_schemas() -> Vec<Value> {
    TOOLS
        .iter()
        .filter_map(|t| serde_json::from_str(t.content).ok())
        .collect()
}
