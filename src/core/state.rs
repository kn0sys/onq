use num_complex::Complex;
use std::collections::HashMap;
use std::fmt;

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

use crate::topology::IvmTopology;

/// A localized state tensor for a single QDU
#[derive(Clone, Debug)]
pub struct LocalTensor {
    /// The minimal binary basis {Quality0, Quality1}
    pub core_state: [Complex<f64>; 2],

    /// Entanglement bonds to physical neighbors in the IVM.
    pub bonds: HashMap<u64, Vec<Complex<f64>>>,
}

impl LocalTensor {
    /// Initializes a QDU in the absolute baseline state (|Q0>) with no entanglements.
    pub fn new_baseline() -> Self {
        LocalTensor {
            core_state: [
                Complex::new(1.0, 0.0), // 100% Probability of Quality0
                Complex::new(0.0, 0.0), // 0% Probability of Quality1
            ],
            bonds: HashMap::new(),
        }
    }
}

/// The geometrically bound quantum state engine
#[derive(Clone, Debug)]
pub struct GeometricPotentialityState {
    /// The distributed state network, mapping QDU IDs to their local tensors
    pub network: HashMap<u64, LocalTensor>,

    /// The immutable structural rules governing the network
    pub topology: IvmTopology,
}

impl Default for GeometricPotentialityState {
    fn default() -> GeometricPotentialityState {
        GeometricPotentialityState::new()
    }
}

impl GeometricPotentialityState {
    /// Initializes the flat baseline state across all 64 nodes
    pub fn new() -> Self {
        let topology = IvmTopology::new();
        let mut network = HashMap::new();

        for &qdu_id in topology.nodes.keys() {
            network.insert(qdu_id, LocalTensor::new_baseline());
        }

        GeometricPotentialityState { network, topology }
    }

    /// Initializes the Vector Equilibrium (The Zero-Point Vacuum)
    /// Finds the innermost nodes of the IVM and prepares them in a highly coherent state.
    pub fn new_equilibrium() -> Self {
        let mut state = Self::new();

        // In our {-3, -1, 1, 3} grid, the innermost core nodes are the ones
        // constructed solely from {-1, 1}. There are exactly 8 of these
        // coordinate combinations representing the central Star Tetrahedron!
        let inner_bound = 1;

        for (&qdu_id, coord) in &state.topology.nodes {
            if coord.x.abs() == inner_bound
                && coord.y.abs() == inner_bound
                && coord.z.abs() == inner_bound
            {
                // Fetch the local tensor and push it into a coherent superposition (|+> analog)
                if let Some(tensor) = state.network.get_mut(&qdu_id) {
                    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
                    tensor.core_state =
                        [Complex::new(inv_sqrt2, 0.0), Complex::new(inv_sqrt2, 0.0)];
                    // Later, we will establish the initial entanglement bonds here to
                    // lock this core into the true Vector Equilibrium geometry!
                }
            }
        }

        state
    }

    /// Deterministically resolves the potentiality of specific QDUs.
    /// Replaces probabilistic measurement with a Golden Ratio (1/phi) coherence filter.
    pub fn stabilize(&mut self, targets: &[u64]) -> Result<HashMap<u64, u8>, String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut outcomes = HashMap::new();
        let inverse_phi = 0.61803398875; // The Golden Ratio Coherence Threshold

        for &target in targets {
            let tensor = self
                .network
                .get(&target)
                .ok_or_else(|| format!("QDU {} does not exist in the network.", target))?;

            // 1. Calculate local amplitudes (interpreted as Phase Coherence base)
            let prob_0 = tensor.core_state[0].norm_sqr();
            let prob_1 = tensor.core_state[1].norm_sqr();

            // 2. The Deterministic Seed
            // We hash the exact floating-point memory of the core state to generate
            // a strictly deterministic pseudo-random number.
            let mut hasher = DefaultHasher::new();
            prob_0.to_bits().hash(&mut hasher);
            prob_1.to_bits().hash(&mut hasher);
            let seed = hasher.finish();

            // Generate a deterministic float between 0.0 and 1.0
            let prng_val = (seed % 1000000) as f64 / 1000000.0;

            // 3. The Coherence Filter & Selection
            // If a state breaches the Golden Ratio threshold, it forces structural reality.
            // Otherwise, the deterministic PRNG collapses the wave based on weight.
            let outcome = if prob_0 > inverse_phi {
                0 // Quality0 has achieved dominant structural coherence
            } else if prob_1 > inverse_phi {
                1 // Quality1 has achieved dominant structural coherence
            } else {
                // Neither breached the threshold natively; use the deterministic PRNG
                if prng_val <= (prob_0 / (prob_0 + prob_1)) {
                    0
                } else {
                    1
                }
            };

            outcomes.insert(target, outcome);
        }

        // 4. Collapse the Geometry
        // Once the outcome is determined, we sever the potentiality and lock it into reality.
        for (&target, &outcome) in &outcomes {
            let tensor = self.network.get_mut(&target).unwrap();

            if outcome == 0 {
                tensor.core_state = [Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
            } else {
                tensor.core_state = [Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)];
            }

            // Sever the entanglement bonds! The potentiality has collapsed,
            // freeing the adjacent geometry to form new connections.
            tensor.bonds.clear();
        }

        Ok(outcomes)
    }

    /// Applies a single-QDU operation natively (O(1) complexity!)
    pub fn apply_local_operation(
        &mut self,
        target: u64,
        matrix: &[[Complex<f64>; 2]; 2],
    ) -> Result<(), String> {
        let tensor = self
            .network
            .get_mut(&target)
            .ok_or_else(|| format!("QDU {} does not exist in the network.", target))?;

        let current_state = tensor.core_state;

        // Standard matrix * vector multiplication, but completely localized to one node
        tensor.core_state[0] = matrix[0][0] * current_state[0] + matrix[0][1] * current_state[1];
        tensor.core_state[1] = matrix[1][0] * current_state[0] + matrix[1][1] * current_state[1];

        Ok(())
    }

    /// Enforces the Locality Rule for two-QDU operations
    /// Enforces the Locality Rule and establishes a shared Bond Tensor between two adjacent QDUs
    pub fn apply_entanglement(&mut self, control: u64, target: u64) -> Result<(), String> {
        // 1. The Locality Rule
        if !self.topology.are_adjacent(control, target) {
            return Err(format!(
                "Topological Error: QDU {} and QDU {} are not physically adjacent in the IVM. Route through intermediate nodes.",
                control, target
            ));
        }

        // 2. To avoid Rust's borrow checker issues when mutating two items in the same HashMap,
        // we temporarily extract their core states.
        let control_state = self.network.get(&control).unwrap().core_state;
        let target_state = self.network.get(&target).unwrap().core_state;

        // 3. Create the initial Bond Tensor (a 2x2 matrix flattened into a Vec of length 4).
        // This represents the joint probability space of just these two adjacent nodes.
        // T_{ij} = Control_{i} * Target_{j}
        let mut bond_tensor = vec![Complex::new(0.0, 0.0); 4];
        bond_tensor[0] = control_state[0] * target_state[0]; // |00>
        bond_tensor[1] = control_state[0] * target_state[1]; // |01>
        bond_tensor[2] = control_state[1] * target_state[0]; // |10>
        bond_tensor[3] = control_state[1] * target_state[1]; // |11>

        // 4. Update both LocalTensors to hold a reference to this shared bond
        if let Some(c_tensor) = self.network.get_mut(&control) {
            c_tensor.bonds.insert(target, bond_tensor.clone());
        }

        if let Some(t_tensor) = self.network.get_mut(&target) {
            // Depending on the tensor network definition, the target might store the transposed bond,
            // but for simplicity in this baseline, they share the identical state map.
            t_tensor.bonds.insert(control, bond_tensor);
        }

        Ok(())
    }

    /// Approximates the global norm of the tensor network.
    /// For locally unitary states, this ensures the system hasn't leaked probability.
    pub fn global_norm_sq(&self) -> f64 {
        let mut total_norm = 1.0;
        for tensor in self.network.values() {
            let local_norm_sq = tensor.core_state[0].norm_sqr() + tensor.core_state[1].norm_sqr();
            total_norm *= local_norm_sq;
        }
        total_norm
    }
}

/// Patch for state migration
pub type PotentialityState = GeometricPotentialityState;
