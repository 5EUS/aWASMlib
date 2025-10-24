use std::collections::HashMap;
use std::{path::PathBuf, time::Duration};
use std::sync::{atomic::{AtomicU64, AtomicBool, Ordering}, Arc};
use anyhow::{anyhow, Result};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task;
use tracing::{error, info, warn};
use wasmtime::{Config, Engine};

use plugin::Plugin;

wasmtime::component::bindgen!({
    world: "library",
    path: "wit/",
});

mod plugin;
mod host;
mod config;

// Commands routed to a dedicated worker thread per plugin
enum PluginCmd {
    FetchMediaList {
        kind: MediaType,
        query: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Media>>>,
    },
    FetchUnits {
        media_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Unit>>>,
    },
    FetchAssets {
        unit_id: String,
        reply: oneshot::Sender<anyhow::Result<Vec<Asset>>>,
    },
    GetCapabilities {
        reply: oneshot::Sender<anyhow::Result<ProviderCapabilities>>,
    },
    GetAllowedHosts {
        reply: oneshot::Sender<anyhow::Result<Vec<String>>>,
    }
}

/// Worker managing a single plugin instance
#[derive(Clone)]
struct PluginWorker {
    tx: mpsc::Sender<PluginCmd>,
    call_timeout: Duration,
}

/// A loaded plugin instance
struct PluginSlot {
    name: String,
    artifacts: PluginArtifacts,
    engine: Arc<Engine>,
    epoch_ticks: Arc<AtomicU64>,
    epoch_interval: Duration,
    state: Mutex<Option<PluginWorker>>,
}
impl PluginSlot {
    /// Create a new PluginSlot struct (not yet initialized)
    fn new(
        name: String,
        artifacts: PluginArtifacts,
        engine: Arc<Engine>,
        epoch_ticks: Arc<AtomicU64>,
        epoch_interval: Duration,
    ) -> Self {
        Self {name, artifacts, engine, epoch_ticks, epoch_interval, state: Mutex::new(None)}
    }

    /// Initialize a plugin from the given artifact path
    async fn init(&self, path_buf: &PathBuf) -> Result<PluginWorker> {
        if !path_buf.exists() {
            return Err(anyhow!("missing plugin artifact: {}", path_buf.display()));
        }

        let cfg_path = self.artifacts.config.clone();
        if !cfg_path.exists() {
            return Err(anyhow!("missing plugin config: {}", cfg_path.display()));
        }

        let slot_name = self.name.clone();
        let engine = self.engine.clone();
        let epoch_ticks = self.epoch_ticks.clone();
        let interval = self.epoch_interval;
        let path_to_load = path_buf.to_path_buf();

        let plugin = task::spawn_blocking(move || -> Result<Plugin> {
            let worker_threads = 2;
            let rt_arc = std::sync::Arc::new(
                tokio::runtime::Builder::new_multi_thread()
                    // .enable_all()
                    .worker_threads(worker_threads)
                    .build()?,
            );
            let fut = Plugin::new_async(
                &engine,
                &path_to_load,
                epoch_ticks,
                interval,
                rt_arc.clone(),
            );
            rt_arc.block_on(fut)
        })
        .await
        .map_err(|e| {
            anyhow!(
                "failed to join plugin loader thread for {}: {}",
                slot_name,
                e
            )
        })??;

        let call_timeout = plugin.call_timeout;
        let (tx, mut rx) = mpsc::channel::<PluginCmd>(64);
        std::thread::spawn(move || {
            let mut plugin = plugin;
            while let Some(cmd) = rx.blocking_recv() {
                match cmd {
                    PluginCmd::FetchMediaList { kind, query, reply } => {
                        let _ = reply.send(plugin.fetch_media_list(kind, &query));
                    }
                    PluginCmd::FetchUnits { media_id, reply } => {
                        let _ = reply.send(plugin.fetch_units(&media_id));
                    }
                    PluginCmd::FetchAssets { unit_id, reply } => {
                        let _ = reply.send(plugin.fetch_assets(&unit_id));
                    }
                    PluginCmd::GetCapabilities { reply } => {
                        let _ = reply.send(plugin.get_capabilities());
                    }
                    PluginCmd::GetAllowedHosts { reply } => {
                        let hosts = plugin.allowed_hosts.clone().unwrap_or_default();
                        let _ = reply.send(Ok(hosts));
                    }
                }
            }
        });
        info!("Loaded plugin: {}", path_buf.display());
        Ok(PluginWorker { tx, call_timeout })
    }

    /// Get or create the PluginWorker for this slot
    async fn worker(&self) -> Result<PluginWorker> {
        // If we already have a worker, return it
        let mut guard = self.state.lock().await;
        if let Some(worker) = guard.as_ref() {
            return Ok(worker.clone());
        }

        // Otherwise, instantiate a new worker from the primary artifact, falling back if needed
        let primary_path = &self.artifacts.primary.clone();
        match self.init(primary_path).await {
            Ok(worker) => {
                *guard = Some(worker.clone());
                return Ok(worker);
            }
            Err(mut err) => {
                warn!(plugin=%self.name, path=%primary_path.display(), error=?err, "failed to load plugin artifact");
                if let Some(fallback_path) = &self.artifacts.fallback {
                    warn!(plugin=%self.name, path=%fallback_path.display(), error=?err, "attempting fallback artifact");
                    match self.init(fallback_path).await {
                        Ok(worker) => {
                            *guard = Some(worker.clone());
                            return Ok(worker);
                        }
                        Err(fallback_err) => {
                            error!(plugin=%self.name, path=%fallback_path.display(), error=?fallback_err, "fallback plugin load failed");
                            err = fallback_err;
                        }
                    }
                }
                Err(err)
            }
        }
    }

    /// Get the name of this plugin
    fn name(&self) -> &str {
        &self.name
    }
}


/// Owned by a PluginSlot, holds paths to compiled artifacts
struct PluginArtifacts {
    primary: PathBuf,
    fallback: Option<PathBuf>,
    config: PathBuf,
}

/// Paths to compiled artifacts for a plugin
#[derive(Default)]
struct ArtifactSet {
    wasm: Option<PathBuf>,
    cwasm: Option<PathBuf>,
    toml: Option<PathBuf>,
}
impl ArtifactSet {
    /// Choose the appropriate artifacts based on preference and availability
    /// If prefer_precompiled is true, .cwasm will be preferred over .wasm
    fn into_artifacts(self, prefer_precompiled: bool) -> Option<PluginArtifacts> {
        match (self.cwasm, self.wasm, prefer_precompiled) {
            (Some(cwasm), Some(wasm), true) => {
                let cwasm_clone = cwasm.clone();
                Some(PluginArtifacts {
                    primary: cwasm,
                    fallback: Some(wasm),
                    config: self.toml.unwrap_or_else(|| cwasm_clone.with_extension("toml")),
                })
            },
            (Some(cwasm), Some(wasm), false) => {
                let wasm_clone = wasm.clone();
                Some(PluginArtifacts {
                    primary: wasm,
                    fallback: Some(cwasm),
                    config: self.toml.unwrap_or_else(|| wasm_clone.with_extension("toml")),    
                })
            },
            (Some(cwasm), None, _) => Some(PluginArtifacts {
                primary: cwasm.clone(),
                fallback: None,
                config: self.toml.unwrap_or_else(|| cwasm.with_extension("toml")),
            }),
            (None, Some(wasm), _) => Some(PluginArtifacts {
                primary: wasm.clone(),
                fallback: None,
                config: self.toml.unwrap_or_else(|| wasm.with_extension("toml")),
            }),
            _ => None,
        }
    }
}

/// Manages loading, unloading, and interfacing with plugins
pub struct PluginManager {
    engine: Arc<Engine>,
    slots: Vec<Arc<PluginSlot>>,
    epoch_ticks: Arc<AtomicU64>,
    epoch_interval: Duration,
    _epoch_stop: Arc<AtomicBool>,
    _epoch_thread: Option<std::thread::JoinHandle<()>>,
}

impl PluginManager {
    pub async fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        config.epoch_interruption(true);

        let engine = Arc::new(Engine::new(&config)?);

        // Start epoch ticker (10ms)
        let epoch_interval = Duration::from_millis(10);
        let epoch_ticks = Arc::new(AtomicU64::new(0));
        let epoch_stop = Arc::new(AtomicBool::new(false));
        let eng = engine.clone();
        let ticks = epoch_ticks.clone();
        let stop = epoch_stop.clone();
        let handle = std::thread::spawn(move || {
            ticks.store(1, Ordering::Relaxed);
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                std::thread::sleep(epoch_interval);
                eng.increment_epoch();
                ticks.fetch_add(1, Ordering::Relaxed);
            }
        });

        Ok(Self {
            engine,
            slots: Vec::new(),
            epoch_ticks,
            epoch_interval,
            _epoch_stop: epoch_stop,
            _epoch_thread: Some(handle),
        })
    }

    /// Load plugins from the specified directory, replacing any previously loaded plugins.
    /// If the directory does not exist, no plugins will be loaded
    pub async fn load_plugins_from_directory(&mut self, dir: &PathBuf) -> Result<()> {
        self.slots.clear();
        if !dir.exists() {
            warn!("Plugin directory does not exist: {}", dir.display());
            return Ok(());
        }
        let prefer_precompiled = !cfg!(target_os = "android");
        let mut artifacts_by_name: HashMap<String, ArtifactSet> = HashMap::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
                continue;
            };
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let entry = artifacts_by_name.entry(stem.to_string()).or_default();
            match ext {
                "cwasm" => entry.cwasm = Some(path),
                "wasm" => entry.wasm = Some(path),
                _ => {}
            }
        }

        for (name, artifact_set) in artifacts_by_name {
            let Some(artifacts) = artifact_set.into_artifacts(prefer_precompiled) else {
                warn!(plugin=%name, "skipping plugin - no valid artifacts found");
                continue;
            };
            let cfg_path = artifacts.config.clone();
            if !cfg_path.exists() {
                warn!(plugin=%name, config=%cfg_path.display(), "rejecting plugin: missing .toml config");
                continue;
            }
            let slot = PluginSlot::new(
                name.clone(),
                artifacts,
                self.engine.clone(),
                self.epoch_ticks.clone(),
                self.epoch_interval,
            );
            info!(plugin=%name, "registered plugin for lazy loading");
            self.slots.push(Arc::new(slot));
        }

        self.slots.sort_by(|a, b| a.name().cmp(b.name()));
        Ok(())
    }

    /// Get all plugin names
    pub fn list_plugins(&self) -> Vec<String> {
        self.slots
            .iter()
            .map(|slot| slot.name().to_string())
            .collect()
    }

    pub async fn get_all_capabilities(&self, _refresh: bool) -> Result<HashMap<String, ProviderCapabilities>> {
        let mut results = HashMap::new();
        for slot in &self.slots {
            let worker = slot.worker().await?;
            let (reply_tx, reply_rx) = oneshot::channel();
            let cmd = PluginCmd::GetCapabilities { reply: reply_tx };
            if let Err(e) = worker.tx.send(cmd).await {
                warn!(plugin=%slot.name(), "failed to send GetCapabilities command: {}", e);
                continue;
            }
            match tokio::time::timeout(worker.call_timeout, reply_rx).await {
                Ok(Ok(Ok(caps))) => {
                    results.insert(slot.name().to_string(), caps);
                }
                Ok(Ok(Err(e))) => {
                    warn!(plugin=%slot.name(), "plugin GetCapabilities error: {}", e);
                }
                Ok(Err(_)) => {
                    warn!(plugin=%slot.name(), "GetCapabilities reply channel closed");
                }
                Err(_) => {
                    warn!(plugin=%slot.name(), "GetCapabilities call timed out");
                }
            }
        }
        Ok(results)
    }
}