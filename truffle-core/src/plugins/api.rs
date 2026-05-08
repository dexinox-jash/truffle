//! Plugin API Definitions

pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
}

pub struct PluginApi;
