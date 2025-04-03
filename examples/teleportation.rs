//! Succinct examples demonstrating building and simulating circuits

use onq::{
    Circuit, CircuitBuilder, Operation, QduId, Simulator, SimulationResult, StableState, OnqError
};
use std::f64::consts::PI;

// Helper for QduId creation for brevity in examples
fn qid(id: u64) -> QduId { QduId(id) }

fn main() {
    // Define the three QDUs involved
    let msg_q = qid(0);    // Message QDU (will be prepared in |+>)
    let alice_q = qid(1);  // Alice's QDU (part of the Bell pair)
    let bob_q = qid(2);    // Bob's QDU (part of the Bell pair, receives the state)

    println!("QDUs defined: msg={}, alice={}, bob={}", msg_q, alice_q, bob_q);

    // --- Build Circuit using Quantum Recovery Logic ---

    let mut builder = CircuitBuilder::new();
    println!("Building Teleportation Circuit...");

    // 1. Prepare Message State: Put msg_q in |+> state
    builder = builder.add_op(Operation::InteractionPattern {
        target: msg_q,
        pattern_id: "Superposition".to_string(), // H analog
    });
    println!("  Step 1: Prepared Message QDU in |+> state (using Superposition).");

    // 2. Create Bell Pair between Alice and Bob: |Φ+> = (1/sqrt(2))(|00> + |11>)
    builder = builder.add_op(Operation::InteractionPattern { // H on Alice
        target: alice_q,
        pattern_id: "Superposition".to_string(),
    });
    builder = builder.add_op(Operation::ControlledInteraction { // CNOT(Alice, Bob)
        control: alice_q,
        target: bob_q,
        pattern_id: "QualityFlip".to_string(), // X analog
    });
    println!("  Step 2: Created Bell Pair between Alice and Bob.");

    // 3. Alice performs Bell Measurement operations (basis change)
    builder = builder.add_op(Operation::ControlledInteraction { // CNOT(Message, Alice)
        control: msg_q,
        target: alice_q,
        pattern_id: "QualityFlip".to_string(),
    });
    builder = builder.add_op(Operation::InteractionPattern { // H on Message
        target: msg_q,
        pattern_id: "Superposition".to_string(),
    });
    println!("  Step 3: Applied Bell Measurement basis change gates (CNOT, H).");

    // 4. Quantum Recovery Operations (before stabilization)
    //    These apply corrections based on the state *before* stabilization.
    builder = builder.add_op(Operation::ControlledInteraction { // CNOT(Alice, Bob)
        control: alice_q,
        target: bob_q,
        pattern_id: "QualityFlip".to_string(),
    });
    builder = builder.add_op(Operation::ControlledInteraction { // CZ(Message, Bob) analog
        control: msg_q,
        target: bob_q,
        pattern_id: "PhaseIntroduce".to_string(), // Use derived Z analog pattern
    });
    println!("  Step 4: Applied Quantum Recovery gates (CNOT, CZ analog).");

    // 5. Stabilize Bob's QDU to observe the teleported state
    //    Optionally stabilize Alice and Message to see their final states too.
    builder = builder.add_op(Operation::Stabilize {
        targets: vec![msg_q, alice_q, bob_q],
    });
    println!("  Step 5: Added final stabilization for all QDUs.");

    let circuit = builder.build();

    // Print the constructed circuit diagram
    println!("\nQuantum Teleportation Circuit (Quantum Recovery):\n{}", circuit);

    // --- Run Simulation ---
    let simulator = Simulator::new();
    println!("\nRunning simulation...");

    match simulator.run(&circuit) {
        Ok(result) => {
            println!("Simulation finished successfully.");
            println!("\nSimulation Result Details:");
            println!("{}", result);

            // --- Basic Result Analysis ---
            // Ideally, Bob's QDU (bob_q) now holds the original state of msg_q (which was |+>).
            // Stabilizing the |+> state = (1/sqrt(2))[|0> + |1>] depends on the rules.
            // Based on our current stabilization (S(0)=0.25, S(1)=0.25), the outcome for Bob
            // will be deterministically 0 or 1 based on the final state hash and PRNG.
            // A full verification would require state vector tomography (not implemented)
            // or running statistical tests if stabilization were probabilistic.

            println!("\nAnalysis:");
            if let Some(StableState::ResolvedQuality(bob_outcome)) = result.get_stable_state(&bob_q) {
                println!("- Bob's QDU ({}) stabilized to state: {}", bob_q, bob_outcome);
                println!("  (Note: Expected pre-stabilization state was |+>, outcome {} depends on deterministic stabilization)", bob_outcome);
            } else {
                println!("- Bob's QDU ({}) was not found in stabilization results.", bob_q);
            }
            // Print Alice and Message outcomes too
            if let Some(StableState::ResolvedQuality(alice_outcome)) = result.get_stable_state(&alice_q) {
                println!("- Alice's QDU ({}) stabilized to state: {}", alice_q, alice_outcome);
            }
            if let Some(StableState::ResolvedQuality(msg_outcome)) = result.get_stable_state(&msg_q) {
                println!("- Message's QDU ({}) stabilized to state: {}", msg_q, msg_outcome);
                println!("  (These represent the classical bits Alice would send in standard protocol)");
            }
            println!("\nVerification of perfect state teleportation would require state vector analysis.");

        }
        Err(e) => {
            eprintln!("\n--- Simulation Failed ---");
            eprintln!("Error: {}", e);
        }
    }
}
