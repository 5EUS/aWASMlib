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

use anyhow::Result;

/// High-level handle to library functionality
pub struct Handle {
    pub rt: tokio::runtime::Runtime,
    pub agg: Aggregator,
    pub config: Config,
}

impl Handle {

    /// Create a new Handle with optional database URL and plugins directory.
    /// If database_url is None, the ApplicationSupport directory will be used.
    /// If plugins_dir is None, the ApplicationSupport directory will be used.
    pub async fn new() -> Result<Self> {
        let config = Config::new();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let agg = Aggregator::new(&config).await?;
        Ok(Self { rt, agg, config })
    }

    /// Load plugins from the configured plugins directory.
    pub async fn load_plugins(&mut self) -> Result<()> {
        if let Some(plugins_dir) = &self.config.plugins_dir {
            self.agg.pm.load_plugins_from_directory(plugins_dir).await?;
        }
        Ok(())
    }
}