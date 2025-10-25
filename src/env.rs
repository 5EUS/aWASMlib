use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
pub struct Config {
    pub db_path: Option<PathBuf>,
    pub plugins_dir: Option<PathBuf>,
    pub run_migrations: bool,
    pub search_ttl_secs: u64,
    pub pages_ttl_secs: u64,
}

impl Config {
    /// Create a new Config, setting default environment variables if not already set.
    pub fn new() -> Self {
        let mut db_path: Option<PathBuf> = None;
        let mut plugins_dir: Option<PathBuf> = None;
        let mut run_migrations = true; // default to true
        let mut search_ttl_secs = 3600; // default to 1 hour
        let mut pages_ttl_secs = 86400; // default to 24 hours

        let _ = fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .try_init(); // initialize tracing subscriber if not already initialized

        if std::env::var("RUST_LOG").is_err() { 
            std::env::set_var(
                "RUST_LOG",
                "info,wasmtime_wasi_http=trace,wasmtime_wasi=info,awasmlib=debug",
            );
        } // default to info level logging, with more verbose logging for HTTP and WASI components

        if std::env::var("DATABASE_URL").is_err() {
            if let Some(proj_dirs) = directories::ProjectDirs::from("com", "fiveeus", "aWASMlib") {
                let app_support_dir = proj_dirs.data_dir();
                std::fs::create_dir_all(app_support_dir).ok();
                db_path = Some(app_support_dir.join("awasm.db"));
                std::env::set_var("DATABASE_URL", db_path.as_ref().unwrap().to_string_lossy().to_string());
            } else {
                // Fallback to a sensible default if ProjectDirs fails
                let fallback_path = PathBuf::from("awasm.db");
                db_path = Some(fallback_path.clone());
                std::env::set_var("DATABASE_URL", fallback_path.to_string_lossy().to_string());
            }
        } // set default database URL if not set

        if std::env::var("PLUGINS_DIR").is_err() {
            if let Some(proj_dirs) = directories::ProjectDirs::from("com", "fiveeus", "aWASMlib") {
                plugins_dir = Some(proj_dirs.data_dir().join("plugins"));
                std::fs::create_dir_all(plugins_dir.as_ref().unwrap()).ok();
                std::env::set_var("PLUGINS_DIR", plugins_dir.as_ref().unwrap().to_string_lossy().to_string());
            } else {
                // Fallback to a sensible default if ProjectDirs fails
                let fallback_plugins_dir = PathBuf::from("plugins");
                plugins_dir = Some(fallback_plugins_dir.clone());
                std::fs::create_dir_all(&fallback_plugins_dir).ok();
                std::env::set_var("PLUGINS_DIR", fallback_plugins_dir.to_string_lossy().to_string());
            }
        } // set default plugins directory if not set

        if std::env::var("RUN_MIGRATIONS").is_err() {
            std::env::set_var("RUN_MIGRATIONS", "true");
            run_migrations = true;
        } else if let Ok(val) = std::env::var("RUN_MIGRATIONS") {
            run_migrations = val == "true";
        } // determine whether to run migrations based on environment variable

        if std::env::var("AWASM_SEARCH_TTL_SECS").is_err() {
            std::env::set_var("AWASM_SEARCH_TTL_SECS", "3600");
            search_ttl_secs = 3600;
        } else if let Ok(val) = std::env::var("AWASM_SEARCH_TTL_SECS") {
            if let Ok(parsed) = val.parse::<u64>() {
                search_ttl_secs = parsed;
            }
        } // set search TTL from environment variable if provided

        if std::env::var("AWASM_PAGES_TTL_SECS").is_err() {
            std::env::set_var("AWASM_PAGES_TTL_SECS", "86400");
            pages_ttl_secs = 86400;
        } else if let Ok(val) = std::env::var("AWASM_PAGES_TTL_SECS") {
            if let Ok(parsed) = val.parse::<u64>() {
                pages_ttl_secs = parsed;
            }
        } // set pages TTL from environment variable if provided

        Self { 
            db_path, 
            plugins_dir, 
            run_migrations, 
            search_ttl_secs, 
            pages_ttl_secs 
        }
    }
}