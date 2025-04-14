//! Example: Quantum Teleportation using the ONQ-VM.
//! Demonstrates preparing an entangled pair, Bell measurement,
//! recording results to classical registers, and applying conditional
//! recovery operations based on the classical results.

use onq::core::QduId;
use onq::operations::Operation;
use onq::vm::{Instruction, OnqVm, ProgramBuilder};
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
