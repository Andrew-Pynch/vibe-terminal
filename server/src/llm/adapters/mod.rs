pub mod gemini;
pub mod claude;
pub mod codex;

use crate::llm::ProviderKind;

pub trait ProviderAdapter: Send + Sync {
    fn get_command(&self) -> String;
    fn get_args(&self, prompt_file: &str, model: &str) -> Vec<String>;
}

pub fn get_adapter(kind: &ProviderKind) -> Box<dyn ProviderAdapter> {
    match kind {
        ProviderKind::Gemini => Box::new(gemini::GeminiAdapter),
        ProviderKind::Claude => Box::new(claude::ClaudeAdapter),
        ProviderKind::Codex => Box::new(codex::CodexAdapter),
        ProviderKind::Dummy => Box::new(gemini::GeminiAdapter), // Fallback to Gemini for now
    }
}
