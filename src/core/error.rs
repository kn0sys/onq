//! Error handling logic

use std::fmt;

/// Unique identifier for a Qualitative Distinction Unit (QDU).
/// Its uniqueness is context-dependent within a simulation, reflecting
/// its distinct position and origin within a structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QduId(pub u64);

impl fmt::Display for QduId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QDU({})", self.0)
    }
}

/// Error types representing failures based on principles.
/// These errors arise when simulated processes violate the necessary
/// consequences derived from the framework (e.g., loss of coherence, instability).
#[derive(Debug, Clone, PartialEq, Eq)] // Eq useful for testing error variants
pub enum OnqError {
    /// Failure to maintain coherent state or unified reference.
    /// Analogous to failing (Phase Coherence).
    Incoherence {
        /// Incoherence failure message
        message: String
    },

    /// Failure to stabilize or maintain stable patterns.
    /// Analogous to failing (Frame Stability) or (Pattern Convergence).
    Instability {
        /// Instability failure message
        message: String
    },

    /// Compromised distinction boundary, losing qualitative distinction
    BoundaryFailure {
        /// Compromised QDU
        qdu_id: QduId,
        /// BoundaryFailure failure message
        message: String
    },

    /// Invalid reference, relationship, or connection between elements
    ReferenceViolation {
        /// ReferenceViolation failure message
        message: String
    },

    /// An applied operation is inconsistent with the current state or framework rules
    InvalidOperation {
        /// InvalidOperation failure message
        message: String
    },

    /// General error encountered during the simulation process itself.
    SimulationError {
        /// SimulationError failure message
        message: String
    },
    // Future: Could add variants like `FrameNotFound`, `QduNotFound` if needed by simulation logic.
}

impl fmt::Display for OnqError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OnqError::Incoherence { message } => write!(f, "Incoherence Violation: {}", message),
            OnqError::Instability { message } => write!(f, "Instability Violation: {}", message),
            OnqError::BoundaryFailure { qdu_id, message } => write!(f, "Boundary Failure ({}): {}", qdu_id, message),
            OnqError::ReferenceViolation { message } => write!(f, "Reference Violation: {}", message),
            OnqError::InvalidOperation { message } => write!(f, "Invalid Operation: {}", message),
            OnqError::SimulationError { message } => write!(f, "Simulation Process Error: {}", message),
        }
    }
}

// Implement the standard Error trait to allow for easy integration with Rust error handling.
impl std::error::Error for OnqError {}
