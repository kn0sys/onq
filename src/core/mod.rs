// src/core/mod.rs

//! Core data structures and types

// Declare modules within core
pub mod error;
pub mod qdu;
pub mod frame;
pub mod state;

// Re-export public types for convenient access via `onq::core::TypeName`
pub use error::{OnqError, QduId};
pub use qdu::Qdu;
pub use frame::ReferenceFrame;
pub use state::{PotentialityState, StableState};

pub mod constants;
pub use constants::onq_constants::{PHI, PI}; // Re-export
