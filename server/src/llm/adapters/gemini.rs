use super::ProviderAdapter;

pub struct GeminiAdapter;

impl ProviderAdapter for GeminiAdapter {
    fn get_command(&self) -> String {
        "gemini".to_string()
    }

    fn get_args(&self, prompt_file: &str, model: &str) -> Vec<String> {
        vec![
            "-p".to_string(),
            prompt_file.to_string(),
            "--yolo".to_string(),
            "-m".to_string(),
            model.to_string(),
        ]
    }
}
