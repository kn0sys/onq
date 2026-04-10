// src/simulation/mod.rs

//! Simulates the execution of `onq::circuits::Circuit` based on framework principles.
//! This module contains the `Simulator` entry point and the internal `SimulationEngine`
//! responsible for managing and evolving the state according to derived rules.

// Make engine module crate visible for tests
pub(crate) mod engine;
mod results; // Changed visibility to pub(crate)

// Re-export the main public interface types
pub use results::SimulationResult;

// Import necessary types for the Simulator struct and its methods
use crate::circuits::Circuit;
use crate::core::OnqError;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{QduId, StableState};
    use crate::simulation::engine::SimulationEngine;
    use num_complex::Complex;
    use std::collections::HashSet;

    #[test]
    fn test_geometric_stabilization() {
        let mut qdus = HashSet::new();
        qdus.insert(QduId(0));

        let mut engine = SimulationEngine::init(&qdus).unwrap();
        let mut result = SimulationResult::new();

        // Target QDU 0. By default, it initializes in |0>
        engine.stabilize(&[QduId(0)], &mut result).unwrap();

        // It should deterministically resolve to 0
        let outcome = result.get_stable_state(&QduId(0)).unwrap();
        assert_eq!(outcome, &StableState::ResolvedQuality(0));
    }

    #[test]
    fn test_superposition_collapse() {
        let mut qdus = HashSet::new();
        qdus.insert(QduId(0));

        let mut engine = SimulationEngine::init(&qdus).unwrap();

        // Manually force QDU 0 into a |1> state for testing
        if let Some(tensor) = engine.get_state_mut_for_test().network.get_mut(&0) {
            tensor.core_state = [Complex::new(0.0, 0.0), Complex::new(1.0, 0.0)];
        }

        let mut result = SimulationResult::new();
        engine.stabilize(&[QduId(0)], &mut result).unwrap();

        // It should deterministically resolve to 1
        let outcome = result.get_stable_state(&QduId(0)).unwrap();
        assert_eq!(outcome, &StableState::ResolvedQuality(1));
    }
}
