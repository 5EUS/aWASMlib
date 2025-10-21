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
    /// Create a new Aggregator with the given Config.
    pub async fn new(config: &Config) -> Result<Self> {
        let db = Database::new(config).await?;
        let pm = PluginManager::new().await?;
        Ok(Self { db, pm })
    }
}