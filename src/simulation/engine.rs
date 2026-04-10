use crate::core::{OnqError, PotentialityState, QduId, StableState};
use crate::operations::Operation;
use crate::simulation::SimulationResult;
use crate::validation;
use num_complex::Complex;
use num_traits::identities::Zero;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub(crate) struct SimulationEngine {
    /// Maps abstract QDU IDs to their physical coordinate index if needed,
    /// though PotentialityState now handles the main network map.
    qdu_indices: HashMap<QduId, u64>,

    /// The localized Tensor Network bounded by the Isotropic Vector Matrix
    global_state: PotentialityState,

    /// Number of QDUs being simulated
    num_qdus: usize,
}

impl SimulationEngine {
    /// Initializes the engine. We now boot up the "Zero-Point" Vector Equilibrium!
    pub(crate) fn init(qdu_ids: &HashSet<QduId>) -> Result<Self, OnqError> {
        if qdu_ids.is_empty() {
            return Err(OnqError::InvalidOperation {
                message: "Cannot initialize simulation engine with zero QDUs".to_string(),
            });
        }

        let num_qdus = qdu_ids.len();
        let mut qdu_indices = HashMap::new();

        // Map the requested QDUs to the 64 available IVM slots
        for (i, &qdu_id) in qdu_ids.iter().enumerate() {
            if i >= 64 {
                return Err(OnqError::SimulationError {
                    message: "Hardware Limit Exceeded: The Isotropic Vector Matrix supports a maximum of 64 localized QDUs.".to_string()
                });
            }
            qdu_indices.insert(qdu_id, i as u64);
        }

        // Initialize the Tensor Network
        let global_state = PotentialityState::new();

        Ok(Self {
            qdu_indices,
            global_state,
            num_qdus,
        })
    }

    pub fn get_state(&self) -> &PotentialityState {
        &self.global_state
    }

    #[cfg(test)]
    pub(crate) fn set_state(&mut self, state: PotentialityState) -> Result<(), OnqError> {
        self.global_state = state;
        Ok(())
    }

    /// The new O(1) Localized Execution Engine
    pub(crate) fn apply_operation(&mut self, op: &Operation) -> Result<(), OnqError> {
        match op {
            Operation::PhaseShift { target, theta } => {
                let physical_id = self.get_physical_id(target)?;
                let matrix = phase_shift_matrix(*theta);
                self.global_state
                    .apply_local_operation(physical_id, &matrix)
                    .map_err(|e| OnqError::SimulationError { message: e })?;
            }

            Operation::InteractionPattern { target, pattern_id } => {
                let physical_id = self.get_physical_id(target)?;
                let matrix = self.get_interaction_matrix(pattern_id)?;
                self.global_state
                    .apply_local_operation(physical_id, &matrix)
                    .map_err(|e| OnqError::SimulationError { message: e })?;
            }

            Operation::ControlledInteraction {
                control,
                target,
                pattern_id,
            } => {
                let phys_control = self.get_physical_id(control)?;
                let phys_target = self.get_physical_id(target)?;

                // 1. Enforce IVM Geometry & Build the Bond
                self.global_state
                    .apply_entanglement(phys_control, phys_target)
                    .map_err(|e| OnqError::InvalidOperation { message: e })?;

                // 2. Apply the conditional logic to the target's core state
                let matrix = self.get_interaction_matrix(pattern_id)?;

                // (Note: In a full PEPS network, applying U to a bonded state updates the bond tensor.
                // For now, we apply it locally to simulate the gate completion).
                self.global_state
                    .apply_local_operation(phys_target, &matrix)
                    .map_err(|e| OnqError::SimulationError { message: e })?;
            }

            Operation::RelationalLock {
                qdu1,
                qdu2,
                establish,
                ..
            } => {
                if !*establish {
                    return Ok(());
                }

                let phys_1 = self.get_physical_id(qdu1)?;
                let phys_2 = self.get_physical_id(qdu2)?;

                // RelationalLock is now purely geometric bonding! No massive 4x4 projections.
                self.global_state
                    .apply_entanglement(phys_1, phys_2)
                    .map_err(|e| OnqError::InvalidOperation { message: e })?;
            }

            Operation::Stabilize { .. } => {
                return Err(OnqError::InvalidOperation {
                    message: "Stabilize operation should not be passed directly to apply_operation"
                        .to_string(),
                });
            }
        };

        // Optional: Localized norm check
        // validation::check_normalization(&self.global_state, None)?;
        Ok(())
    }

    /// Helper to map abstract QduId to the physical u64 IVM node ID
    fn get_physical_id(&self, qdu_id: &QduId) -> Result<u64, OnqError> {
        self.qdu_indices
            .get(qdu_id)
            .copied()
            .ok_or_else(|| OnqError::ReferenceViolation {
                message: format!("QDU {} not mapped to physical matrix.", qdu_id),
            })
    }

    /// Handles external calls from the Simulator/VM to stabilize specific QDUs
    pub(crate) fn stabilize(
        &mut self,
        targets: &[QduId],
        result: &mut SimulationResult,
    ) -> Result<(), OnqError> {
        if targets.is_empty() {
            return Ok(());
        }

        // 1. Map abstract QDU targets to physical IVM nodes
        let mut target_ids = Vec::new();
        for qdu_id in targets {
            target_ids.push(self.get_physical_id(qdu_id)?);
        }

        // 2. Run the deterministic, geometric collapse!
        let outcomes = self
            .global_state
            .stabilize(&target_ids)
            .map_err(|e| OnqError::SimulationError { message: e })?;

        // 3. Record the results back into the VM's log
        for target_qdu_id in targets {
            let phys_id = self.get_physical_id(target_qdu_id)?;
            if let Some(&quality) = outcomes.get(&phys_id) {
                result.record_stable_state(
                    *target_qdu_id,
                    StableState::ResolvedQuality(quality as u64),
                );
            }
        }

        Ok(())
    }

    /// Gets the 2x2 matrix for a given interaction pattern ID.
    fn get_interaction_matrix(&self, pattern_id: &str) -> Result<[[Complex<f64>; 2]; 2], OnqError> {
        use std::f64::consts::{FRAC_1_SQRT_2, PI};
        const PHI: f64 = 1.618_033_988_749_895;
        let i = Complex::i();
        let exp_i_pi_4 = Complex::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2);
        let exp_neg_i_pi_4 = Complex::new(FRAC_1_SQRT_2, -FRAC_1_SQRT_2);

        match pattern_id {
            "Identity" => Ok([
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), Complex::new(1.0, 0.0)],
            ]),
            "QualityFlip" => Ok([
                [Complex::zero(), Complex::new(1.0, 0.0)],
                [Complex::new(1.0, 0.0), Complex::zero()],
            ]),
            "PhaseIntroduce" => Ok([
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), Complex::new(-1.0, 0.0)],
            ]),
            "Superposition" => Ok([
                [
                    Complex::new(FRAC_1_SQRT_2, 0.0),
                    Complex::new(FRAC_1_SQRT_2, 0.0),
                ],
                [
                    Complex::new(FRAC_1_SQRT_2, 0.0),
                    Complex::new(-FRAC_1_SQRT_2, 0.0),
                ],
            ]),
            "PhiRotate" => {
                let theta = PI / PHI;
                let (sin_a, cos_a) = (theta / 2.0).sin_cos();
                Ok([
                    [Complex::new(cos_a, 0.0), Complex::new(-sin_a, 0.0)],
                    [Complex::new(sin_a, 0.0), Complex::new(cos_a, 0.0)],
                ])
            }
            "PhiXRotate" => {
                let theta = PI / PHI;
                let (sin_a, cos_a) = (theta / 2.0).sin_cos();
                Ok([
                    [Complex::new(cos_a, 0.0), -i * sin_a],
                    [-i * sin_a, Complex::new(cos_a, 0.0)],
                ])
            }
            "SqrtFlip" => Ok([
                [Complex::new(0.5, 0.5), Complex::new(0.5, -0.5)],
                [Complex::new(0.5, -0.5), Complex::new(0.5, 0.5)],
            ]),
            "SqrtFlip_Inv" => Ok([
                [Complex::new(0.5, -0.5), Complex::new(0.5, 0.5)],
                [Complex::new(0.5, 0.5), Complex::new(0.5, -0.5)],
            ]),
            "HalfPhase" => Ok([
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), i],
            ]),
            "QualitativeY" => Ok([[Complex::zero(), -i], [i, Complex::zero()]]),
            "QuarterPhase" => Ok([
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), exp_i_pi_4],
            ]),
            "HalfPhase_Inv" => Ok([
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), -i],
            ]),
            "QuarterPhase_Inv" => Ok([
                [Complex::new(1.0, 0.0), Complex::zero()],
                [Complex::zero(), exp_neg_i_pi_4],
            ]),
            _ => Err(OnqError::InvalidOperation {
                message: format!("Interaction Pattern '{}' is not defined", pattern_id),
            }),
        }
    }
} // <-- END OF impl SimulationEngine

/// Provides the 2x2 matrix for the PhaseShift operation.
fn phase_shift_matrix(theta: f64) -> [[Complex<f64>; 2]; 2] {
    [
        [Complex::new(1.0, 0.0), Complex::zero()],
        [Complex::zero(), Complex::new(theta.cos(), theta.sin())],
    ]
}
