// src/simulation/engine.rs
use crate::core::{QduId, PotentialityState, StableState, OnqError};
use crate::operations::Operation;
// NOTE: Does not directly use Circuit, operates on ops passed from Simulator
use std::collections::{HashMap, HashSet};
use num_complex::Complex;
use num_traits::Zero; // For Complex::zero()
// Placeholder stabilization requires rand crate
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
// Import SimulationResult for the stabilize function signature
use crate::simulation::SimulationResult;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// The core simulation engine that manages and evolves the potentiality state
/// according to operations derived from framework principles.
/// (Internal visibility)
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
            return Err(OnqError::InvalidOperation { message: "Cannot initialize simulation engine with zero QDUs".to_string() });
        }

        let num_qdus = qdu_ids.len();
        // Dimension of the global state vector (2^N).
        // Derivation: Based on interpreting state distinction and qualitative necessity supporting a minimal
        // binary qualitative distinction {Quality0, Quality1} per QDU.
        let dim = 1usize.checked_shl(num_qdus as u32).ok_or_else(|| OnqError::SimulationError { message: "Number of QDUs too large, resulting state vector dimension overflows usize.".to_string() })?;


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
        if dim > 0 { // Ensure vector is not empty if somehow num_qdus was 0 but passed initial check
           initial_vec[0] = Complex::new(1.0, 0.0);
        }
        let global_state = PotentialityState::new(initial_vec);

        Ok(Self {
            qdu_indices,
            global_state,
            num_qdus,
        })
    }

    // Add a crate-visible method to set the state directly for testing
    #[cfg(test)] // Only compile this function when running tests
    pub(crate) fn set_state(&mut self, state: PotentialityState) -> Result<(), OnqError> {
        if state.dim() != self.global_state.dim() {
            Err(OnqError::SimulationError {
                message: format!("Cannot set state: provided dimension {} does not match engine dimension {}", state.dim(), self.global_state.dim())
            })
        } else {
            // Optional: Add validation like normalization check here if desired
            self.global_state = state;
            Ok(())
        }
    }

    /// Applies a single non-stabilization operation to the global state.
    /// This is where the core dynamics need to be implemented.
    /// Note: Some operations still use placeholder logic.
    pub(crate) fn apply_operation(&mut self, op: &Operation) -> Result<(), OnqError> {

        match op {
            Operation::PhaseShift { target, theta } => {
                let target_idx = self.get_qdu_index(target)?;
                // Apply phase shift matrix. The matrix form is derived from applying
                // e^(iθ) phase factor based on the QDU's binary
                // qualitative state {Quality0, Quality1}.
                self.apply_single_qdu_gate(*target_idx, &phase_shift_matrix(*theta))?;
            }
            Operation::InteractionPattern { target, pattern_id } => {
                let target_idx = self.get_qdu_index(target)?;
                // Get matrix for the derived or placeholder pattern_id.
                let matrix = self.get_interaction_matrix(pattern_id)?;
                self.apply_single_qdu_gate(*target_idx, &matrix)?;
            }
            Operation::ControlledInteraction { control, target, pattern_id } => {
                 let control_idx = *self.get_qdu_index(control)?;
                 let target_idx = *self.get_qdu_index(target)?;

                 if control_idx == target_idx {
                     return Err(OnqError::InvalidOperation { message: "Control and target QDUs cannot be the same for controlled operation".to_string() });
                 }

                 // Get the 2x2 matrix for the operation applied to the target
                 let u_matrix = self.get_interaction_matrix(pattern_id)?;

                 // Construct the 4x4 controlled-U matrix based on derived state influence
                 // Applies U to target only when control bit is 1.
                 // Basis order: |control, target> -> |00>, |01>, |10>, |11>
                 let controlled_u_matrix: [[Complex<f64>; 4]; 4] = [
                    // Control |0> subspace: Identity on target
                    [Complex::new(1.0, 0.0), Complex::zero(),         Complex::zero(),         Complex::zero()        ],
                    [Complex::zero(),         Complex::new(1.0, 0.0), Complex::zero(),         Complex::zero()        ],
                    // Control |1> subspace: Apply U matrix to target
                    [Complex::zero(),         Complex::zero(),         u_matrix[0][0],          u_matrix[0][1]         ],
                    [Complex::zero(),         Complex::zero(),         u_matrix[1][0],          u_matrix[1][1]         ],
                 ];

                 // Apply the joint 4x4 transformation using the existing helper
                 // Ensure indices are passed correctly based on how apply_two_qdu_gate expects them
                 // (e.g., idx1 for control, idx2 for target, or vice-versa - needs consistency check)
                 // Let's assume apply_two_qdu_gate maps idx1 -> first bit, idx2 -> second bit in |b1,b2>
                 self.apply_two_qdu_gate(control_idx, target_idx, &controlled_u_matrix)?;
            }
            Operation::RelationalLock { qdu1, qdu2, lock_params, establish } => {
                // Derived: Implements reference, interation, etc. Phase Lock (○↔○)
                // interpretation using a Controlled-Phase interaction pattern.
                let idx1 = *self.get_qdu_index(qdu1)?;
                let idx2 = *self.get_qdu_index(qdu2)?;

                let theta = if *establish { *lock_params } else { -*lock_params };
                let phase_factor = Complex::new(theta.cos(), theta.sin());
                let cphase_matrix: [[Complex<f64>; 4]; 4] = [ // diag(1,1,1,exp(i*theta))
                    [Complex::new(1.0, 0.0), Complex::zero(), Complex::zero(), Complex::zero()],
                    [Complex::zero(), Complex::new(1.0, 0.0), Complex::zero(), Complex::zero()],
                    [Complex::zero(), Complex::zero(), Complex::new(1.0, 0.0), Complex::zero()],
                    [Complex::zero(), Complex::zero(), Complex::zero(), phase_factor],
                ];
                self.apply_two_qdu_gate(idx1, idx2, &cphase_matrix)?;
            }
            Operation::Stabilize { .. } => {
                 return Err(OnqError::InvalidOperation { message: "Stabilize operation should not be passed directly to apply_operation".to_string() });
            }
        };
        Ok(())
    }

/// Performs the stabilization process based on interpreted framework principles.
///
/// This simulates the resolution of `PotentialityState` into `StableState` by:
/// 1. Calculating a Stability Score S(k) for each potential outcome basis state |k>,
///    based on interpretations of C_A (Phase Coherence) and C_B (Pattern Resonance),
///    combined with the state's amplitude |c_k|^2.
/// 2. Filtering outcomes k to only include those meeting amplitude (> tol) and C_A Phase Coherence (> 0.618) criteria.
/// 3. Deterministically selecting an outcome |k> from the filtered possibilities using a seeded PRNG,
/// 4. Collapsing the global state vector to the chosen outcome basis state |k>.
/// 5. Recording the resolved states for the specifically targeted QDUs based on |k>.
///
/// **CRITICAL:** The C_A and CB scoring functions are experimental
/// C_B is currently interpreted as amplifying the contribution
/// of amplitude itself for coherent states (using |c_k|^4 in the final score).
pub(crate) fn stabilize(&mut self, targets: &[QduId], result: &mut SimulationResult) -> Result<(), OnqError> {
        if targets.is_empty() {
            return Ok(()); // Nothing to stabilize
        }

        let dim = self.global_state.dim();
        let state_vector = self.global_state.vector(); // Get immutable borrow first

        // 1. Calculate stability scores S(k) for all possible outcomes k
        let mut valid_outcomes: Vec<(usize, f64)> = Vec::with_capacity(dim); // Stores (index, score S(k))
        let mut total_score = 0.0;

        for k in 0..dim {
            let amplitude_sq = state_vector[k].norm_sqr();

            // Only consider states with non-negligible amplitude potentiality
            if amplitude_sq > 1e-12 {
                // Calculate scores based on interpreted checks
                let score_c1 = self.calculate_c1_score(k, state_vector);

                // **C_A Filter:** Only proceed if phase coherence meets threshold
                if score_c1 > 0.618 {

                    // Final Score S(k) - combines potentiality and filtered stability/coherence factors
                    let final_score = score_c1 * amplitude_sq * amplitude_sq;

                // Check if the final score is numerically valid and positive
                if final_score.is_finite() && final_score > 1e-12 { // Use tolerance for score as well
                    valid_outcomes.push((k, final_score));
                    total_score += final_score;
                }
            }
        }
    }

        // Check if any outcome is possible according to our scoring
        if valid_outcomes.is_empty() || total_score < 1e-12 {
            // This might happen if the state is highly incoherent or only has negligible amplitudes
            return Err(OnqError::Instability { message: "Stabilization failed: No possible outcome met amplitude and C1 Phase Coherence (>0.618) criteria.".to_string() });
        }

        // 2. Deterministic Seeding for PRNG (same as before)
        let seed = {
            let mut hasher = DefaultHasher::new();
            // Use an immutable borrow obtained earlier
            for complex_val in state_vector {
                complex_val.re.to_ne_bytes().hash(&mut hasher);
                complex_val.im.to_ne_bytes().hash(&mut hasher);
            }
            hasher.finish()
        };
        let mut rng = StdRng::seed_from_u64(seed);

        // 3. Outcome Selection from filtered valid outcomes based on scores S(k)
        let p_sample: f64 = rng.random::<f64>() * total_score; // Sample in [0, total_score)
        let mut cumulative_score = 0.0;
        let mut chosen_outcome_index: usize = valid_outcomes.last().map(|(idx, _)| *idx)
             .unwrap_or(0); // Default to 0 if possible_outcomes was empty, though checked above

        for (index, score) in &valid_outcomes {
            cumulative_score += *score;
            if p_sample < cumulative_score {
                chosen_outcome_index = *index;
                break;
            }
        }
         // Fallback in case of floating point issues where p_sample might exactly equal total_score
         // This ensures we always pick an index from the valid list.
         if !valid_outcomes.iter().any(|(idx, _)| *idx == chosen_outcome_index) {
             chosen_outcome_index = valid_outcomes.last().unwrap().0; // Use the index of the last valid outcome
          }

        // 4. State Collapse to the chosen outcome |k> (same as before)
        // Create the new state vector |k> (1.0 at index k, 0 elsewhere)
        let mut new_state_vec = vec![Complex::zero(); dim];
        new_state_vec[chosen_outcome_index] = Complex::new(1.0, 0.0);
        // Create new PotentialityState - cannot use vector_mut() if state_vector borrow is active
        self.global_state = PotentialityState::new(new_state_vec);

        // 5. Record Results for targeted QDUs (same as before)
        for target_qdu_id in targets {
            if let Some(target_idx) = self.qdu_indices.get(target_qdu_id) {
                // Extract the bit value for this QDU from the chosen global outcome index k
                let bit_pos = self.num_qdus - 1 - *target_idx; // Bit position
                let outcome_bit = (chosen_outcome_index >> bit_pos) & 1; // Get the bit value
                result.record_stable_state(*target_qdu_id, StableState::ResolvedQuality(outcome_bit as u64));
            } else {
                 return Err(OnqError::ReferenceViolation{ message: format!("QDU {} targeted for stabilization not found in simulation context.", target_qdu_id)});
            }
        }

        Ok(())
}

    /// Helper to get QDU index, returning a specific error if not found.
    fn get_qdu_index(&self, qdu_id: &QduId) -> Result<&usize, OnqError> {
        self.qdu_indices.get(qdu_id).ok_or_else(|| OnqError::ReferenceViolation { message: format!("QDU {} not found in simulation context", qdu_id) })
    }

    // --- State Manipulation Helper Methods (Using standard simulation math as placeholder basis) ---
    // **CRITICAL:** While the matrix application logic itself is standard, the matrices *used*
    // must be framework-derived. The structure of these helpers assumes a 2^N state vector.

    /// Applies a 2x2 matrix operation targeting a single QDU within the global state vector.
    /// Assumes standard tensor product structure for the global state vector.
    fn apply_single_qdu_gate(&mut self, target_idx: usize, matrix: &[[Complex<f64>; 2]; 2]) -> Result<(), OnqError> {
        let k = self.num_qdus - 1 - target_idx; // Bit position (from right, 0-based)
        let k_mask = 1 << k; // Mask for the target bit
        let lower_mask = k_mask - 1; // Mask for bits to the right
        let upper_mask = !((k_mask << 1) - 1); // Mask for bits to the left

        let dim = self.global_state.dim();
        let mut new_vec = vec![Complex::zero(); dim]; // Store results temporarily

        // Iterate over pairs of basis states differing only at the target QDU's position
        for i in 0..dim / 2 {
            // Calculate indices for |...0...> and |...1...> states for the target QDU
            // Combines upper bits, lower bits, and inserts 0 or 1 at the target position k
            let i0_raw = ((i & upper_mask) << 1) | (i & lower_mask);
            let i1_raw = i0_raw | k_mask;

            // Ensure indices are within bounds (robustness check)
             if i0_raw >= dim || i1_raw >= dim {
                 return Err(OnqError::SimulationError { message: format!("Calculated index out of bounds during single QDU gate application. i0={}, i1={}, dim={}", i0_raw, i1_raw, dim) });
             }

            let psi_0 = self.global_state.vector()[i0_raw]; // Amplitude for |...target=0...>
            let psi_1 = self.global_state.vector()[i1_raw]; // Amplitude for |...target=1...>

            // Apply the 2x2 matrix: [psi_0', psi_1'] = matrix * [psi_0, psi_1]
            new_vec[i0_raw] = matrix[0][0] * psi_0 + matrix[0][1] * psi_1;
            new_vec[i1_raw] = matrix[1][0] * psi_0 + matrix[1][1] * psi_1;
        }

        self.global_state = PotentialityState::new(new_vec);
        Ok(())
    }

    /// Gets the 2x2 matrix for a given interaction pattern ID.
    /// These matrices represent fundamental interaction patterns (`P_op`) derived
    /// from framework principles, interpreted as transformations on the
    /// {Quality0, Quality1} basis state potentiality vector.
    fn get_interaction_matrix(&self, pattern_id: &str) -> Result<[[Complex<f64>; 2]; 2], OnqError> {
         use std::f64::consts::{PI, FRAC_1_SQRT_2};
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
            "Identity" => Ok([ // Persistence
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), Complex::new(1.0, 0.0)]
            ]),
            "QualityFlip" => Ok([ // Binary Swap
                [Complex::zero(), Complex::new(1.0, 0.0)],
                [Complex::new(1.0, 0.0), Complex::zero()]
            ]),
            "PhaseIntroduce" => Ok([ // Phase (PI)
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), Complex::new(-1.0, 0.0)] // Phase PI
            ]),
            "Superposition" => Ok([ // Tentative state mixing
                [Complex::new(ONE_OVER_SQRT_2, 0.0), Complex::new(ONE_OVER_SQRT_2, 0.0)],
                [Complex::new(ONE_OVER_SQRT_2, 0.0), Complex::new(-ONE_OVER_SQRT_2, 0.0)]
            ]),
            "PhiRotate" => { //  'φ' Resonance Rotation (Ry(theta=PI/PHI))
                let theta = PI / PHI;
                let angle_over_2 = theta / 2.0;
                let cos_a = angle_over_2.cos();
                let sin_a = angle_over_2.sin();
                // Ry(theta) = [[cos(a), -sin(a)], [sin(a), cos(a)]] where a=theta/2
                Ok([
                    [Complex::new(cos_a, 0.0), Complex::new(-sin_a, 0.0)],
                    [Complex::new(sin_a, 0.0), Complex::new(cos_a, 0.0)]
                ])
            },
             "PhiXRotate" => { // 'φ' Resonance Rotation (Rx(theta=PI/PHI))
                let theta = PI / PHI;
                let angle_over_2 = theta / 2.0;
                let cos_a = angle_over_2.cos();
                let sin_a = angle_over_2.sin();
                // Rx(theta) = [[cos(a), -i*sin(a)], [-i*sin(a), cos(a)]] where a=theta/2
                Ok([
                    [Complex::new(cos_a, 0.0), -i * sin_a],
                    [-i * sin_a, Complex::new(cos_a, 0.0)]
                ])
            },
            "SqrtFlip" => Ok([ // Tentative: Partial quality flip interaction (Sqrt(X))
                [Complex::new(0.5, 0.5), Complex::new(0.5, -0.5)], // 0.5 * (1+i) , 0.5 * (1-i)
                [Complex::new(0.5, -0.5), Complex::new(0.5, 0.5)]  // 0.5 * (1-i) , 0.5 * (1+i)
            ]),
             "SqrtFlip_Inv" => Ok([ // Derived: Inverse partial flip (Sqrt(X) dagger)
                [Complex::new(0.5, -0.5), Complex::new(0.5, 0.5)], // 0.5 * (1-i) , 0.5 * (1+i)
                [Complex::new(0.5, 0.5), Complex::new(0.5, -0.5)]  // 0.5 * (1+i) , 0.5 * (1-i)
            ]),

            "HalfPhase" => Ok([ // Phase (PI/2) Quadrature Step
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), i] // e^(i*PI/2) = i
            ]),
            "QualitativeY" => Ok([ // Combined Flip/Phase Interaction Pattern
                [Complex::zero(), -i],
                [i, Complex::zero()]
            ]),
            "QuarterPhase" => Ok([ // (PI/4) - Finer Step
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), exp_i_pi_4]
            ]),
            "HalfPhase_Inv" => Ok([ // Inverse Phase (-PI/2)
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), -i] // e^(-i*PI/2) = -i
            ]),
            "QuarterPhase_Inv" => Ok([ // Inverse Phase (-PI/4)
                 [Complex::new(1.0, 0.0), Complex::zero()],
                 [Complex::zero(), exp_neg_i_pi_4]
            ]),
            _ => Err(OnqError::InvalidOperation{ message: format!("Interaction Pattern '{}' is not defined or derived yet", pattern_id)})
         }
    }

    /// Optional: Placeholder for state validation (e.g., check normalization)
    #[allow(dead_code)] // Temporarily allow unused function during development
    fn validate_state(&self) -> Result<(), OnqError> {
         let norm_sq: f64 = self.global_state.vector().iter().map(|c| c.norm_sqr()).sum();
         // Use a tolerance for floating point comparisons
         if (norm_sq - 1.0).abs() > 1e-9 {
            return Err(OnqError::Incoherence{ message: format!("State vector norm deviated significantly from 1: {}", norm_sq) });
         }
         Ok(())
    }

    /// Calculates the interpretive C1 Score (Phase Coherence) for a basis state k.
    /// Measures phase agreement with nearest neighbour basis states that have non-negligible amplitude.
    /// Returns a score between 0.0 and 1.0.
    fn calculate_c1_score(&self, k: usize, state_vector: &[Complex<f64>]) -> f64 {
        if self.num_qdus == 0 { return 1.0; } // Coherence is perfect for 0 QDUs

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
            if neighbour_k < state_vector.len() { // Ensure neighbour is within bounds
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

// Add this helper method inside impl SimulationEngine in src/simulation/engine.rs

    /// Applies a 4x4 matrix operation targeting two specific QDUs within the global state vector.
    /// Assumes standard tensor product structure for the global state vector.
    /// Used for operations like RelationalLock or other two-QDU interactions.
    fn apply_two_qdu_gate(
        &mut self,
        idx1: usize, // Index of the first QDU (e.g., corresponds to row index in 4x4 matrix's 2x2 blocks)
        idx2: usize, // Index of the second QDU (e.g., corresponds to col index in 4x4 matrix's 2x2 blocks)
        matrix: &[[Complex<f64>; 4]; 4], // The 4x4 matrix to apply
    ) -> Result<(), OnqError> {
        if idx1 == idx2 {
            return Err(OnqError::InvalidOperation { message: "Target indices for a two-QDU gate cannot be the same".to_string() });
        }

        let n = self.num_qdus;
        let dim = self.global_state.dim(); // 2^n
        let mut new_vec = vec![Complex::zero(); dim]; // Store results here

        // Determine bit positions (k-values) corresponding to indices
        // Ensure k1 is the higher-order bit position for consistent indexing logic
        let k1_raw = n - 1 - idx1;
        let k2_raw = n - 1 - idx2;
        let (k1, k2) = (k1_raw.max(k2_raw), k1_raw.min(k2_raw)); // k1 > k2

        let k1_mask = 1 << k1;
        let k2_mask = 1 << k2;

        // Iterate through all combinations of the other (n-2) qdus
        for i_other in 0..(dim / 4) { // 2^(n-2) iterations
            // Construct the base index by splitting i_other and inserting 0s at k1 and k2
            // Mask for bits above k1
            let _upper_mask = !((k1_mask << 1) - 1);
            // Mask for bits between k1 and k2
            let _middle_mask = ((1 << k1) - 1) & !((k2_mask << 1) - 1);
            // Mask for bits below k2
            let lower_mask = (1 << k2) - 1;

            let i_upper = (i_other >> (k1 - k2 - 1)) << (k1 + 1); // Shift bits originally above k1
            let i_middle = ((i_other >> k2) & ((1 << (k1 - k2 - 1)) - 1)) << (k2 + 1); // Shift bits originally between k1 and k2
            let i_lower = i_other & lower_mask; // Keep bits originally below k2

            let i_base = i_upper | i_middle | i_lower; // Base index with 0s at k1, k2

            // Calculate the four indices for the subspace {00, 01, 10, 11} for qdus at idx1, idx2
            // The order matters for applying the 4x4 matrix correctly.
            // Assuming matrix rows/cols correspond to |idx1_val, idx2_val> basis: |00> |01> |10> |11>
            // Let's map basis state bits (b1, b2) where b1 is for idx1 (pos k1_raw) and b2 for idx2 (pos k2_raw)
            // Index in 4x4 matrix = b1*2 + b2
            let indices = [
                i_base,                            // 00: b1=0 (k1_raw), b2=0 (k2_raw)
                i_base | (1 << k2_raw),            // 01: b1=0 (k1_raw), b2=1 (k2_raw)
                i_base | (1 << k1_raw),            // 10: b1=1 (k1_raw), b2=0 (k2_raw)
                i_base | (1 << k1_raw) | (1 << k2_raw), // 11: b1=1 (k1_raw), b2=1 (k2_raw)
            ];

            // Extract the four amplitudes
            let mut psi = [Complex::zero(); 4];
            for j in 0..4 {
                 if indices[j] < dim { // Check bounds just in case
                    psi[j] = self.global_state.vector()[indices[j]];
                 } else {
                      return Err(OnqError::SimulationError { message: format!("Calculated index out of bounds during two QDU gate application. Index={}, dim={}", indices[j], dim) });
                 }
            }

            // Apply the 4x4 matrix: psi' = matrix * psi
            let mut psi_prime = [Complex::zero(); 4];
            for row in 0..4 {
                for (col, _) in psi.iter().enumerate()  {
                    psi_prime[row] += matrix[row][col] * psi[col];
                }
            }

            // Write the results back into the new vector
            for j in 0..4 {
                new_vec[indices[j]] = psi_prime[j];
            }
        }

        self.global_state = PotentialityState::new(new_vec);
        Ok(())
    }
}

/// Provides the 2x2 matrix for the PhaseShift operation.
/// Derived form: Represents applying the phase factor `e^(i*theta)`
/// conditional on the QDU being in the `Quality1` state (index 1 of the 2x2 matrix).
fn phase_shift_matrix(theta: f64) -> [[Complex<f64>; 2]; 2] {
    [
        [Complex::new(1.0, 0.0), Complex::zero()],
        [Complex::zero(), Complex::new(theta.cos(), theta.sin())] // e^(i*theta)
    ]
}
