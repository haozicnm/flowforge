//! Dynamic node plugin system.
//!
//! Third-party nodes ship as shared libraries (.so / .dll / .dylib).
//! Each library exports `ff_create_executor`:
//!   extern "C" fn ff_create_executor() -> *mut dyn NodeExecutor
//!
//! On startup, the plugin manager scans `plugins/` and loads all found libraries.
//! Use `export_plugin!` macro in plugin crates to generate the C-ABI entry point.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::nodes::traits::NodeExecutor;

type CreateExecutorFn = unsafe extern "C" fn() -> *mut dyn NodeExecutor;

/// Loaded plugin library (keeps the library alive while registered).
struct LoadedPlugin {
    _lib: libloading::Library,
}

pub struct PluginManager {
    plugins_dir: PathBuf,
    loaded: Mutex<Vec<LoadedPlugin>>,
}

impl PluginManager {
    pub fn new(plugins_dir: impl Into<PathBuf>) -> Self {
        Self {
            plugins_dir: plugins_dir.into(),
            loaded: Mutex::new(Vec::new()),
        }
    }

    /// Scan plugins dir, load all valid libraries, and register into the registry.
    /// Returns the number of plugins loaded.
    pub fn scan_and_load(
        &self,
        registry: &crate::nodes::registry::NodeRegistry,
    ) -> Result<usize, String> {
        let dir = &self.plugins_dir;
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
            return Ok(0);
        }

        let entries = std::fs::read_dir(dir).map_err(|e| format!("read: {}", e))?;
        let mut count = 0;

        for entry in entries {
            let entry = entry.map_err(|e| format!("entry: {}", e))?;
            let path = entry.path();

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_lib = (cfg!(target_os = "windows") && ext == "dll")
                || (cfg!(target_os = "macos") && (ext == "dylib" || ext == "so"))
                || (cfg!(target_os = "linux") && ext == "so");

            if !is_lib {
                continue;
            }

            match Self::load_and_register(&path, registry) {
                Ok(lib) => {
                    let mut loaded = self.loaded.lock().map_err(|e| format!("lock: {}", e))?;
                    loaded.push(lib);
                    tracing::info!("Loaded plugin: {}", path.display());
                    count += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to load plugin {}: {}", path.display(), e);
                }
            }
        }

        Ok(count)
    }

    fn load_and_register(
        path: &std::path::Path,
        registry: &crate::nodes::registry::NodeRegistry,
    ) -> Result<LoadedPlugin, String> {
        // SAFETY: The loaded library and its symbols are used within safe Rust
        // constraints. The library stays loaded for the process lifetime (via
        // LoadedPlugin holding the Library handle).
        unsafe {
            let lib = libloading::Library::new(path).map_err(|e| format!("dlopen: {}", e))?;

            let creator: libloading::Symbol<CreateExecutorFn> = lib
                .get(b"ff_create_executor")
                .map_err(|e| format!("missing symbol: {}", e))?;

            let executor_ptr = creator();
            if executor_ptr.is_null() {
                return Err("ff_create_executor returned null".into());
            }

            let executor: Box<dyn NodeExecutor> = Box::from_raw(executor_ptr);
            registry.register_boxed(executor);

            Ok(LoadedPlugin { _lib: lib })
        }
    }
}

/// Macro for plugin crates to export the entry point.
///
/// ```rust,ignore
/// flowforge::export_plugin!(MyNode);
/// ```
#[macro_export]
macro_rules! export_plugin {
    ($ty:ty) => {
        #[no_mangle]
        pub extern "C" fn ff_create_executor(
        ) -> *mut dyn $crate::nodes::traits::NodeExecutor {
            let executor: $ty = <$ty>::default();
            Box::into_raw(Box::new(executor))
        }
    };
}
