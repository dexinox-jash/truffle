//! Plugin Registry

use std::collections::HashMap;

pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn super::Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
}
