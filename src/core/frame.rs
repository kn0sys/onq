//! QDU interaction logic

use std::fmt;
// Potentially use Arc later if frames need to be shared reference-counted objects
// use std::sync::Arc;

/// Represents a Reference Frame.
/// A Reference Frame provides the context within which distinctions (QDUs) exist,
/// relate, and interact. It's essential for defining relative properties
/// and enabling coherent integration.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceFrame {
    /// Unique identifier for this frame within a simulation context.
    id: u64,
    // Future potential:
    // - Rules governing interactions specific to this frame type.
    // - Links to parent/child frames simulating structural depth.
}

impl ReferenceFrame {
    /// Creates a new Reference Frame.
    /// ID uniqueness should be managed by the simulation environment.
    #[allow(dead_code)]
    pub(crate) fn new(id: u64) -> Self {
    // pub(crate) as frame creation might be controlled centrally.
        Self { id }
    }

    /// Gets the unique identifier of this Reference Frame.
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl fmt::Display for ReferenceFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Frame({})", self.id)
    }
}
