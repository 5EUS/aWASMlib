use std::path::PathBuf;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::sync::Once;
use sqlx::{any::{AnyConnectOptions, AnyPoolOptions}, migrate::Migrator, AnyPool, ConnectOptions};
use std::str::FromStr;

// Ensure drivers are installed exactly once for sqlx::any
static INSTALL_DRIVERS: Once = Once::new();

// Embed SQL migrations from the migrations/ directory
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Manages persistent storage of aggregated data
pub struct Database {
    pool: AnyPool,
}

impl Database {
    /// Connect to the database at the given URL, or use a default sqlite path if None.
    pub async fn connect(database_url: Option<&PathBuf>) -> Result<Self> {
        
        INSTALL_DRIVERS.call_once(|| {
            sqlx::any::install_default_drivers();
        });

        let url = match database_url.as_ref().and_then(|p| p.to_str()) {
            Some(s) => s.to_string(),
            None => default_sqlite_url()?,
        }; 

        let options = AnyConnectOptions::from_str(&url)
            .with_context(|| format!("parsing database url: {}", url))?;
        let options = options.disable_statement_logging(); // quiet by default
        
        let pool = AnyPoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await
            .with_context(|| format!("failed to connect to database: {url}"))?;

        Ok(Self { pool })
    }

    /// Run any pending database migrations.
    pub async fn run_migrations(&self) -> Result<()> {
        match MIGRATOR.run(&self.pool).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                let looks_modified = msg.contains("was previously applied but has been modified");
                let duplicate_version =
                    msg.contains("UNIQUE constraint failed: _sqlx_migrations.version");
                if looks_modified || duplicate_version {
                    let _ = sqlx::query("DELETE FROM _sqlx_migrations")
                        .execute(&self.pool)
                        .await;
                    MIGRATOR
                        .run(&self.pool)
                        .await
                        .context("running migrations after ledger reset")
                } else {
                    Err(e).context("running migrations")
                }
            }
        }
    }

    /// Perform a VACUUM operation on the database to reclaim space.
    pub async fn vacuum(&self) -> Result<()> {
        let _ = sqlx::query("VACUUM").execute(&self.pool).await;
        Ok(())
    }

    /// Get a reference to the underlying connection pool.
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }
    
}


fn default_sqlite_url() -> Result<String> {
    let proj = ProjectDirs::from("com", "fiveeus", "awasmlib")
        .context("unable to determine data directory for default sqlite path")?;
    let mut path: PathBuf = proj.data_dir().to_path_buf();
    std::fs::create_dir_all(&path)
        .with_context(|| format!("creating data dir: {}", path.display()))?;
    path.push("awasm.db");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating db parent dir: {}", parent.display()))?;
    }

    let _ = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&path);

    let mut path_str = path.to_string_lossy().to_string();
    if path_str.contains(' ') {
        path_str = path_str.replace(' ', "%20");
    }
    Ok(format!("sqlite:///{path_str}?mode=rwc"))
}