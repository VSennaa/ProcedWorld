//! Authoritative ProcedWorld server (ADR-0001): HTTP health plus a versioned JSON WebSocket
//! protocol (docs/sdd/10-protocolo.md). Rules live in `pw-engine`; this crate only validates,
//! orders, stores and broadcasts.

pub mod config;
pub mod protocol;
pub mod store;
pub mod view;
pub mod world;
pub mod ws;

pub use config::ServerConfig;
pub use store::{FileStore, InMemoryStore, WorldRecord, WorldStore};
pub use ws::{build_app, AppState};
