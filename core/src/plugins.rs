use std::collections::HashMap;

/// Plugin trait for pluggable adapters and extensions.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
}

pub struct PluginRegistry {
    registry: HashMap<String, Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self { Self { registry: HashMap::new() } }

    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) {
        self.registry.insert(plugin.name().to_string(), Box::new(plugin));
    }

    pub fn names(&self) -> Vec<String> { self.registry.keys().cloned().collect() }
}
