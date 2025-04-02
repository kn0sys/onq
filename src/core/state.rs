// src/core/state.rs

// Make sure `num-complex` is in Cargo.toml: `num-complex = "0.4"`
use num_complex::Complex;
use std::fmt;

/// Represents the potential state of a QDU or system *before* stabilization.
/// Derived from principles:
/// - Potentiality inherent in reference structures within Frames.
/// - Requirement for Integration implies intermediate complex states.
/// - Qualitative Necessity means distinctions have natures/potentials.
/// - Information Fields contain structural potentials.
///
/// The use of `Complex<f64>` is justified by the need to directly simulate
/// the phase component `e^(iθ)` central to functions (Ω, I, T scaling),
/// per Appendix A.2/A.5.
///
/// Analogy: Conceptually analogous to a quantum state vector, but its interpretation
/// and manipulation rules must be derived strictly from the framework.
#[derive(Debug, Clone, PartialEq)] // Avoid Eq for floating-point complex numbers
pub struct PotentialityState {
    /// The vector representing the weighted potentialities.
    /// The basis and interpretation of this vector must stem from nature.
    /// For now, it's an abstract vector whose manipulation rules will be defined
    /// by the simulation engine based on derived Operations.
    /// Using f64 for the components of the complex numbers.
    state_vector: Vec<Complex<f64>>,
    // Potential future metadata:
    // - Associated QDU IDs this state describes.
    // - Reference to the `ReferenceFrame` providing context.
    // - Measures of coherence derived from itegration and unified coherence.
}

impl PotentialityState {
    /// Creates a new potentiality state from a given vector.
    /// The validity and interpretation of the initial vector depend on the
    /// specific context where it's created.
    pub(crate) fn new(initial_vector: Vec<Complex<f64>>) -> Self {
        // Normalization: onq doesn't explicitly mandate probability/normalization like QM.
        // Coherence might imply constraints on valid states, potentially
        // resembling normalization as an emergent property of stable/integrated systems.
        // For now, accept the vector as is. Validation happens during simulation.
        Self { state_vector: initial_vector }
    }

    /// Provides read-only access to the internal state vector.
    pub fn vector(&self) -> &[Complex<f64>] {
        &self.state_vector
    }

    /// Provides mutable access for the simulation engine to modify the state.
    #[allow(dead_code)]
    pub(crate) fn vector_mut(&mut self) -> &mut [Complex<f64>] {
        &mut self.state_vector
    }

    /// Gets the dimension or number of basis potentialities represented.
    pub fn dim(&self) -> usize {
        self.state_vector.len()
    }
}

impl fmt::Display for PotentialityState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Potentiality[")?;
        for (i, c) in self.state_vector.iter().enumerate() {
            write!(f, "{}{:.4}", if i > 0 { ", " } else { "" }, c)?;
            // Using Display for Complex which shows "re+imj" or similar
        }
        write!(f, "]")
    }
}

/// Represents a resolved, definite state after a Stabilization Protocol.
/// This reflects the emergence of stable patterns, the result of
/// integration, leading to distinguished states and the
/// formation of stable reality structures.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StableState {
    /// Represents a specific, distinct qualitative outcome.
    /// The interpretation of the `u64` value depends on the basis defined
    /// by the context and the stabilization process. It might represent
    /// an index into a set of possible qualities or a direct value.
    ResolvedQuality(u64),
    // Future: Could have variants like `Undetermined` if stabilization fails coherently,
    // or `Dissolved` if it leads to framework errors (though errors might be better).
}

impl StableState {
    /// Helper to extract the numerical value from a ResolvedQuality state.
    pub fn get_resolved_value(&self) -> Option<u64> {
        match self {
            StableState::ResolvedQuality(val) => Some(*val),
        }
    }
}

impl fmt::Display for StableState {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StableState::ResolvedQuality(val) => write!(f, "Stable({})", val),
        }
    }
}

// Optional: Implement approximate equality for PotentialityState if needed for testing
// fn states_approx_equal(s1: &PotentialityState, s2: &PotentialityState, tolerance: f64) -> bool {
//     if s1.dim() != s2.dim() { return false; }
//     s1.vector().iter().zip(s2.vector().iter()).all(|(c1, c2)| {
//         (c1.re - c2.re).abs() < tolerance && (c1.im - c2.im).abs() < tolerance
//     })
// }
