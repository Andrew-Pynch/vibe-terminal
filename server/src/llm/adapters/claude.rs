use super::ProviderAdapter;

pub struct ClaudeAdapter;

impl ProviderAdapter for ClaudeAdapter {
    fn get_command(&self) -> String {
        "claude".to_string()
    }

    fn get_args(&self, prompt_file: &str, model: &str) -> Vec<String> {
        // Claude Code CLI (claude) usually takes the prompt as a positional argument
        // or reads from stdin.
        // For non-interactive, we use -p/--print.
        // We also use --dangerously-skip-permissions to be autonomous.
        // We can read the prompt from the file using `cat` or passing the file path if supported.
        // `claude "prompt"` works.
        // To include context, we might need to construct a string.
        // But for our Vibe pattern, we want it to read INSTRUCTION.md.
        
        // A robust way is: claude -p --dangerously-skip-permissions "Please read INSTRUCTION.md and follow the instructions."
        
        vec![
            "-p".to_string(),
            "--dangerously-skip-permissions".to_string(),
            // We might want to pass the model if supported, `claude --model ...`?
            // The help text said: --model <model>
            // "--model".to_string(),
            // model.to_string(),
            format!("Please read {} and follow the instructions.", prompt_file),
        ]
    }
}
