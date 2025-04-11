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
//! Example: Quantum Teleportation using the ONQ-VM.
//! Demonstrates preparing an entangled pair, Bell measurement,
//! recording results to classical registers, and applying conditional
//! recovery operations based on the classical results.

use onq::core::QduId;
use onq::operations::Operation;
use onq::vm::{Instruction, OnqVm, ProgramBuilder}; // Import VM components

// Helper for QduId creation
fn qid(id: u64) -> QduId {
    QduId(id)
}

fn main() -> Result<(), Box<dyn std::error::Error>> { // Use Box<dyn Error> for main
    println!("--- ONQ-VM Example: Quantum Teleportation (with Classical Control) ---");

    // Define QDUs
    let msg_q = qid(0);
    let alice_q = qid(1);
    let bob_q = qid(2);

    // --- Build the Teleportation Program ---
    let program = ProgramBuilder::new()
        // 1. Prepare Message state |+>
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: msg_q,
            pattern_id: "Superposition".to_string(),
        }))
        // 2. Create Bell pair |Φ+> between Alice and Bob
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: alice_q,
            pattern_id: "Superposition".to_string(),
        }))
        .pb_add(Instruction::QuantumOp(Operation::ControlledInteraction {
            control: alice_q,
            target: bob_q,
            pattern_id: "QualityFlip".to_string(),
        }))
        // 3. Alice's Bell measurement basis change
        .pb_add(Instruction::QuantumOp(Operation::ControlledInteraction {
            control: msg_q,
            target: alice_q,
            pattern_id: "QualityFlip".to_string(),
        }))
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: msg_q,
            pattern_id: "Superposition".to_string(),
        }))
        // 4. Stabilize Alice's qubits and Record results
        .pb_add(Instruction::Stabilize { targets: vec![msg_q, alice_q] })
        .pb_add(Instruction::Record { qdu: msg_q,   register: "m_msg".to_string() })
        .pb_add(Instruction::Record { qdu: alice_q, register: "m_alice".to_string() })

        // 5. Bob's Classical Corrections (Conditional Operations)
        // 5a. X Correction based on Alice's measurement (m_alice)
        .pb_add(Instruction::Label("X_Correction".to_string())) // Label for clarity
        .pb_add(Instruction::BranchIfZero { // If m_alice == 0, jump past X gate
            register: "m_alice".to_string(),
            label: "skip_x_corr".to_string(),
        })
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { // Apply X if m_alice == 1
            target: bob_q,
            pattern_id: "QualityFlip".to_string(),
        }))
        .pb_add(Instruction::Label("skip_x_corr".to_string()))

        // 5b. Z Correction based on Message measurement (m_msg)
        .pb_add(Instruction::Label("Z_Correction".to_string())) // Label for clarity
        .pb_add(Instruction::BranchIfZero { // If m_msg == 0, jump past Z gate
            register: "m_msg".to_string(),
            label: "skip_z_corr".to_string(),
        })
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { // Apply Z if m_msg == 1
            target: bob_q,
            pattern_id: "PhaseIntroduce".to_string(),
        }))
        .pb_add(Instruction::Label("skip_z_corr".to_string()))

        // 6. Stabilize Bob's qubit (optional, to verify outcome)
        .pb_add(Instruction::Stabilize { targets: vec![bob_q] })
        .pb_add(Instruction::Record { qdu: bob_q, register: "m_bob".to_string() })

        // 7. Halt
        .pb_add(Instruction::Halt)
        .build()?; // Build the program

    // Print the program instructions
    println!("\nTeleportation Program Instructions:\n{}", program);

    // --- Run the ONQ-VM ---
    let mut vm = OnqVm::new();
    println!("Running ONQ-VM...");
    vm.run(&program)?; // Execute the program

    // --- Analyze Results ---
    println!("\n--- ONQ-VM Execution Finished ---");
    let final_mem = vm.get_classical_memory();
    println!("Final Classical Memory: {:?}", final_mem);

    let m_msg = vm.get_classical_register("m_msg");
    let m_alice = vm.get_classical_register("m_alice");
    let m_bob = vm.get_classical_register("m_bob");

    println!("\nAnalysis:");
    println!("- Alice's measurement outcomes (classical bits sent): msg={}, alice={}", m_msg, m_alice);
    println!("- Bob's final stabilized state: {}", m_bob);
    println!("- Verification Notes:");
    println!("  - Input state for Message QDU was |+>.");
    println!("  - Teleportation *should* transfer |+> state to Bob's QDU *before* final stabilization.");
    println!("  - Stabilizing |+> state deterministically yields 0 or 1 based on rules (observed outcome: {}).", m_bob);
    println!("  - Perfect verification requires state vector access/tomography.");

    Ok(())
}
```
Outputs:

```bash
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/examples/vm_teleportation`
--- ONQ-VM Example: Quantum Teleportation (with Classical Control) ---

Teleportation Program Instructions:
ONQ-VM Program (15 instructions)
  0000: QuantumOp(InteractionPattern { target: QduId(0), pattern_id: "Superposition" })
  0001: QuantumOp(InteractionPattern { target: QduId(1), pattern_id: "Superposition" })
  0002: QuantumOp(ControlledInteraction { control: QduId(1), target: QduId(2), pattern_id: "QualityFlip" })
  0003: QuantumOp(ControlledInteraction { control: QduId(0), target: QduId(1), pattern_id: "QualityFlip" })
  0004: QuantumOp(InteractionPattern { target: QduId(0), pattern_id: "Superposition" })
  0005: Stabilize { targets: [QduId(0), QduId(1)] }
  0006: Record { qdu: QduId(0), register: "m_msg" }
  0007: Record { qdu: QduId(1), register: "m_alice" }
X_Correction:
  0008: BranchIfZero { register: "m_alice", label: "skip_x_corr" }
  0009: QuantumOp(InteractionPattern { target: QduId(2), pattern_id: "QualityFlip" })
Z_Correction:
  0010: BranchIfZero { register: "m_msg", label: "skip_z_corr" }
  0011: QuantumOp(InteractionPattern { target: QduId(2), pattern_id: "PhaseIntroduce" })
skip_z_corr:
  0012: Stabilize { targets: [QduId(2)] }
  0013: Record { qdu: QduId(2), register: "m_bob" }
  0014: Halt

Running ONQ-VM...
[VM RUN START]
[VM Engine Initialized for {QduId(1), QduId(2), QduId(0)}]
[VM] PC=0000 Fetching...
[VM] PC=0000 Executing: QuantumOp(InteractionPattern { target: QduId(0), pattern_id: "Superposition" })
[VM] PC=0001 Fetching...
[VM] PC=0001 Executing: QuantumOp(InteractionPattern { target: QduId(1), pattern_id: "Superposition" })
[VM] PC=0002 Fetching...
[VM] PC=0002 Executing: QuantumOp(ControlledInteraction { control: QduId(1), target: QduId(2), pattern_id: "QualityFlip" })
[VM] PC=0003 Fetching...
[VM] PC=0003 Executing: QuantumOp(ControlledInteraction { control: QduId(0), target: QduId(1), pattern_id: "QualityFlip" })
[VM] PC=0004 Fetching...
[VM] PC=0004 Executing: QuantumOp(InteractionPattern { target: QduId(0), pattern_id: "Superposition" })
[VM] PC=0005 Fetching...
[VM] PC=0005 Executing: Stabilize { targets: [QduId(0), QduId(1)] }
[VM] PC=0005 Calling engine.stabilize for [QduId(0), QduId(1)]
[VM] PC=0005 engine.stabilize finished. Temp result: SimulationResult { stable_outcomes: {QduId(0): ResolvedQuality(0), QduId(1): ResolvedQuality(1)} }
[VM] PC=0005 Stabilize: QDU QDU(0), State ResolvedQuality(0), Resolved Value: Some(0)
[VM] PC=0005 Stabilize: QDU QDU(1), State ResolvedQuality(1), Resolved Value: Some(1)
[VM] PC=0005 Stored last_stabilization_outcomes: {QduId(0): 0, QduId(1): 1}
[VM] PC=0006 Fetching...
[VM] PC=0006 Executing: Record { qdu: QduId(0), register: "m_msg" }
[VM] PC=0006 Attempting to record for QDU QDU(0)
[VM] PC=0006 Current last_stabilization_outcomes: {QduId(0): 0, QduId(1): 1}
[VM] PC=0006 Value Option for QDU QDU(0): Some(0)
[VM] PC=0006 Recording value 0 to register 'm_msg'
[VM] PC=0006 Classical memory now: {"m_msg": 0}
[VM] PC=0007 Fetching...
[VM] PC=0007 Executing: Record { qdu: QduId(1), register: "m_alice" }
[VM] PC=0007 Attempting to record for QDU QDU(1)
[VM] PC=0007 Current last_stabilization_outcomes: {QduId(0): 0, QduId(1): 1}
[VM] PC=0007 Value Option for QDU QDU(1): Some(1)
[VM] PC=0007 Recording value 1 to register 'm_alice'
[VM] PC=0007 Classical memory now: {"m_alice": 1, "m_msg": 0}
[VM] PC=0008 Fetching...
[VM] PC=0008 Executing: BranchIfZero { register: "m_alice", label: "skip_x_corr" }
[VM] PC=0008 BranchIfZero: Reg 'm_alice' = 1
[VM] PC=0008 Branch not taken.
[VM] PC=0009 Fetching...
[VM] PC=0009 Executing: QuantumOp(InteractionPattern { target: QduId(2), pattern_id: "QualityFlip" })
[VM] PC=0010 Fetching...
[VM] PC=0010 Executing: BranchIfZero { register: "m_msg", label: "skip_z_corr" }
[VM] PC=0010 BranchIfZero: Reg 'm_msg' = 0
[VM] PC=0010 Branch taken to label 'skip_z_corr' (PC=12)
[VM] PC=0012 Fetching...
[VM] PC=0012 Executing: Stabilize { targets: [QduId(2)] }
[VM] PC=0012 Calling engine.stabilize for [QduId(2)]
[VM] PC=0012 engine.stabilize finished. Temp result: SimulationResult { stable_outcomes: {QduId(2): ResolvedQuality(1)} }
[VM] PC=0012 Stabilize: QDU QDU(2), State ResolvedQuality(1), Resolved Value: Some(1)
[VM] PC=0012 Stored last_stabilization_outcomes: {QduId(2): 1}
[VM] PC=0013 Fetching...
[VM] PC=0013 Executing: Record { qdu: QduId(2), register: "m_bob" }
[VM] PC=0013 Attempting to record for QDU QDU(2)
[VM] PC=0013 Current last_stabilization_outcomes: {QduId(2): 1}
[VM] PC=0013 Value Option for QDU QDU(2): Some(1)
[VM] PC=0013 Recording value 1 to register 'm_bob'
[VM] PC=0013 Classical memory now: {"m_bob": 1, "m_alice": 1, "m_msg": 0}
[VM] PC=0014 Fetching...
[VM] PC=0014 Executing: Halt
[VM] PC=0014 Halting.
[VM RUN END]

--- ONQ-VM Execution Finished ---
Final Classical Memory: {"m_bob": 1, "m_alice": 1, "m_msg": 0}

Analysis:
- Alice's measurement outcomes (classical bits sent): msg=0, alice=1
- Bob's final stabilized state: 1
- Verification Notes:
  - Input state for Message QDU was |+>.
  - Teleportation *should* transfer |+> state to Bob's QDU *before* final stabilization.
  - Stabilizing |+> state deterministically yields 0 or 1 based on rules (observed outcome: 1).
  - Perfect verification requires state vector access/tomography.
```

## Notebooks

* install [jupyter-lab](https://jupyter.org/)
* install [evcxr](https://github.com/evcxr/evcxr/blob/main/evcxr_jupyter/README.md)
* see the `onq/notebooks` directory

## License

Licensed under

MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
