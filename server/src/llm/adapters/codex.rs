use super::ProviderAdapter;

pub struct CodexAdapter;

impl ProviderAdapter for CodexAdapter {
    fn get_command(&self) -> String {
        "codex".to_string()
    }

    fn get_args(&self, prompt_file: &str, _model: &str) -> Vec<String> {
        // Codex CLI: codex exec [OPTIONS] [PROMPT]
        // --dangerously-bypass-approvals-and-sandbox
        
        vec![
            "exec".to_string(),
            "--dangerously-bypass-approvals-and-sandbox".to_string(),
            // Codex accepts prompt as argument
            format!("Please read {} and follow the instructions.", prompt_file),
        ]
    }
}
