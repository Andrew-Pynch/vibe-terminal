use std::{collections::HashMap, fs, path::Path};

use serde::Serialize;

#[derive(Clone)]
pub struct ProfileCatalog {
    profiles: HashMap<String, PromptProfile>,
}

#[derive(Clone, Serialize)]
pub struct ProfileSummary {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub modes: Vec<String>,
}

#[derive(Clone)]
pub struct PromptProfile {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub agents_doc: String,
    pub modes: HashMap<String, String>,
}

impl ProfileCatalog {
    pub fn load(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut profiles = HashMap::new();
        let dir = dir.as_ref();
        if !dir.exists() {
            anyhow::bail!("Prompt profile directory {} not found", dir.display());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().to_string();
            let name = title_case(&id);
            let agents_path = entry.path().join("AGENTS.md");
            let agents_doc = fs::read_to_string(&agents_path)?;
            let modes_dir = entry.path().join("MODES");
            let mut modes = HashMap::new();
            if modes_dir.exists() {
                for mode_file in fs::read_dir(&modes_dir)? {
                    let mode_file = mode_file?;
                    if !mode_file.file_type()?.is_file() {
                        continue;
                    }
                    let mode_id = mode_file
                        .path()
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                        .to_uppercase();
                    let content = fs::read_to_string(mode_file.path())?;
                    modes.insert(mode_id, content);
                }
            }
            profiles.insert(
                id.clone(),
                PromptProfile {
                    id,
                    name,
                    description: None,
                    agents_doc,
                    modes,
                },
            );
        }

        Ok(Self { profiles })
    }

    pub fn summaries(&self) -> Vec<ProfileSummary> {
        self.profiles
            .values()
            .map(|profile| ProfileSummary {
                id: profile.id.clone(),
                name: profile.name.clone(),
                description: profile.description.clone(),
                modes: profile.modes.keys().cloned().collect(),
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<PromptProfile> {
        self.profiles.get(id).cloned()
    }
}

fn title_case(value: &str) -> String {
    value
        .split(|c: char| c == '-' || c == '_' || c == ' ')
        .filter(|segment| !segment.is_empty())
        .map(|segment| {
            let mut chars = segment.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
