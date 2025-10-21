pub mod aggregator;
pub mod plugins;
pub mod database;
pub mod env;
/// Prelude re-exports commonly used types for easy import
pub mod prelude {
    pub use crate::aggregator::Aggregator;
    pub use crate::database::Database;
    pub use crate::plugins::*; // includes wit types
}

use crate::env::Config;
use aggregator::Aggregator;

use anyhow::{Result, bail, Context};

/// High-level handle to library functionality
pub struct Handle {
    pub agg: Aggregator,
    pub config: Config,
}

impl Handle {
    /// Create a new Handle with optional database URL and plugins directory.
    /// If database_url is None, the ApplicationSupport directory will be used.
    /// If plugins_dir is None, the ApplicationSupport directory will be used.
    /// run_migrations defaults to true.
    pub async fn new() -> Result<Self> {
        let config = Config::new();
        let agg = Aggregator::new().await?;
        Ok(Self { agg, config })
    }

    /// Connect to the connection string specified in the configuration.
    pub async fn connect(&self) -> Result<()> {
        if let Some(database_url) = &self.config.db_path {
            self.agg.db
                .connect(database_url)
                .await
                .context("Failed to connect to database")?;
        } else {
            bail!("No database URL configured");
        }
        Ok(())
    }

    /// Load plugins from the configured plugins directory.
    pub async fn load_plugins(&mut self) -> Result<()> {
        if let Some(plugins_dir) = &self.config.plugins_dir {
            self.agg.pm
                .load_plugins_from_directory(plugins_dir)
                .await
                .context("Failed to load plugins from directory")?;
        } else {
            bail!("No plugins directory configured");
        }
        Ok(())
    }
}