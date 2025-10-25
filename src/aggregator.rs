use anyhow::Result;
use crate::database::Database;
use crate::env::Config;
use crate::plugins::PluginManager;

/// Aggregator owns database + plugins and provides higher-level cached & persisted operations.
pub struct Aggregator {
    pub db: Database,
    pub pm: PluginManager,
}

impl Aggregator {
    /// Create a new Aggregator.
    pub async fn new(config: &Config) -> Result<Self> {
        let db_path = config.db_path.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Database path is required but was not found in the configuration.")
        })?;

        let db = Database::connect(db_path).await?;
        if config.run_migrations {
            db.run_migrations().await?;
        }
        
        let plugins_dir = config.plugins_dir.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Plugins directory is required but was not found in the configuration.")
        })?;

        let mut pm = PluginManager::new().await?;
        pm.load_plugins_from_directory(plugins_dir).await?;

        Ok(Self { 
            db, 
            pm, 
        })
    }
}