//! Server configuration read from the environment.

use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

/// Default bind address: loopback only, matching the VPS port plan (8100-8199).
pub const DEFAULT_BIND: &str = "127.0.0.1:8100";

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind: SocketAddr,
    /// Technical reconnect grace (ADR-0008): not a game clock. After it expires without a
    /// reconnect, the seat is treated as absent and its Governor plays.
    pub reconnect_grace: Duration,
    pub max_worlds: usize,
    pub max_civs: u32,
    /// Optional data directory. When absent, worlds are intentionally ephemeral.
    pub data_dir: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND.parse().expect("default bind address is valid"),
            reconnect_grace: Duration::from_secs(60),
            max_worlds: 16,
            max_civs: 16,
            data_dir: None,
        }
    }
}

impl ServerConfig {
    /// Reads `PW_SERVER_BIND`, `PW_RECONNECT_GRACE_SECS`, `PW_MAX_WORLDS`, `PW_MAX_CIVS`, `PW_DATA_DIR`.
    pub fn from_env() -> Result<Self, String> {
        let mut config = Self::default();
        if let Ok(value) = env::var("PW_SERVER_BIND") {
            config.bind = value.parse().map_err(|e| format!("PW_SERVER_BIND: {e}"))?;
        }
        if let Ok(value) = env::var("PW_RECONNECT_GRACE_SECS") {
            config.reconnect_grace = Duration::from_secs(
                value
                    .parse()
                    .map_err(|e| format!("PW_RECONNECT_GRACE_SECS: {e}"))?,
            );
        }
        if let Ok(value) = env::var("PW_MAX_WORLDS") {
            config.max_worlds = value.parse().map_err(|e| format!("PW_MAX_WORLDS: {e}"))?;
        }
        if let Ok(value) = env::var("PW_MAX_CIVS") {
            config.max_civs = value.parse().map_err(|e| format!("PW_MAX_CIVS: {e}"))?;
        }
        if let Ok(value) = env::var("PW_DATA_DIR") {
            if value.is_empty() {
                return Err("PW_DATA_DIR must not be empty".into());
            }
            config.data_dir = Some(PathBuf::from(value));
        }
        Ok(config)
    }
}
