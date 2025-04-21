# onq: Operations for Next Generation Quantum Computing

[![crates.io](https://img.shields.io/crates/v/onq.svg)](https://crates.io/crates/onq) [![docs.rs](https://docs.rs/onq/badge.svg)](https://docs.rs/onq) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) 

`onq` is a Rust library for simulating quantum computation and information processing derived **strictly from the theoretical first principles**.

Unlike conventional quantum computing simulators based on quantum mechanics postulates, `onq` explores how computational dynamics might emerge necessarily from a single axiom. It aims to model phenomena analogous to quantum computation (superposition, entanglement analogs, interference, stabilization/measurement analogs) **without assuming physics**, relying instead on the structural and logical consequences defined by the framework provided during its development context.

This library serves as a tool for:
* Exploring the computational implications of the framework.
* Comparing the resulting dynamics with standard quantum mechanics to understand how foundational assumptions shape computation.
* Investigating alternative models of computation derived from abstract principles.

## Core Concepts

Key concepts derived and implemented in `onq`:

* **Qualitative Distinction Unit (QDU):** (`onq::core::QduId`, struct `onq::core::Qdu` implicitly managed by VM/Engine) The fundamental unit, analogous to a qubit. Represents a necessary, bounded distinction with inherent qualitative properties. Interpreted as having a minimal binary basis {Quality0, Quality1}.
* **Potentiality State:** (`onq::core::PotentialityState`) Represents the state before stabilization. Uses a complex state vector (`Vec<Complex<f64>>`) to capture multiple potentialities and phase relationships (inspired by `e^(iθ)`). For N QDUs, uses a 2<sup>N</sup> dimensional vector to model integrated states. Assumes initial state `|Q0...Q0>`.
* **Operations:** (`onq::operations::Operation`) Transformations derived from interaction principles (`T=P⊗P` analog). Implemented as variants acting on the state vector, including:
    * Single-QDU `InteractionPattern`s with derived matrices (analogs for I, X, Y, Z, H, S, T, S†, T†, √X, √X†, plus φ-based rotations).
    * General `PhaseShift`.
    * `ControlledInteraction` (using derived conditional gating logic via 4x4 matrix).
    * `RelationalLock` (using derived non-unitary projection onto Bell states).
* **ONQ Virtual Machine (ONQ-VM):** (`onq::vm::OnqVm`) An interpreter that executes `Program`s containing sequences of `Instruction`s.
* **Program / Instructions:** (`onq::vm::Program`, `onq::vm::Instruction`, `onq::vm::ProgramBuilder`) Defines programs with mixed quantum operations, stabilization, classical memory access (`Record`), classical computation (arithmetic, logic, compare), and control flow (labels, jumps, branches).
* **Stabilization:** (`onq::vm::Instruction::Stabilize`, internal `SimulationEngine::stabilize`) The  analog of measurement. Deterministically resolves `PotentialityState` into `StableState`. See "Key Differences" below.
* **Validation:** (`onq::validation::*`) Functions to check state normalization and  Phase Coherence criteria.

## Key Differences from Quantum Mechanics

Understanding `onq` requires recognizing its fundamental departures from standard quantum mechanics (QM):

1.  **Foundation:** `onq` derives from abstract logical necessity, not QM postulates derived from physical observation.
2.  **"Measurement" (Stabilization):** This is the most significant difference.
    * **QM:** Projective measurement is probabilistic (Born rule), collapsing the state vector onto an eigenstate, inherently random for superpositions.
    * **`onq`:** `Stabilize` is **deterministic**. It resolves potentiality based on criteria:
        * **Filtering:** Potential outcomes (basis states `|k>`) *must* meet an interpreted Phase Coherence threshold (`Score_C1(k) > 0.618`) relative to the input state. States failing this cannot be stabilized into.
        * **Scoring:** Valid outcomes are weighted by `S(k) = Score_C1(k) * |amplitude_k|^2` (interpreting resonance/convergence via amplitude).
        * **Selection:** A single outcome `k` is chosen deterministically from the valid, scored possibilities using a pseudo-random number generator seeded by a hash of the input state vector. (Same input state -> Same outcome).
    * **Consequence:** Running the same `onq` program yields the same stabilization results every time. Statistical distributions seen in QM experiments must emerge differently, if at all (perhaps through variations in initial state preparation or environmental interaction, which are not yet modeled).
3.  **Operations:** While many operations have QM analogs (H, X, Z, CNOT...), their existence and specific matrix forms are justified solely by interpretation. Unique operations like `PhiRotate` may exist, and standard QM operations might be absent if not derivable.
4.  **"Entanglement" (Locking):** `RelationalLock` uses non-unitary projection to force the state into specific integrated subspaces (Bell states currently), directly modeling Integration/Coherence. This differs from QM where entanglement arises purely from unitary evolution (e.g., CNOT on superposition).

## Interpretations, Assumptions, and Limitations

**This library is heavily based on interpretation** due to the abstract nature and mathematical ambiguities within the source texts provided. Key assumptions include:

* **QDU Basis:** Assumes a minimal binary {Quality0, Quality1} basis per QDU.
* **Initial State:** Assumes a default `|Q0...Q0>` state.
* **State Vector Model:** Uses a standard complex state vector and interprets operations as matrix multiplications (interpreting `T=P⊗P`).
* **Stabilization Scoring:** The metrics used for C1 (neighbour phase diff avg cosine) and C3 (via amplitude `|c_k|^2` weighting) are interpretations. The application of the C1 `>0.618` threshold as a filter on outcomes is also an interpretation.
* **Control Logic:** `ControlledInteraction` assumes simple gating based on control quality.
* **Locking:** `RelationalLock` assumes non-unitary projection is a valid mechanism for achieving integrated states.
* **Mathematical Gaps:** The library cannot fully resolve ambiguities stemming from undefined operators (`⊗`, `×`, `∇²`, etc.) and terms (φ, ψ in `Ω(x)`).

**Therefore, simulation results reflect the behavior of *this specific interpretation* of the  framework.**

## Current Status

* Core simulation engine for state vector evolution based on derived operations.
* ONQ-VM interpreter supporting mixed quantum/classical programs with control flow.
* stabilization mechanism implemented.
* State validation checks integrated.
* Circuit visualization via `Display` trait.
* Basic tests and examples demonstrating functionality.
* **Ongoing:** Refinement of interpretations, derivation of more operations, addressing framework ambiguities.

## Usage Example (Quantum Teleportation Analog)

```rust
// From examples/vm_teleportation.rs (showing structure)
use onq::{
    core::{OnqError, QduId},
    operations::Operation,
    vm::{Instruction, LockType, OnqVm, ProgramBuilder}, // Use LockType if needed elsewhere
};

fn qid(id: u64) -> QduId { QduId(id) }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let msg_q = qid(0);
    let alice_q = qid(1);
    let bob_q = qid(2);

    let program = ProgramBuilder::new()
        // 1. Prep Message |+>
        .add(Instruction::QuantumOp(Operation::InteractionPattern { target: msg_q, pattern_id: "Superposition".to_string() }))
        // 2. Create Bell Pair |Φ+> (Alice, Bob)
        .add(Instruction::QuantumOp(Operation::InteractionPattern { target: alice_q, pattern_id: "Superposition".to_string() }))
        .add(Instruction::QuantumOp(Operation::ControlledInteraction { control: alice_q, target: bob_q, pattern_id: "QualityFlip".to_string() }))
        // 3. Bell Basis Change (Msg, Alice)
        .add(Instruction::QuantumOp(Operation::ControlledInteraction { control: msg_q, target: alice_q, pattern_id: "QualityFlip".to_string() }))
        .add(Instruction::QuantumOp(Operation::InteractionPattern { target: msg_q, pattern_id: "Superposition".to_string() }))
        // 4. Stabilize & Record Alice's results
        .add(Instruction::Stabilize { targets: vec![msg_q, alice_q] })
        .add(Instruction::Record { qdu: msg_q, register: "m_msg".to_string() })
        .add(Instruction::Record { qdu: alice_q, register: "m_alice".to_string() })
        // 5. Bob's Conditional Corrections
        .add(Instruction::Label("X_Correction".to_string()))
        .add(Instruction::BranchIfZero { register: "m_alice".to_string(), label: "skip_x_corr".to_string() })
        .add(Instruction::QuantumOp(Operation::InteractionPattern { target: bob_q, pattern_id: "QualityFlip".to_string() }))
        .add(Instruction::Label("skip_x_corr".to_string()))
        .add(Instruction::Label("Z_Correction".to_string()))
        .add(Instruction::BranchIfZero { register: "m_msg".to_string(), label: "skip_z_corr".to_string() })
        .add(Instruction::QuantumOp(Operation::InteractionPattern { target: bob_q, pattern_id: "PhaseIntroduce".to_string() }))
        .add(Instruction::Label("skip_z_corr".to_string()))
        // 6. Stabilize Bob & Record
        .add(Instruction::Stabilize { targets: vec![bob_q] })
        .add(Instruction::Record { qdu: bob_q, register: "m_bob".to_string() })
        // 7. Halt
        .add(Instruction::Halt)
        .build()?;

    println!("Program:\n{}", program); // Uses Program Display impl

    let mut vm = OnqVm::new();
    vm.run(&program)?;

    println!("\nFinal Classical Memory: {:?}", vm.get_classical_memory());
    Ok(())
}
```

## Development

```bash
# Clone the repository (if applicable)
# git clone ...
# cd onq

# Build
cargo build [--release]

# Run tests (unit, integration, doc)
cargo test [--release]

# Run examples
cargo run --example sqrt_flip_demo [--release]
cargo run --example vm_teleportation [--release]
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
