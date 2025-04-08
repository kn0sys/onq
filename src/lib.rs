// src/lib.rs

//! `onq` - A library for simulating quantum information processing
//!
//! This library provides structures and tools derived strictly from nature
//! to model phenomena analogous to quantum computation.

pub mod core;
pub mod operations;
pub mod circuits;
pub mod simulation;
pub mod vm;
pub mod validation;

// Re-export the most common types for easier top-level use
pub use core::{QduId, PotentialityState, StableState, OnqError}; // Removed Qdu, ReferenceFrame unless needed publicly
pub use operations::Operation;
pub use circuits::{Circuit, CircuitBuilder};
pub use simulation::{Simulator, SimulationResult};
pub use vm::{Instruction, program::LockType, Program, ProgramBuilder};
pub use validation::{
    check_normalization,
    check_phase_coherence,
    calculate_global_phase_coherence, // Also export the calculator function
    validate_state,
};

// Example 1: Single QDU Superposition and Stabilization
// Demonstrates creating a superposition using a derived gate and observing
// the outcome based on the stabilization logic.
/// ```
/// use onq::{QduId, CircuitBuilder, Operation, Simulator, StableState, OnqError};
/// use std::f64::consts::PI; /// Used for potential phase shifts if added later
///
/// // Helper for creating QduId
/// fn qid(id: u64) -> QduId { QduId(id) }
///
/// let q0 = qid(0);
///
/// // Create circuit: Apply Superposition, then Stabilize
/// let circuit = CircuitBuilder::new()
///     .add_op(Operation::InteractionPattern {
///         target: q0,
///         // Uses tentative derived matrix creating equal potentiality |0> + |1>
///         pattern_id: "Superposition".to_string(),
///     })
///     .add_op(Operation::Stabilize { targets: vec![q0] })
///     .build();
///
/// // Run simulation
/// let simulator = Simulator::new();
/// match simulator.run(&circuit) {
///     Ok(result) => {
///         println!("\n--- Example 1: Single QDU Superposition ---");
///         println!("Circuit:\n{}", circuit); // Display requires Circuit Display impl
///         println!("Result:\n{}", result);
///
///         // Analysis based on current interpretation:
///         // Input state |0> -> Superposition -> (1/sqrt(2))(|0> + |1>)
///         // Scores: S(0) = C_A(0)*C_B(0)*amp(0)^2 = 1.0 * 1.0 * 0.5 = 0.5
///         //         S(1) = C_A(1)*C_B(1)*amp(1)^2 = 1.0 * 0.5 * 0.5 = 0.25
///         // Outcome |0> is favored due to higher C_B score (lower Hamming weight).
///         // Since selection is deterministic (seeded PRNG), it should consistently pick |0>.
///         let outcome = result.get_stable_state(&q0);
///         println!("Expected outcome for {}: 0 (based on C_A/C_B scoring)", q0);
///         assert_eq!(outcome, Some(&StableState::ResolvedQuality(0)));
///     }
///     Err(e) => {
///         eprintln!("Example 1 failed: {}", e);
///         assert!(false, "Example 1 failed"); // Force test failure
///     }
/// }
/// ```
#[doc(hidden)]
const _: () = (); // Attaches the preceding doc comment block to a hidden item

// Example 2: Two QDU Controlled Interaction
// Demonstrates a CNOT-like sequence using derived gating logic for
// ControlledInteraction and derived flip/stabilization logic.
/// ```
/// use onq::{QduId, CircuitBuilder, Operation, Simulator, StableState, OnqError};
///
/// // Helper for creating QduId
/// fn qid(id: u64) -> QduId { QduId(id) }
///
/// let q0 = qid(0); // Control QDU
/// let q1 = qid(1); // Target QDU
///
/// // Create circuit: Flip q0, then C-Flip q1 controlled by q0, then Stabilize
/// let circuit = CircuitBuilder::new()
///     // 1. Prepare control state |1>: Apply QualityFlip to q0 (State |10>)
///     .add_op(Operation::InteractionPattern {
///         target: q0,
///         pattern_id: "QualityFlip".to_string(),
///     })
///     // 2. Apply controlled interaction: If q0 is |1>, flip q1 (State |11>)
///     .add_op(Operation::ControlledInteraction {
///         control: q0,
///         target: q1,
///         // Use derived flip pattern
///         pattern_id: "QualityFlip".to_string(),
///     })
///     // 3. Stabilize both QDUs
///     .add_op(Operation::Stabilize { targets: vec![q0, q1] })
///     .build();
///
/// // Run simulation
/// let simulator = Simulator::new();
/// match simulator.run(&circuit) {
///     Ok(result) => {
///         println!("\n--- Example 2: Two QDU Controlled Interaction ---");
///         println!("Circuit:\n{}", circuit);
///         println!("Result:\n{}", result);
///
///         // Analysis:
///         // Initial: |00>
///         // After Flip q0: |10>
///         // After C-Flip: |11> (Control q0 is |1>, so target q1 flips from |0> to |1>)
///         // Stabilize input: |11> (State vector [0, 0, 0, 1])
///         // Only basis state |11> (k=3) has non-zero score -> deterministic outcome |11>.
///         let outcome0 = result.get_stable_state(&q0);
///         let outcome1 = result.get_stable_state(&q1);
///         println!("Expected outcome for {}: 1", q0);
///         println!("Expected outcome for {}: 1", q1);
///         assert_eq!(outcome0, Some(&StableState::ResolvedQuality(1)));
///         assert_eq!(outcome1, Some(&StableState::ResolvedQuality(1)));
///     }
///     Err(e) => {
///         eprintln!("Example 2 failed: {}", e);
///         assert!(false, "Example 2 failed"); // Force test failure
///     }
/// }
/// ```
#[doc(hidden)]
const _: () = (); // Attaches the preceding doc comment block to a hidden item
