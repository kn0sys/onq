// src/simulation/engine.rs
use crate::core::{QduId, PotentialityState, StableState, OnqError};
use crate::operations::Operation;
// NOTE: Does not directly use Circuit, operates on ops passed from Simulator
use std::collections::{HashMap, HashSet};
use num_complex::Complex;
use num_traits::{Zero, One};
// Placeholder stabilization requires rand crate
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
// Import SimulationResult for the stabilize function signature
use crate::simulation::SimulationResult;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::validation;
use crate::LockType;

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

    pub fn get_state(&self) -> &PotentialityState {
        // engine is guaranteed to be Some if called within a test after init
        &self.global_state
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
            Operation::RelationalLock { qdu1, qdu2, lock_type, establish } => {
                if !*establish {
                    // Currently, "releasing" a lock is a no-op.
                    // ONQ clearly define how structure "un-integrates" unitarily.
                    // We could potentially apply a randomizing unitary later if needed.
                    println!("[VM Warning] RelationalLock establish=false is currently a No-Op."); // Inform user
                    return Ok(());
                }

                // --- Projection Logic for establish=true ---
                let idx1 = *self.get_qdu_index(qdu1)?;
                let idx2 = *self.get_qdu_index(qdu2)?;
                if idx1 == idx2 {
                     return Err(OnqError::InvalidOperation { message: "RelationalLock requires two different QDUs.".to_string() });
                }

                // 1. Define the target subspace vector (normalized) based on LockType
                let sqrt2_inv = Complex::<f64>::one() / Complex::new(2.0f64.sqrt(), 0.0); // 1/sqrt(2)
                let target_subspace_vector: [Complex<f64>; 4] = match lock_type {
                    LockType::BellPhiPlus =>  [sqrt2_inv, Complex::zero(), Complex::zero(), sqrt2_inv],          // (1/sqrt(2))(|00> + |11>)
                    LockType::BellPhiMinus => [sqrt2_inv, Complex::zero(), Complex::zero(), -sqrt2_inv],         // (1/sqrt(2))(|00> - |11>)
                    LockType::BellPsiPlus =>  [Complex::zero(), sqrt2_inv, sqrt2_inv, Complex::zero()],          // (1/sqrt(2))(|01> + |10>)
                    LockType::BellPsiMinus => [Complex::zero(), sqrt2_inv, -sqrt2_inv, Complex::zero()],         // (1/sqrt(2))(|01> - |10>)
                };

                // 2. Perform projection onto the target state for the qdu1/qdu2 subspace
                // This involves calculating the overlap <target|psi_sub> and creating the
                // new state <target|psi_sub> * |target>. We then renormalize globally.
                self.project_onto_subspace_state(idx1, idx2, &target_subspace_vector)?;

            }
            Operation::Stabilize { .. } => {
                 return Err(OnqError::InvalidOperation { message: "Stabilize operation should not be passed directly to apply_operation".to_string() });
            }
        };
        validation::check_normalization(&self.global_state, None)?;
        Ok(())
    }

    /// Helper function to project the global state onto a specified target state
    /// within the 2-QDU subspace defined by idx1 and idx2. NON-UNITARY.
    fn project_onto_subspace_state(
        &mut self,
        idx1: usize,
        idx2: usize,
        target_state_normalized: &[Complex<f64>; 4], // e.g., Bell state vector [c00, c01, c10, c11]
    ) -> Result<(), OnqError> {

        let n = self.num_qdus;
        let dim = self.global_state.dim();
        let current_vector = self.global_state.vector();
        let mut projected_vector = vec![Complex::zero(); dim]; // Start with zero vector

        // Determine bit positions
        let k_idx1 = n - 1 - idx1;
        let k_idx2 = n - 1 - idx2;

        // Calculate the total overlap <target | current_psi>
        // by summing over all segments of the state vector.
        let mut total_overlap = Complex::zero();

        for i_other in 0..(dim / 4) {
            // Construct i_base for the other n-2 qdus
            let mut i_base = 0;
            let mut other_bit_mask = 1;
            for current_k in 0..n {
                 if current_k == k_idx1 || current_k == k_idx2 { } else {
                     if (i_other & other_bit_mask) != 0 { i_base |= 1 << current_k; }
                     other_bit_mask <<= 1;
                 }
            }
            // Get indices for the 4 states in this segment's subspace
            let k1_mask = 1 << k_idx1;
            let k2_mask = 1 << k_idx2;
            let indices = [ i_base, i_base | k2_mask, i_base | k1_mask, i_base | k1_mask | k2_mask ];

            // Check bounds
            if indices[3] >= dim { return Err(OnqError::SimulationError { message: format!("Internal error: Calculated index {} >= dimension {} in project_onto_subspace_state.", indices[3], dim) }); }

            // Extract current subspace amplitudes psi = [psi00, psi01, psi10, psi11]
            let psi_sub = [
                current_vector[indices[0]], current_vector[indices[1]],
                current_vector[indices[2]], current_vector[indices[3]]
            ];

            // Calculate overlap for this segment: <target | psi_sub> = sum(target[j].conj() * psi_sub[j])
            let mut segment_overlap = Complex::zero();
            for j in 0..4 {
                segment_overlap += target_state_normalized[j].conj() * psi_sub[j];
            }
            total_overlap += segment_overlap; // Accumulate total overlap <target | current_psi>
        }

        // Check if overlap is non-zero (projection is possible)
        let overlap_norm_sq: f64  = total_overlap.norm_sqr();
        if overlap_norm_sq < 1e-12 { // Use tolerance
            return Err(OnqError::Instability {
                 message: format!("Projection failed: State has zero overlap with target lock state ({:?}).", target_state_normalized) // Improve LockType display later
            });
        }

        // Construct the new state vector: total_overlap * |target_state> (in global space)
        // The projection operator is P = |target><target|. Applied to psi gives |target><target|psi> = <target|psi> * |target>
        // The resulting vector has the shape of |target_state> scaled by the complex overlap.
        for i_other in 0..(dim / 4) {
            // Construct i_base again (same logic as above)
             let mut i_base = 0;
             let mut other_bit_mask = 1;
             for current_k in 0..n { if current_k == k_idx1 || current_k == k_idx2 { } else { if (i_other & other_bit_mask) != 0 { i_base |= 1 << current_k; } other_bit_mask <<= 1; } }
             // Get indices again
             let k1_mask = 1 << k_idx1;
             let k2_mask = 1 << k_idx2;
             let indices = [ i_base, i_base | k2_mask, i_base | k1_mask, i_base | k1_mask | k2_mask ];

            // Check bounds
            if indices[3] >= dim { return Err(OnqError::SimulationError { message: format!("Internal error: Calculated index {} >= dimension {} in project_onto_subspace_state (write phase).", indices[3], dim) }); }

            // Assign the scaled target state components to the new vector
            for j in 0..4 {
                projected_vector[indices[j]] = total_overlap * target_state_normalized[j];
            }
        }


        // Renormalize the entire state vector
        // The norm squared of the projected state should be overlap_norm_sq
        let norm_factor = Complex::new(1.0 / overlap_norm_sq.sqrt(), 0.0);
        for amp in projected_vector.iter_mut() {
            *amp *= norm_factor;
        }

        // Update the global state
        self.global_state = PotentialityState::new(projected_vector);

        Ok(())
    }


    /// Performs the stabilization process based on interpreted framework principles.
    ///
    /// This simulates the resolution of `PotentialityState` into `StableState` by:
    /// 1. Calculating a Stability Score S(k) for each potential outcome basis state |k>,
    ///    based on interpretations of C_A (Phase Coherence) and C_B (Pattern Resonance),
    ///    combined with the state's amplitude |c_k|^2.
    /// 2. Filtering outcomes k to only include those meeting amplitude (> tol) criteria.
    /// 3. Deterministically selecting an outcome |k> from the filtered possibilities using a seeded PRNG
    ///    weighted by the calculated scores `S(k) = score_c1 * |c_k|^2` (where score_c1 influences probability).
    /// 4. Collapsing the global state vector to the chosen outcome basis state |k>.
    /// 5. Recording the resolved states for the specifically targeted QDUs based on |k>.
    ///
    /// **CRITICAL:** The C_A and C_B scoring functions are experimental
    /// C_B is currently interpreted as amplifying the contribution
    /// of amplitude itself for coherent states (using |c_k|^4 in the final score).
    pub(crate) fn stabilize(&mut self, targets: &[QduId], result: &mut SimulationResult) -> Result<(), OnqError> {
            if targets.is_empty() {
                return Ok(()); // Nothing to stabilize
            }
            validation::validate_state(&self.global_state, self.num_qdus, None, None, None)?;
            let dim = self.global_state.dim();
            let state_vector = self.global_state.vector(); // Get immutable borrow first

            // 1. Calculate stability scores S(k) for all possible outcomes k
            let mut valid_outcomes: Vec<(usize, f64)> = Vec::with_capacity(dim); // Stores (index, score S(k))
            let mut total_score = 0.0;

            for (k, _) in state_vector.iter().enumerate().take(dim) {
                let amplitude_sq = state_vector[k].norm_sqr();

                // Only consider states with non-negligible amplitude potentiality
                if amplitude_sq > 1e-12 {

                    let final_score = amplitude_sq; // S(k) = score_c1 * |c_k|^2
                    // Check if the final score is numerically valid and positive
                    if final_score.is_finite() && final_score > 1e-12 { // Use tolerance for score as well
                        valid_outcomes.push((k, final_score));
                        total_score += final_score;
                    }
                }
        }

        // Check if any outcome is possible according to our scoring
        if valid_outcomes.is_empty() || total_score < 1e-12 {
            // This might happen if the state is highly incoherent or only has negligible amplitudes
            return Err(OnqError::Instability { message: "Stabilization failed: No possible outcome met amplitude and Stabilization failed: No valid outcomes found (check state norm and phase coherence scores).".to_string() });
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
        let n = self.num_qdus;
        let k = n - 1 - target_idx; // Bit position (0 to n-1, from right)
        let k_mask = 1 << k;

        let dim = self.global_state.dim();
        let mut new_vec = vec![Complex::zero(); dim];

        for i in 0..dim / 2 { // i represents the bit pattern for the n-1 qubits *other* than target k
            // Calculate i0_raw: insert a 0 at bit position k into the pattern i
            let lower_part = i & ((1 << k) - 1); // Bits of i below position k
            let upper_part = i & !((1 << k) - 1); // Bits of i at or above position k
            let i0_raw = (upper_part << 1) | lower_part; // Shift upper part left by 1, make space, OR in lower part

            let i1_raw = i0_raw | k_mask; // Set the k-th bit to 1

            // Bounds check (optional but safe)
            if i1_raw >= dim { // Only need to check i1_raw as it's the largest
                return Err(OnqError::SimulationError { message: format!("Internal error: Calculated index {} >= dimension {} in apply_single_qdu_gate.", i1_raw, dim) });
            }

            let psi_0 = self.global_state.vector()[i0_raw];
            let psi_1 = self.global_state.vector()[i1_raw];

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
    #[allow(dead_code)]
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

    /// Applies a 4x4 matrix operation targeting two specific QDUs within the global state vector.
    /// Assumes standard tensor product structure for the global state vector.
    /// Used for operations like RelationalLock or ControlledInteraction.
    fn apply_two_qdu_gate(
        &mut self,
        idx1: usize, // Index of the first QDU (maps to first index in |b1,b2> basis for matrix)
        idx2: usize, // Index of the second QDU (maps to second index in |b1,b2> basis for matrix)
        matrix: &[[Complex<f64>; 4]; 4], // The 4x4 matrix (assuming basis |00>,|01>,|10>,|11>)
    ) -> Result<(), OnqError> {
        if idx1 == idx2 {
            return Err(OnqError::InvalidOperation { message: "Target indices for a two-QDU gate cannot be the same".to_string() });
        }

        let n = self.num_qdus;
        let dim = self.global_state.dim(); // 2^n
        let mut new_vec = self.global_state.vector().to_vec(); // Operate on a mutable copy

        // Determine bit positions (k-values, 0 to n-1 from right) corresponding directly to indices
        let k_idx1 = n - 1 - idx1;
        let k_idx2 = n - 1 - idx2;

        // Iterate through all 2^(n-2) combinations of the 'other' qdus not being targeted
        for i_other in 0..(dim / 4) {
            // Construct the base index 'i_base' which has the correct bit pattern for
            // the 'other' qdus, and zeros at the target positions k_idx1 and k_idx2.
            let mut i_base = 0;
            let mut other_bit_mask = 1; // Tracks the bit significance in i_other
            for current_k in 0..n {
                if current_k == k_idx1 || current_k == k_idx2 {
                    // This position is for target qdus, leave it 0 in i_base
                } else {
                    // This position corresponds to one of the 'other' qdus.
                    // Set the bit in i_base if the corresponding bit is set in i_other.
                    if (i_other & other_bit_mask) != 0 {
                        i_base |= 1 << current_k;
                    }
                    // Move to the next bit significance for i_other
                    other_bit_mask <<= 1;
                }
            }

            // Calculate the four subspace indices by setting bits k_idx1 and k_idx2 in i_base
            let k1_mask = 1 << k_idx1;
            let k2_mask = 1 << k_idx2;

            // Indices corresponding to |idx1=0,idx2=0>, |0,1>, |1,0>, |1,1> basis states
            let indices = [
                i_base,                    // 00
                i_base | k2_mask,          // 01
                i_base | k1_mask,          // 10
                i_base | k1_mask | k2_mask,// 11
            ];

            // Extract the current amplitudes for this subspace
            // Note: Need to ensure indices are valid before accessing vector - though i_base logic should guarantee this if dim >= 4
             if indices[3] >= dim { // Check highest index only is sufficient
                 return Err(OnqError::SimulationError { message: format!("Internal error: Calculated index {} >= dimension {} in apply_two_qdu_gate.", indices[3], dim) });
             }
            let psi = [
                self.global_state.vector()[indices[0]], // psi_00
                self.global_state.vector()[indices[1]], // psi_01
                self.global_state.vector()[indices[2]], // psi_10
                self.global_state.vector()[indices[3]], // psi_11
            ];


            // Apply the 4x4 matrix M: psi_prime = M * psi
            // (Assuming matrix is in basis order |00>,|01>,|10>,|11>)
            let mut psi_prime = [Complex::zero(); 4];
            for row in 0..4 {
                for (col, _) in psi.iter().enumerate() {
                    psi_prime[row] += matrix[row][col] * psi[col];
                }
            }

            // Write the results back into the new vector (which was cloned initially)
            for j in 0..4 {
                 new_vec[indices[j]] = psi_prime[j];
            }
        } // end loop over i_other

        // Update the engine's state vector
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
