use anyhow::Result;
use crate::database::Database;
use crate::plugins::PluginManager;

/// Aggregator owns database + plugins and provides higher-level cached & persisted operations.
pub struct Aggregator {
    pub db: Database,
    pub pm: PluginManager,
}

impl Aggregator {
    /// Create a new Aggregator.
    pub async fn new() -> Result<Self> {
        let db = Database::new().await?;
        let pm = PluginManager::new().await?;
        Ok(Self { db, pm })
    }
}