use anyhow::Result;

use crate::env::Config;

/// Manages persistent storage of aggregated data
pub struct Database {

}

impl Database {
    pub async fn new(config: &Config) -> Result<Self> {
        Ok(Database {})
    }
}