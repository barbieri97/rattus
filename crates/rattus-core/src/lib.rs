//! Simulation engine for Rattus, a virtual rat in an operant chamber.
//!
//! Everything here is deterministic given a seed and independent of the user interface,
//! so the learning phenomena can be tested headless (see `tests/phenomena.rs`).

pub mod assoc;
pub mod behavior;
pub mod chamber;
pub mod classical;
pub mod file;
pub mod operant;
pub mod protocol;
pub mod rat;
pub mod recorder;
pub mod rng;
pub mod schedule;
pub mod sim;
mod templates;

pub use sim::{DT, Simulation};
