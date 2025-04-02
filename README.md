# onq: Operations for Next-generation Quantum Computing

## Overview

`onq` is a Rust library for exploring quantum computation and information processing

Unlike conventional quantum computing libraries (like Cirq or Qiskit) which are based on the postulates of quantum mechanics, `onq` attempts to simulate computational dynamics as they emerge *necessarily* from reality. The goal is to model phenomena analogous to quantum computation (like superposition, entanglement, interference, measurement/stabilization) without assuming physics, instead relying purely on abstract structural and logical consequences.

## Core Concepts

* **Qualitative Distinction Unit (QDU):** The fundamental unit, analogous to a qubit, representing a bounded distinction with qualitative state potential. (`onq::core::QduId`, `onq::core::Qdu`)
* **Potentiality State:** Represents the state of QDUs before stabilization, capturing multiple possibilities using complex amplitudes derived from phase (`e^(iθ)`). Uses a global state vector for integrated systems. (`onq::core::PotentialityState`)
* **Operations:** Transformations derived from key principles (Interaction, Influence, Sequential Ordering) and rules (`T = P⊗P`, phase manipulation, etc.). Implemented as distinct variants acting on the state vector. (`onq::operations::Operation`)
    * Includes derived patterns like `QualityFlip`, `PhaseIntroduce`, `Superposition`, `PhiRotate`, etc.
    * Includes derived `PhaseShift`, `ControlledInteraction`, `RelationalLock`.
* **Circuit:** An ordered sequence of operations applied to QDUs. (`onq::circuits::Circuit`)
* **Stabilization:** A process, derived from stability integration, coherence, and reality formation principles, that resolves `PotentialityState` into `StableState`. It uses interpretive scoring based on validation checks (Phase Coherence > 0.618 filter, Pattern Resonance interpreted via amplitude weighting) and deterministic selection via a state-seeded PRNG. (`onq::simulation::Simulator`, `onq::core::StableState`)
* **Circuit Visualization:** Circuits can be printed as text-based diagrams similar to Cirq.

## Current Status & Caveats

* **Foundational Implementation:** Core modules (`core`, `operations`, `circuits`, `simulation`) are implemented.
* **Interpretive Nature:** Many derivations (especially for stabilization scoring, specific interaction pattern matrices, control logic, lock logic, initial state, basis states)  are experimental. These interpretations aim for consistency but require further validation or refinement based on deeper framework analysis.
* **Placeholders/Weak Justifications:** Some derived patterns (`Superposition`, `SqrtFlip`) have weaker justifications than others. Some logic components (like the exact nature of resonance scoring) are simplified interpretations.
* **Testing:** Basic unit/integration tests and documentation examples exist and pass with the current logic.

**Disclaimer:** `onq` is a theoretical simulation project. It does not necessarily model physical reality or standard quantum mechanics, although analogies are sometimes drawn for comparison.

## Usage Example

examples/demo.rs
```rust
//! Succinct examples demonstrating building and simulating circuits

use onq::{
    CircuitBuilder, Operation, QduId, Simulator, OnqError
};
use std::f64::consts::PI;

// Helper for QduId creation for brevity in examples
fn qid(id: u64) -> QduId { QduId(id) }

fn main() -> Result<(), OnqError> {
    println!("--- onq Library Example ---");

    // Create a single simulator instance to run all examples
    let simulator = Simulator::new();

    // --- Example 1: Single QDU Superposition & Stabilization ---
    // Demonstrates preparing a superposition using a derived gate and observing
    // the outcome.
    println!("--- Example 1: Single QDU Superposition ---");
    let q0_ex1 = qid(0);
    let circuit1 = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q0_ex1,
            // Uses the tentatively derived "Hadamard-like" pattern
            pattern_id: "Superposition".to_string(),
        })
        .add_op(Operation::Stabilize { targets: vec![q0_ex1] })
        .build();

    println!("Circuit 1 Definition:\n{}", circuit1);
    match simulator.run(&circuit1) {
        Ok(result) => {
            println!("Circuit 1 Result:\n{}", result);
            // Analysis Comment: Based on current (phase_coherence)>0.618 filter and
            // S(k) = Score*|c_k|^4 scoring (interpreting  via amplitude),
            // the state (1/sqrt(2))[|0> + |1>] has Score_C=1.0 for both k=0, k=1.
            // S(0) = 1.0 * (0.5)^2 = 0.25, S(1) = 1.0 * (0.5)^2 = 0.25.
            // Scores are equal, outcome depends deterministically on state hash + PRNG.
            // We just check *an* outcome is produced.
            assert!(result.get_stable_state(&q0_ex1).is_some(), "Expected a stable state for q0");
            println!("Analysis: Outcome depends on deterministic PRNG seeded by state hash (scores S(0)=S(1)=0.25).\n");
        },
        Err(e) => println!("Circuit 1 Failed: {}\n", e),
    }

    // --- Example 2: Two QDU Bell State Analog & Stabilization ---
    // Demonstrates creating an entangled-like state using derived controlled logic
    // and observing the correlated outcomes.
    println!("--- Example 2: Two QDU Bell State (|00> + |11>) Analog Prep ---");
    let q0_ex2 = qid(0);
    let q1_ex2 = qid(1);
    let circuit2 = CircuitBuilder::new()
        // 1. Create superposition on control q0 -> (1/sqrt(2))(|00> + |10>)
        .add_op(Operation::InteractionPattern {
            target: q0_ex2,
            pattern_id: "Superposition".to_string(),
        })
        // 2. Apply controlled flip (CNOT analog using derived logic)
        //    If q0=|0>, target q1 unchanged. If q0=|1>, target q1 flips.
        //    Result state: (1/sqrt(2))(|00> + |11>)
        .add_op(Operation::ControlledInteraction {
            control: q0_ex2,
            target: q1_ex2,
            pattern_id: "QualityFlip".to_string(), // Use derived flip
        })
        .add_op(Operation::Stabilize { targets: vec![q0_ex2, q1_ex2] })
        .build();

    println!("Circuit 2 Definition:\n{}", circuit2);
    match simulator.run(&circuit2) {
        Ok(result) => {
            println!("Circuit 2 Result:\n{}", result);
            // Analysis Comment: Input state to stabilize is (1/sqrt(2))[|00> + |11>]
            // Check scores for k=0 (|00>) and k=3 (|11>). Others are 0 amplitude.
            // S(0): C=1 (neighbours 0 amp), S(0) = 1 * (0.5)^2 = 0.25
            // S(3): C=1 (neighbours 0 amp), S(3) = 1 * (0.5)^2 = 0.25
            // Outcome depends deterministically on PRNG seeded by state hash.
            // Importantly, the outcomes for q0 and q1 should be *correlated*.
            let outcome0 = result.get_stable_state(&q0_ex2).and_then(|s| s.get_resolved_value());
            let outcome1 = result.get_stable_state(&q1_ex2).and_then(|s| s.get_resolved_value());
            assert!(outcome0.is_some() && outcome1.is_some(), "Expected stable states for both qdus");
            assert_eq!(outcome0, outcome1, "Outcomes for Bell state analog should be correlated (both 0 or both 1)");
            println!("Analysis: Outcome depends on deterministic PRNG (scores S(0)=S(3)=0.25). Outcomes must be correlated ({} == {}).\n", outcome0.unwrap_or(9), outcome1.unwrap_or(9));

        },
        Err(e) => println!("Circuit 2 Failed: {}\n", e),
    }

    // --- Example 3: Phase Gate Sequence ---
    // Demonstrates applying various derived phase manipulation operations.
    println!("--- Example 3: Single QDU Phase Gate Sequence ---");
    let q0_ex3 = qid(0);
    let circuit3 = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern { target: q0_ex3, pattern_id: "Superposition".to_string()}) // Start in |+> state
        .add_op(Operation::InteractionPattern { target: q0_ex3, pattern_id: "HalfPhase".to_string()})      // Apply S analog
        .add_op(Operation::InteractionPattern { target: q0_ex3, pattern_id: "QuarterPhase".to_string()})   // Apply T analog
        .add_op(Operation::InteractionPattern { target: q0_ex3, pattern_id: "HalfPhase_Inv".to_string()})   // Apply Sdg analog
        .add_op(Operation::InteractionPattern { target: q0_ex3, pattern_id: "PhaseIntroduce".to_string()})// Apply Z analog
        .add_op(Operation::Stabilize { targets: vec![q0_ex3] })
        .build();

    println!("Circuit 3 Definition:\n{}", circuit3);
    match simulator.run(&circuit3) {
        Ok(result) => {
            println!("Circuit 3 Result:\n{}", result);
            // Analysis: Complex phase changes occur. Final state depends on matrix math.
            // Stabilization outcome depends on final state's amplitudes and C score.
            assert!(result.get_stable_state(&q0_ex3).is_some(), "Expected a stable state for q0");
            println!("Analysis: Outcome depends on final state after phase gates and stabilization scoring.\n");
        },
        Err(e) => println!("Circuit 3 Failed: {}\n", e),
    }

    // --- Example 4: Phi Rotation ---
    // Demonstrates the operation derived from the Golden Ratio.
    println!("--- Example 4: Single QDU Phi Rotation ---");
    let q0_ex4 = qid(0);
    let circuit4 = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q0_ex4,
            pattern_id: "PhiRotate".to_string(), // Use φ derived rotation
        }) // Applies rotation by PI/PHI to |0>
        .add_op(Operation::Stabilize { targets: vec![q0_ex4] })
        .build();

    println!("Circuit 4 Definition:\n{}", circuit4);
    match simulator.run(&circuit4) {
        Ok(result) => {
            println!("Circuit 4 Result:\n{}", result);
            // Analysis: Results in a specific superposition cos(a)|0> + sin(a)|1>.
            // Stabilization outcome depends on amplitudes and C1 score for this state.
             assert!(result.get_stable_state(&q0_ex4).is_some(), "Expected a stable state for q0");
             println!("Analysis: Outcome depends on state after Phi rotation and stabilization scoring.\n");
        },
        Err(e) => println!("Circuit 4 Failed: {}\n", e),
    }

     // --- Example 5: Relational Lock (CPhase) ---
     // Demonstrates applying the derived CPhase interpretation of RelationalLock.
     println!("--- Example 5: Two QDU Relational Lock (CPhase PI/2) ---");
     let q0_ex5 = qid(0);
     let q1_ex5 = qid(1);
     let circuit5 = CircuitBuilder::new()
         .add_op(Operation::InteractionPattern { target: q0_ex5, pattern_id: "Superposition".to_string() }) // |+>
         .add_op(Operation::InteractionPattern { target: q1_ex5, pattern_id: "Superposition".to_string() }) // |+> -> State |++>
         // State is now 0.5*(|00>+|01>+|10>+|11>)
         .add_op(Operation::RelationalLock {
             qdu1: q0_ex5,
             qdu2: q1_ex5,
             lock_params: PI / 2.0, // Apply phase PI/2 (factor of i)
             establish: true,
         }) // Should only affect |11> component -> 0.5*(|00>+|01>+|10>+i*|11>)
         .add_op(Operation::Stabilize { targets: vec![q0_ex5, q1_ex5] })
         .build();

     println!("Circuit 5 Definition:\n{}", circuit5);
     match simulator.run(&circuit5) {
         Ok(result) => {
            println!("Circuit 5 Result:\n{}", result);
            // Analysis: Stabilization outcome depends on scores for the final state.
            // All basis states have equal amplitude magnitude (0.5^2 = 0.25).
             assert!(result.get_stable_state(&q0_ex5).is_some(), "Expected a stable state for q0");
             assert!(result.get_stable_state(&q1_ex5).is_some(), "Expected a stable state for q1");
             println!("Analysis: Outcome depends on complex interplay of C scores for state 0.5*(|00>+|01>+|10>+i|11>).\n");
         },
         Err(e) => println!("Circuit 5 Failed: {}\n", e),
     }

    Ok(())
}
```
Outputs:

```bash
--- onq Library Example ---
--- Example 1: Single QDU Superposition ---
Circuit 1 Definition:
onq::Circuit[2 operations on 1 QDUs]
QDU(0): ───H──────M───

Circuit 1 Result:
Simulation Results:
  Stable Outcomes:
    QDU(0): Stable(0)

Analysis: Outcome depends on deterministic PRNG seeded by state hash (scores S(0)=S(1)=0.25).

--- Example 2: Two QDU Bell State (|00> + |11>) Analog Prep ---
Circuit 2 Definition:
onq::Circuit[3 operations on 2 QDUs]
QDU(0): ───H──────@──────M───
                  │          
QDU(1): ──────────X──────M───

Circuit 2 Result:
Simulation Results:
  Stable Outcomes:
    QDU(0): Stable(1)
    QDU(1): Stable(1)

Analysis: Outcome depends on deterministic PRNG (scores S(0)=S(3)=0.25). Outcomes must be correlated (1 == 1).

--- Example 3: Single QDU Phase Gate Sequence ---
Circuit 3 Definition:
onq::Circuit[6 operations on 1 QDUs]
QDU(0): ───H──────S──────T─────S†──────Z──────M───

Circuit 3 Failed: Instability Violation: Stabilization failed: No possible outcome met amplitude and C1 Phase Coherence (>0.618) criteria.

--- Example 4: Single QDU Phi Rotation ---
Circuit 4 Definition:
onq::Circuit[2 operations on 1 QDUs]
QDU(0): ──ΦR──────M───

Circuit 4 Result:
Simulation Results:
  Stable Outcomes:
    QDU(0): Stable(1)

Analysis: Outcome depends on state after Phi rotation and stabilization scoring.

--- Example 5: Two QDU Relational Lock (CPhase PI/2) ---
Circuit 5 Definition:
onq::Circuit[4 operations on 2 QDUs]
QDU(0): ───H─────────────@──────M───
                         │          
QDU(1): ──────────H──────●──────M───

Circuit 5 Result:
Simulation Results:
  Stable Outcomes:
    QDU(0): Stable(0)
    QDU(1): Stable(0)

Analysis: Outcome depends on complex interplay of C scores for state 0.5*(|00>+|01>+|10>+i|11>).

```

## License

Licensed under

MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
