// src/core/mod.rs

//! Core data structures and types

// Declare modules within core
pub mod error;
pub mod frame;
pub mod qdu;
/// Geometric tensor network state representation
pub mod state;

// Re-export public types for convenient access via `onq::core::TypeName`
pub use error::{OnqError, QduId};
pub use frame::ReferenceFrame;
pub use qdu::Qdu;
pub use state::{PotentialityState, StableState};

pub mod constants;
pub use constants::onq_constants::{PHI, PI}; // Re-export
