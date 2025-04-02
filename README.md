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

```rust
// From examples/demo.rs

use onq::{
    CircuitBuilder, Operation, QduId, Simulator, OnqError
};

// Helper for QduId creation
fn qid(id: u64) -> QduId { QduId(id) }

fn run_example() -> Result<(), OnqError> {
    println!("--- Example: Bell State Analog Prep ---");
    let q0 = qid(0);
    let q1 = qid(1);
    let circuit = CircuitBuilder::new()
        // 1. Create superposition on control q0
        .add_op(Operation::InteractionPattern {
            target: q0,
            pattern_id: "Superposition".to_string(),
        })
        // 2. Apply controlled flip (CNOT analog)
        .add_op(Operation::ControlledInteraction {
            control: q0,
            target: q1,
            pattern_id: "QualityFlip".to_string(), // Use derived flip
        })
        // 3. Stabilize (Measure)
        .add_op(Operation::Stabilize { targets: vec![q0, q1] })
        .build();

    // Print the circuit diagram
    println!("Circuit Definition:\n{}", circuit);

    // Run the simulation
    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    // Print the results
    println!("Simulation Result:\n{}", result);

    // Example assertion (outcomes depend on deterministic stabilization)
    let outcome0 = result.get_stable_state(&q0).and_then(|s| s.get_resolved_value());
    let outcome1 = result.get_stable_state(&q1).and_then(|s| s.get_resolved_value());
    assert!(outcome0.is_some() && outcome1.is_some());
    assert_eq!(outcome0, outcome1, "Outcomes should be correlated");

    Ok(())
}

// main function would call run_example()
// fn main() { run_example().unwrap(); }
