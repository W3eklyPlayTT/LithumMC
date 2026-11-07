use lithum_plugin_api::{Context, Plugin, PluginMetadata, logging, register_plugin};

struct HelloLithum;

impl Plugin for HelloLithum {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "hello-lithum".into(),
            version: "0.1.0".into(),
            authors: vec!["Lithum example".into()],
            description: "A minimal Lithum WebAssembly plugin".into(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
        }
    }

    fn on_load(&self, _context: Context) -> Result<(), String> {
        logging::log(logging::LogLevel::Info, "Hello from a Lithum plugin!");
        Ok(())
    }
}

register_plugin!(HelloLithum);
