pub mod aggregator;
pub mod plugins;
pub mod database;
pub mod dao;
pub mod env;
/// Prelude re-exports commonly used types for easy import
pub mod prelude {
    pub use crate::aggregator::Aggregator;
    pub use crate::database::Database;
    pub use crate::plugins::*; // includes wit types
}

use crate::env::Config;
use aggregator::Aggregator;

use anyhow::{Result, bail};

/// High-level handle to library functionality
pub struct Handle {
    pub agg: Aggregator,
    pub config: Config,
}

impl Handle {
    /// Create a new Handle with optional database URL and plugins directory.
    /// This initializes the underlying Aggregator and configuration, connects to the database,
    /// and prepares the plugin manager.
    /// If database_url is None, the ApplicationSupport directory will be used.
    /// If plugins_dir is None, the ApplicationSupport directory will be used.
    /// run_migrations defaults to true.
    pub async fn new() -> Result<Self> {
        let config = Config::new();
        let agg = Aggregator::new(&config).await?;
        Ok(Self { agg, config })
    }

    /// Load plugins from the configured plugins directory. Specifically, it registers each plugin artifact found
    /// in the directory with the PluginManager for lazy loading.
    pub async fn load_plugins(&mut self) -> Result<()> {
        match &self.config.plugins_dir {
            Some(plugins_dir) => self.agg.pm.load_plugins_from_directory(plugins_dir).await,
            None => bail!("No plugins directory configured"),
        }
    }
}