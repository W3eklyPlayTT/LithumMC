use lithummc_plugin_api::{Context, Plugin, PluginMetadata, logging, register_plugin};

struct HelloLithumMC;

impl Plugin for HelloLithumMC {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "hello-lithummc".into(),
            version: "0.1.0".into(),
            authors: vec!["LithumMC example".into()],
            description: "A minimal LithumMC WebAssembly plugin".into(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
        }
    }

    fn on_load(&self, _context: Context) -> Result<(), String> {
        logging::log(logging::LogLevel::Info, "Hello from a LithumMC plugin!");
        Ok(())
    }
}

register_plugin!(HelloLithumMC);
