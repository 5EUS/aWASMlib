pub mod aggregator;
pub mod pluginmanager;
pub mod database;

use aggregator::Aggregator;

/// High-level handle to library functionality
pub struct Handle {
    pub agg: Aggregator,
}