//! Plugin System Architecture

pub mod api;
pub mod registry;

pub use api::{Plugin, PluginApi};
pub use registry::PluginRegistry;
