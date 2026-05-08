//! Custom Model Management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

pub struct AiModelManager {
    models: HashMap<String, LoadedModel>,
}

pub struct LoadedModel {
    pub id: Uuid,
    pub name: String,
    pub config: ModelConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub path: PathBuf,
    pub context_length: usize,
}

impl AiModelManager {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }
    
    pub fn load_model(&mut self, config: ModelConfig) -> Uuid {
        let id = Uuid::new_v4();
        self.models.insert(id.to_string(), LoadedModel { id, name: config.name.clone(), config });
        id
    }
}
