// src/simulation/engine.rs
use crate::core::{OnqError, PotentialityState, QduId, StableState};
use crate::operations::Operation;
// NOTE: Does not directly use Circuit, operates on ops passed from Simulator
use num_complex::Complex;
use num_traits::{One, Zero};
use std::collections::{HashMap, HashSet};
// Placeholder stabilization requires rand crate
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
// Import SimulationResult for the stabilize function signature
use crate::LockType;
use crate::simulation::SimulationResult;
use crate::validation;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// The core simulation engine that manages and evolves the potentiality state
/// according to operations derived from framework principles.
/// (Internal visibility)
#[derive(Debug)]
pub(crate) struct SimulationEngine {
    /// Maps QDU IDs to their index (0..N-1) in the ordered list used for the global state vector.
    qdu_indices: HashMap<QduId, usize>,
    /// The global state vector representing the combined PotentialityState of all simulated QDUs.
    /// The dimension is 2^N, where N is num_qdus. This representation is derived from:
    /// - (State Distinction) & (Qualitative Necessity), interpreted here
    ///   as supporting at least a binary basis {Quality0, Quality1} per QDU.
    /// - core functions (Ω, I) using phase `e^(iθ)`, justifying Complex<f64> amplitudes.
    /// - (Integration) & (Unified Coherence), requiring a combined state representation.
    global_state: PotentialityState,
    /// Number of QDUs being simulated (N).
    num_qdus: usize,
}

impl SimulationEngine {
    /// Initializes the engine for a given set of QDUs.
    /// The initial state is set to |0...0>, representing a default baseline state
    /// before any operations. This baseline needs justification.
    pub(crate) fn init(qdu_ids: &HashSet<QduId>) -> Result<Self, OnqError> {
        if qdu_ids.is_empty() {
            // Cannot simulate zero QDUs.
            return Err(OnqError::InvalidOperation {
                message: "Cannot initialize simulation engine with zero QDUs".to_string(),
            });
        }

        let num_qdus = qdu_ids.len();
        // Dimension of the global state vector (2^N).
        // Derivation: Based on interpreting state distinction and qualitative necessity supporting a minimal
        // binary qualitative distinction {Quality0, Quality1} per QDU.
        let dim = 1usize
            .checked_shl(num_qdus as u32)
            .ok_or_else(|| OnqError::SimulationError {
                message:
                    "Number of QDUs too large, resulting state vector dimension overflows usize."
                        .to_string(),
            })?;

        // Create mapping from QduId to index (0..N-1)
        let mut qdu_indices = HashMap::with_capacity(num_qdus);
        // Sort IDs to ensure deterministic index assignment regardless of HashSet iteration order.
        let mut sorted_ids: Vec<QduId> = qdu_ids.iter().cloned().collect();
        sorted_ids.sort(); // Relies on Ord trait derived for QduId
        for (index, qdu_id) in sorted_ids.into_iter().enumerate() {
            qdu_indices.insert(qdu_id, index);
        }

        // Initialize state to |0...0> : vector with 1.0 at index 0, rest 0.0
        // Derivation: Interpreted as the baseline state where all QDUs are in
        // the initial 'Quality0' state, representing minimal interaction or excitation
        // before framework operations are applied.
        let mut initial_vec = vec![Complex::zero(); dim];
        if dim > 0 {
            // Ensure vector is not empty if somehow num_qdus was 0 but passed initial check
            initial_vec[0] = Complex::new(1.0, 0.0);
        }
        let global_state = PotentialityState::new();

        Ok(Self {
            qdu_indices,
            global_state,
            num_qdus,
        })
    }

    pub fn get_state(&self) -> &PotentialityState {
        // engine is guaranteed to be Some if called within a test after init
        &self.global_state
    }

    // Add a crate-visible method to set the state directly for testing
    #[cfg(test)] // Only compile this function when running tests
    pub(crate) fn set_state(&mut self, state: PotentialityState) -> Result<(), OnqError> {
        // Optional: Add validation like normalization check here if desired
        self.global_state = state;
        Ok(())
    }

    /// These matrices represent fundamental interaction patterns (`P_op`) derived
    /// from framework principles, interpreted as transformations on the
    /// {Quality0, Quality1} basis state potentiality vector.
    fn get_interaction_matrix(&self, pattern_id: &str) -> Result<[[Complex<f64>; 2]; 2], OnqError> {
        use std::f64::consts::{FRAC_1_SQRT_2, PI};
        // Define PHI locally or import from core::constants
        const PHI: f64 = 1.618_033_988_749_895;
        // Constant for 1/sqrt(2) used in Superposition
        const ONE_OVER_SQRT_2: f64 = std::f64::consts::FRAC_1_SQRT_2;
        // Helper for Complex i
        let i = Complex::i();
        // Value for exp(i*PI/4) = (1+i)/sqrt(2)
        let exp_i_pi_4 = Complex::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        // Value for exp(-i*PI/4) = (1-i)/sqrt(2)
        let exp_neg_i_pi_4 = Complex::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2);

        match pattern_id {
            // === Previously Derived ===
            "Identity" => Ok([
                // Persistence
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), Complex::new(1.0, 0.0)],
            ]),
            "QualityFlip" => Ok([
                // Binary Swap
                [Complex::zero(), Complex::new(1.0, 0.0)],
                [Complex::new(1.0, 0.0), Complex::zero()],
            ]),
            "PhaseIntroduce" => Ok([
                // Phase (PI)
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), Complex::new(-1.0, 0.0)], // Phase PI
            ]),
            "Superposition" => Ok([
                // Tentative state mixing
                [
                    Complex::new(ONE_OVER_SQRT_2, 0.0),
                    Complex::new(ONE_OVER_SQRT_2, 0.0),
                ],
                [
                    Complex::new(ONE_OVER_SQRT_2, 0.0),
                    Complex::new(-ONE_OVER_SQRT_2, 0.0),
                ],
            ]),
            "PhiRotate" => {
                //  'φ' Resonance Rotation (Ry(theta=PI/PHI))
                let theta = PI / PHI;
                let angle_over_2 = theta / 2.0;
                let cos_a = angle_over_2.cos();
                let sin_a = angle_over_2.sin();
                // Ry(theta) = [[cos(a), -sin(a)], [sin(a), cos(a)]] where a=theta/2
                Ok([
                    [Complex::new(cos_a, 0.0), Complex::new(-sin_a, 0.0)],
                    [Complex::new(sin_a, 0.0), Complex::new(cos_a, 0.0)],
                ])
            }
            "PhiXRotate" => {
                // 'φ' Resonance Rotation (Rx(theta=PI/PHI))
                let theta = PI / PHI;
                let angle_over_2 = theta / 2.0;
                let cos_a = angle_over_2.cos();
                let sin_a = angle_over_2.sin();
                // Rx(theta) = [[cos(a), -i*sin(a)], [-i*sin(a), cos(a)]] where a=theta/2
                Ok([
                    [Complex::new(cos_a, 0.0), -i * sin_a],
                    [-i * sin_a, Complex::new(cos_a, 0.0)],
                ])
            }
            "SqrtFlip" => Ok([
                // Tentative: Partial quality flip interaction (Sqrt(X))
                [Complex::new(0.5, 0.5), Complex::new(0.5, -0.5)], // 0.5 * (1+i) , 0.5 * (1-i)
                [Complex::new(0.5, -0.5), Complex::new(0.5, 0.5)], // 0.5 * (1-i) , 0.5 * (1+i)
            ]),
            "SqrtFlip_Inv" => Ok([
                // Derived: Inverse partial flip (Sqrt(X) dagger)
                [Complex::new(0.5, -0.5), Complex::new(0.5, 0.5)], // 0.5 * (1-i) , 0.5 * (1+i)
                [Complex::new(0.5, 0.5), Complex::new(0.5, -0.5)], // 0.5 * (1+i) , 0.5 * (1-i)
            ]),

            "HalfPhase" => Ok([
                // Phase (PI/2) Quadrature Step
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), i], // e^(i*PI/2) = i
            ]),
            "QualitativeY" => Ok([
                // Combined Flip/Phase Interaction Pattern
                [Complex::zero(), -i],
                [i, Complex::zero()],
            ]),
            "QuarterPhase" => Ok([
                // (PI/4) - Finer Step
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), exp_i_pi_4],
            ]),
            "HalfPhase_Inv" => Ok([
                // Inverse Phase (-PI/2)
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), -i], // e^(-i*PI/2) = -i
            ]),
            "QuarterPhase_Inv" => Ok([
                // Inverse Phase (-PI/4)
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), exp_neg_i_pi_4],
            ]),
            _ => Err(OnqError::InvalidOperation {
                message: format!(
                    "Interaction Pattern '{}' is not defined or derived yet",
                    pattern_id
                ),
            }),
        }
    }

    /// Optional: Placeholder for state validation (e.g., check normalization)
    #[allow(dead_code)] // Temporarily allow unused function during development
    fn validate_state(&self) -> Result<(), OnqError> {
        let norm_sq: f64 = self.global_state.global_norm_sq();
        // Use a tolerance for floating point comparisons
        if (norm_sq - 1.0).abs() > 1e-9 {
            return Err(OnqError::Incoherence {
                message: format!(
                    "State vector norm deviated significantly from 1: {}",
                    norm_sq
                ),
            });
        }
        Ok(())
    }

    /// Calculates the interpretive C1 Score (Phase Coherence) for a basis state k.
    /// Measures phase agreement with nearest neighbour basis states that have non-negligible amplitude.
    /// Returns a score between 0.0 and 1.0.
    #[allow(dead_code)]
    fn calculate_c1_score(&self, k: usize, state_vector: &[Complex<f64>]) -> f64 {
        if self.num_qdus == 0 {
            return 1.0;
        } // Coherence is perfect for 0 QDUs

        let amp_k = state_vector[k];
        let amp_k_norm_sq = amp_k.norm_sqr();

        // If the amplitude of state k itself is negligible, coherence is undefined/low.
        if amp_k_norm_sq < 1e-12 {
            return 0.0; // Or some other low value / handle appropriately
        }
        let phase_k = amp_k.arg();

        let mut total_cos_diff = 0.0;
        let mut num_significant_neighbours = 0;

        for bit_pos in 0..self.num_qdus {
            let neighbour_k = k ^ (1 << bit_pos); // Index of neighbour differing at bit_pos
            if neighbour_k < state_vector.len() {
                // Ensure neighbour is within bounds
                let amp_neighbour = state_vector[neighbour_k];
                // Only consider neighbours with significant amplitude for phase comparison
                if amp_neighbour.norm_sqr() > 1e-12 {
                    let phase_neighbour = amp_neighbour.arg();
                    total_cos_diff += (phase_k - phase_neighbour).cos();
                    num_significant_neighbours += 1;
                }
                // If neighbour amplitude is zero, it doesn't contribute to phase (in)coherence here.
            }
        }

        if num_significant_neighbours == 0 {
            // No significant neighbours to compare phase with - maximally coherent? Or neutral? Let's say 1.0.
            return 1.0;
        }

        let avg_cos_diff = total_cos_diff / (num_significant_neighbours as f64);
        // Normalize score to be between 0.0 and 1.0
        let score = (1.0 + avg_cos_diff) / 2.0;
        score.clamp(0.0, 1.0) // Clamp for robustness
    }
}

/// Provides the 2x2 matrix for the PhaseShift operation.
/// Derived form: Represents applying the phase factor `e^(i*theta)`
/// conditional on the QDU being in the `Quality1` state (index 1 of the 2x2 matrix).
fn phase_shift_matrix(theta: f64) -> [[Complex<f64>; 2]; 2] {
    [
        [Complex::new(1.0, 0.0), Complex::zero()],
        [Complex::zero(), Complex::new(theta.cos(), theta.sin())], // e^(i*theta)
    ]
}
