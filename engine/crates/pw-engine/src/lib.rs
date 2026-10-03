//! Deterministic primitives for the ProcedWorld simulation engine.
//!
//! This crate intentionally has no dependencies. Simulation code uses only integer
//! arithmetic and explicit, versioned pseudo-random streams.

pub mod hash;
pub mod hex;
pub mod ids;
pub mod mapgen;
pub mod rng;
pub mod bots;
pub mod world;

/// Version printed by the harness and pinned by future replay metadata.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
