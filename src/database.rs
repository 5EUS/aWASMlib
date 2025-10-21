use std::path::PathBuf;
use anyhow::Result;

/// Manages persistent storage of aggregated data
pub struct Database {

}

impl Database {
    pub async fn new() -> Result<Self> {
        Ok(Database {})
    }

    pub async fn connect(&self, database_url: &PathBuf) -> Result<()> {
        // TODO Implement connection logic here
        Ok(())
    }
}