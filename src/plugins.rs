use rhai::{AST, Dynamic, Engine, Scope};
use std::sync::{Arc, Mutex};
use std::{error::Error, fs, path::Path};

#[derive(Clone)]
pub struct PluginManager(Arc<Mutex<PluginRuntime>>);

struct PluginRuntime {
    engine: Engine,
    plugins: Vec<(String, AST)>,
}

impl PluginManager {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let path = path.as_ref();
        fs::create_dir_all(path)?;
        let engine = Engine::new();
        let mut plugins = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file = entry.path();
            if file.extension().and_then(|x| x.to_str()) != Some("rhai") {
                continue;
            }
            let source = fs::read_to_string(&file)?;
            match engine.compile(&source) {
                Ok(ast) => plugins.push((file.display().to_string(), ast)),
                Err(error) => eprintln!("Plugin {} failed to compile: {error}", file.display()),
            }
        }
        println!("Loaded {} Lithum script plugin(s)", plugins.len());
        Ok(Self(Arc::new(Mutex::new(PluginRuntime {
            engine,
            plugins,
        }))))
    }

    pub fn enable_all(&self) {
        self.call("on_enable", vec![]);
    }

    pub fn fire_tick(&self, tick: u64) {
        self.call("on_tick", vec![Dynamic::from(tick as i64)]);
    }

    pub fn fire_command(&self, command: &str) {
        self.call(
            "on_console_command",
            vec![Dynamic::from(command.to_owned())],
        );
    }

    fn call(&self, function: &str, args: Vec<Dynamic>) {
        let Ok(runtime) = self.0.lock() else {
            eprintln!("Plugin lock poisoned");
            return;
        };
        for (name, ast) in &runtime.plugins {
            let mut scope = Scope::new();
            if function != "on_enable" {
                scope.push("tick", args.first().cloned().unwrap_or(Dynamic::UNIT));
            }
            if let Err(error) =
                runtime
                    .engine
                    .call_fn::<Dynamic>(&mut scope, ast, function, args.clone())
            {
                if !error.to_string().contains("Function not found") {
                    eprintln!("Plugin {name} callback {function} failed: {error}");
                }
            }
        }
    }
}
