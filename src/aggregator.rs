use crate::database::Database;
use crate::pluginmanager::PluginManager;

/// Aggregator owns database + plugins and provides higher-level cached & persisted operations.
pub struct Aggregator {
    pub db: Database,
    pub pm: PluginManager,
}