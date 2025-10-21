use std::path::PathBuf;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Clone)]
pub struct Config {
    pub db_path: Option<PathBuf>,
    pub plugins_dir: Option<PathBuf>,
    pub run_migrations: bool,
}

impl Config {
    /// Create a new Config, setting default environment variables if not already set.
    pub fn new() -> Self {
        let mut db_path: Option<PathBuf> = None;
        let mut plugins_dir: Option<PathBuf> = None;
        let mut run_migrations = true; // default to true

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
                db_path = Some(app_support_dir.join("awasmlib.db"));
                std::env::set_var("DATABASE_URL", format!("sqlite://{}", db_path.as_ref().unwrap().to_string_lossy()));
            }
        } // default to a SQLite database in the Application Support directory

        if std::env::var("PLUGINS_DIR").is_err() {
            if let Some(proj_dirs) = directories::ProjectDirs::from("com", "fiveeus", "aWASMlib") {
                plugins_dir = Some(proj_dirs.data_dir().join("plugins"));
                std::fs::create_dir_all(plugins_dir.as_ref().unwrap()).ok();
                std::env::set_var("PLUGINS_DIR", plugins_dir.as_ref().unwrap().to_string_lossy().to_string());
            }
        } // default to a plugins directory in the Application Support directory

        if std::env::var("RUN_MIGRATIONS").is_err() {
            std::env::set_var("RUN_MIGRATIONS", "true");
            run_migrations = true;
        } else if let Ok(val) = std::env::var("RUN_MIGRATIONS") {
            run_migrations = val == "true";
        } // determine whether to run migrations based on environment variable

        Self { db_path, plugins_dir, run_migrations }
    }
}