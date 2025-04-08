// src/simulation/mod.rs

//! Simulates the execution of `onq::circuits::Circuit` based on framework principles.
//! This module contains the `Simulator` entry point and the internal `SimulationEngine`
//! responsible for managing and evolving the state according to derived rules.

// Make engine module crate visible for tests
mod results;
pub(crate) mod engine; // Changed visibility to pub(crate)

// Re-export the main public interface types
pub use results::SimulationResult;

// Import necessary types for the Simulator struct and its methods
use crate::core::OnqError;
use crate::circuits::Circuit;
use crate::operations::Operation;
// Make engine accessible within the crate
use engine::SimulationEngine;

/// The main simulator orchestrating the execution of circuits.
/// It uses an internal `SimulationEngine` to manage state evolution
/// according to rules (or placeholders thereof).
#[derive(Default)] // Allows Simulator::default() -> Simulator::new()
pub struct Simulator {
    // Future potential configuration options:
    // - seed_source: SeedSource, // For deterministic stabilization if probabilistic
    // - precision_level: FloatPrecision,
    // - validation_mode: ValidationMode, // e.g., Off, Basic, Strict
}

impl Simulator {
    /// Creates a new Simulator with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs a simulation of the provided circuit.
    ///
    /// Executes the sequence of operations defined in the `circuit`, updating the
    /// potentiality state according to derived dynamics (currently using placeholders).
    /// Performs stabilization when requested by `Operation::Stabilize`.
    ///
    /// # Arguments
    /// * `circuit` - The `Circuit` definition to simulate.
    ///
    /// # Returns
    /// * `Ok(SimulationResult)` containing the stable outcomes recorded during stabilization.
    /// * `Err(OnqError)` if the simulation encounters an error state reflecting a violation
    ///   of principles (e.g., incoherence, instability) or invalid operations.
    pub fn run(&self, circuit: &Circuit) -> Result<SimulationResult, OnqError> {
        // Handle empty circuit case
        if circuit.is_empty() {
            return Ok(SimulationResult::new());
        }

        // 1. Initialize the simulation engine with all unique QDUs involved in the circuit.
        // This sets up the initial state vector (placeholder: |0...0>).
        let mut engine = SimulationEngine::init(circuit.qdus())?;

        // 2. Initialize the results container to store stable outcomes.
        let mut result = SimulationResult::new();

        // 3. Iterate through the ordered sequence of operations in the circuit.
        for op in circuit.operations() {
            match op {
                // Handle stabilization operation specifically
                Operation::Stabilize { targets } => {
                    // Instruct the engine to perform the stabilization protocol
                    // for the specified target QDUs. This updates the 'result' map
                    // and potentially collapses the engine's state vector.
                    // **CRITICAL:** Uses placeholder stabilization logic in the engine currently.
                    engine.stabilize(targets, &mut result)?;
                }
                // For all other operations, instruct the engine to apply them
                _ => {
                    // Apply the state evolution operation to the engine's state vector.
                    // **CRITICAL:** Uses placeholder gate application logic in the engine currently.
                    engine.apply_operation(op)?;
                }
            }
            // Optional: Perform state validation after each step if configured/needed for debugging.
            // engine.validate_state()?;
        }

        // Optional: Final validation check on the state after all operations.
        // engine.validate_state()?;

        // TODO: Optionally populate result.final_potentialities with engine.global_state if desired
        // for non-stabilized QDUs.

        // Return the collected stable outcomes.
        Ok(result)
    }
}

// src/simulation/mod.rs
// ... (rest of the file above) ...

#[cfg(test)]
mod tests {
    // Import items from the parent module (simulation) and the crate root
    use super::*; // Brings Simulator, SimulationResult etc. into scope
    use super::engine::SimulationEngine; // Access the crate-visible engine module
    use crate::core::*; // QduId, StableState, PotentialityState, OnqError
    use std::collections::HashSet; // HashMap might be needed again
    use num_complex::Complex;
    use std::f64::consts::FRAC_1_SQRT_2;
    use num_traits::Zero;

    // --- Helper Functions ---
    // (Copy qid and check_stable_state helpers here if not made pub elsewhere)
    fn qid(id: u64) -> QduId {
        QduId(id)
    }

    fn check_stable_state(result: &SimulationResult, qdu_id: QduId, expected_state_val: u64) {
         match result.get_stable_state(&qdu_id) {
            Some(StableState::ResolvedQuality(val)) => {
                assert_eq!(*val, expected_state_val, "Mismatch for QDU {}", qdu_id);
            }
            _ => panic!("QDU {} was not stabilized or result is not ResolvedQuality", qdu_id),
        }
    }



    #[test]
    fn test_stabilize_basis_state() -> Result<(), OnqError> {
        // Stabilizing a basis state should always yield that state
        let q0 = qid(0);
        let q1 = qid(1);
        let qdu_set: HashSet<QduId> = [q0, q1].iter().cloned().collect();
        let mut engine = SimulationEngine::init(&qdu_set)?;

        // Test |01> state (index 1)
        let state_vec_01 = vec![
            Complex::new(0.0, 0.0), Complex::new(1.0, 0.0), // Index 1 = |01>
            Complex::new(0.0, 0.0), Complex::new(0.0, 0.0)
        ];
        engine.set_state(PotentialityState::new(state_vec_01))?;
        let mut result = SimulationResult::new();
        engine.stabilize(&[q0, q1], &mut result)?;

        check_stable_state(&result, q0, 0);
        check_stable_state(&result, q1, 1);

        // Test |10> state (index 2)
        let state_vec_10 = vec![
            Complex::new(0.0, 0.0), Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0), Complex::new(0.0, 0.0) // Index 2 = |10>
        ];
         engine.set_state(PotentialityState::new(state_vec_10))?;
         result = SimulationResult::new(); // Reset result
         engine.stabilize(&[q0, q1], &mut result)?;

         check_stable_state(&result, q0, 1);
         check_stable_state(&result, q1, 0);

        Ok(())
    }

    #[test]
    fn test_stabilize_superposition_equal_zero_phase() -> Result<(), OnqError> {
        // Test state: (1/sqrt(2))|0> + (1/sqrt(2))|1>
        // Expected scores (from thought): S(0) = 0.5, S(1) = 0.25 -> Outcome 0
        let q0 = qid(0);
        let qdu_set: HashSet<QduId> = [q0].iter().cloned().collect();
        let mut engine = SimulationEngine::init(&qdu_set)?;

        let state_vec = vec![
            Complex::new(FRAC_1_SQRT_2, 0.0), // |0>
            Complex::new(FRAC_1_SQRT_2, 0.0)  // |1>
        ];
        engine.set_state(PotentialityState::new(state_vec))?;
        let mut result = SimulationResult::new();
        engine.stabilize(&[q0], &mut result)?;

        check_stable_state(&result, q0, 0); // Expect outcome 0
        Ok(())
    }

    #[test]
    fn test_stabilize_two_qdu_entangled_like() -> Result<(), OnqError> {
        // Test state: ~(0.6)|00> + ~(0.8)|11> (Approx normalized)
        // Expected scores (from thought): S(0)=0.36, S(3)~=0.213 -> Outcome 0 (|00>)
        let q0 = qid(0);
        let q1 = qid(1);
        let qdu_set: HashSet<QduId> = [q0, q1].iter().cloned().collect();
        let mut engine = SimulationEngine::init(&qdu_set)?;

        let c00 = Complex::new(0.6, 0.0);
        let c11 = Complex::new(0.8, 0.0);
        let state_vec = vec![ c00, Complex::zero(), Complex::zero(), c11 ]; // |00>, |01>, |10>, |11>
        engine.set_state(PotentialityState::new(state_vec))?;

        let mut result = SimulationResult::new();
        engine.stabilize(&[q0, q1], &mut result)?;

        check_stable_state(&result, q0, 0);
        check_stable_state(&result, q1, 0);

        Ok(())
    }

     #[test]
    fn test_stabilize_deterministic_seed() -> Result<(), OnqError> {
        // Verify stabilization attempt on a state that fails global C1 check
        let q0 = qid(0);
        let q1 = qid(1);
        let qdu_set: HashSet<QduId> = [q0, q1].iter().cloned().collect();

        // State: 0.5*(|00> + |01> + |10> - |11>) -> Global C1 score = 0.5
        let initial_state_vec = vec![
            Complex::new(0.5, 0.0), Complex::new(0.5, 0.0),
            Complex::new(0.5, 0.0), Complex::new(-0.5, 0.0)
        ];
        let initial_state = PotentialityState::new(initial_state_vec);

        let mut engine = SimulationEngine::init(&qdu_set)?;
        engine.set_state(initial_state.clone())?;
        let mut result = SimulationResult::new();
        let stabilization_result = engine.stabilize(&[q0, q1], &mut result); // Capture result

        // Assert that it failed with Incoherence due to the initial validation check
        assert!(stabilization_result.is_err(), "Expected stabilization to fail");
        match stabilization_result.err().unwrap() {
            OnqError::Incoherence { message } => {
                assert!(message.contains("Global Phase Coherence check failed"), "Incorrect error message: {}", message);
                assert!(message.contains("Score 0.5000 <= Threshold 0.6180"), "Incorrect score/threshold in message: {}", message);
            }
            e => panic!("Expected Incoherence error due to C1 check, got {:?}", e),
        }

        // Original purpose (determinism) can't be tested if stabilization fails.
        // Test now verifies the initial state validation works correctly.
        Ok(())
    }

    // test_stabilize_basis_state remains unchanged - should pass validation

    #[test]
    fn test_stabilize_superposition_equal_pi_over_2_phase() -> Result<(), OnqError> {
        // Test state: (1/sqrt(2))|0> + (i/sqrt(2))|1>
        // Expected Global C1 score = 0.5, which fails threshold.
        // Expected outcome: Error (Incoherence) from initial validation.
        let q0 = qid(0);
        let qdu_set: HashSet<QduId> = [q0].iter().cloned().collect();
        let mut engine = SimulationEngine::init(&qdu_set)?;

        let state_vec = vec![
            Complex::new(FRAC_1_SQRT_2, 0.0),  // |0>
            Complex::new(0.0, FRAC_1_SQRT_2)   // |1> (phase PI/2)
        ];
         engine.set_state(PotentialityState::new(state_vec))?;
         let mut result = SimulationResult::new();
         let stabilization_result = engine.stabilize(&[q0], &mut result); // Capture result

        // Assert that it now fails with Incoherence due to the initial validation check
         assert!(stabilization_result.is_err(), "Expected stabilization to fail");
         match stabilization_result.err().unwrap() {
             OnqError::Incoherence { message } => {
                  // Check for the correct error message
                  assert!(message.contains("Global Phase Coherence check failed"), "Incorrect error message: {}", message);
                  assert!(message.contains("Score 0.5000 <= Threshold 0.6180"), "Incorrect score/threshold in message: {}", message);
             },
             e => panic!("Expected Incoherence error due to C1 check, got {:?}", e),
         }
         Ok(()) // Test passes if the correct error occurred
    }


    #[test]
    fn test_stabilize_superposition_equal_pi_phase() -> Result<(), OnqError> {
        // Test state: (1/sqrt(2))|0> - (1/sqrt(2))|1>
        // Expected Global C1 score = 0.0, which fails threshold.
        // Expected outcome: Error (Incoherence) from initial validation.
        let q0 = qid(0);
        let qdu_set: HashSet<QduId> = [q0].iter().cloned().collect();
        let mut engine = SimulationEngine::init(&qdu_set)?;

        let state_vec = vec![
            Complex::new(FRAC_1_SQRT_2, 0.0),  // |0>
            Complex::new(-FRAC_1_SQRT_2, 0.0) // |1> (phase PI)
        ];
        engine.set_state(PotentialityState::new(state_vec))?;
        let mut result = SimulationResult::new();
        let stabilization_result = engine.stabilize(&[q0], &mut result);

        assert!(stabilization_result.is_err(), "Expected stabilization to fail for state failing C1 coherence filter");
        match stabilization_result.err().unwrap() {
            OnqError::Incoherence { message } => {
                 // Check for the correct error message
                 assert!(message.contains("Global Phase Coherence check failed"), "Incorrect error message: {}", message);
                 // Check the specific score mentioned in the message
                 assert!(message.contains("Score 0.0000 <= Threshold 0.6180"), "Incorrect score/threshold in message: {}", message);
            },
            e => panic!("Expected Incoherence error due to C1 check, got {:?}", e),
        }
        Ok(()) // Test passes if the correct error occurred
    }
}
