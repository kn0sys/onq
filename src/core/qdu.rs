// src/core/qdu.rs

use super::error::QduId;
use std::fmt;

/// Represents a Qualitative Distinction Unit (QDU).
/// This is the fundamental unit of distinction derived from
/// initial derivations (Existence, Boundary, Information Necessity).
/// Each QDU carries the potential for state/quality.
///
/// Analogy: Conceptually analogous to a qubit in quantum computing, but its
/// properties and behavior are strictly defined by derivations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Qdu {
    /// Unique identifier derived from its creation context within a simulation.
    /// This simulates the uniqueness arising from the history and position
    /// within the reference structures.
    id: QduId,
}

impl Qdu {
    /// Creates a new QDU instance.
    ///
    /// Note: Ensuring the uniqueness of `id_val` within a given simulation
    /// context (e.g., within a `ReferenceFrame` or `Simulator`) is crucial
    /// and the responsibility of the calling code managing QDU allocation.
    /// This reflects the principle that distinction arises within a context.
    #[allow(dead_code)]
    pub(crate) fn new(id_val: u64) -> Self {
         //Marked pub(crate) as direct creation might be managed internally.
        // A public factory method might exist elsewhere (e.g., on Simulator or Frame).
        Self { id: QduId(id_val) }
    }

    /// Gets the unique identifier of this QDU.
    pub fn id(&self) -> QduId {
        self.id
    }
}

impl fmt::Display for Qdu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id) // Display QDU by its ID
    }
}
