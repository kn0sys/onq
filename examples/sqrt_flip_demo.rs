//! Example demonstrating SqrtFlip and based Stabilization.
//! Compares conceptual approach to standard quantum measurement experiments.

use onq::{
    CircuitBuilder, Operation, QduId, Simulator, OnqError,
};

// Helper for QduId creation
fn qid(id: u64) -> QduId { QduId(id) }

fn main() -> Result<(), OnqError> {
    println!("--- onq Example: SqrtFlip (√X Analog) & Stabilization ---");

    let q0 = qid(0);

    // --- Build Circuit ---
    // 1. Apply SqrtFlip to the initial |0> state.
    // 2. Stabilize the result.
    let circuit = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q0,
            pattern_id: "SqrtFlip".to_string(), // Use derived Sqrt(X) analog
        })
        .add_op(Operation::Stabilize { targets: vec![q0] })
        .build();

    // Print the circuit diagram
    println!("\nCircuit Definition:\n{}", circuit);

    // --- Analyze Expected Intermediate State (Conceptual) ---
    // Applying SqrtFlip (matrix 0.5*[[1+i, 1-i],[1-i, 1+i]]) to |0> ([1, 0] vector)
    // gives state vector: [0.5*(1+i), 0.5*(1-i)]
    // This corresponds to the state: ( (1+i)/2 )|0> + ( (1-i)/2 )|1>
    // Amplitudes squared: |c0|^2 = |(1+i)/2|^2 = 0.5, |c1|^2 = |(1-i)/2|^2 = 0.5
    println!("Conceptual State Before Stabilization: ~ [0.5+0.5i, 0.5-0.5i]");
    println!("  (Equal probability potentiality for |0> and |1> like standard √X gate)");

    // --- Run Simulation ---
    let simulator = Simulator::new();
    println!("\nRunning onq simulation (single deterministic run)...");

    match simulator.run(&circuit) {
        Ok(result) => {
            println!("Simulation finished successfully.");
            println!("\nSimulation Result Details:");
            println!("{}", result); // Uses the Display impl for SimulationResult

            // --- Analyze Result ---
            if let Some(stable_state) = result.get_stable_state(&q0) {
                 println!("\nFinal Stable State for {}: {}", q0, stable_state);

                 // Explanation of the difference from QM repeated measurement
                 println!("\n--- Comparison to Quantum Measurement ---");
                 println!("In standard Quantum Mechanics, measuring the state after sqrt(X)");
                 println!("yields |0> or |1> with 50% probability each. Repeating the experiment");
                 println!("builds up statistics (like Cirq's 'm=1111101010...').");
                 println!("\nIn this derived simulation ('onq'):");
                 println!("- Stabilization is a deterministic process based on rules.");
                 println!("- It filters outcomes by Phase Coherence (>0.618).");
                 println!("- It scores valid outcomes using S(k) = Score_C1 * |c_k|^2.");
                 println!("- It uses a state-seeded PRNG to pick *one* outcome if multiple are possible.");
                 println!("- Therefore, running this *same circuit* again will yield the *same* result: {}", stable_state);
                 // We could calculate the C1/S scores for k=0, k=1 for this state to predict
                 // which outcome is favored, but the exact result depends on the state hash + PRNG sequence.
            } else {
                 println!("\nQDU {} was not found in stabilization results.", q0);
                 return Err(OnqError::SimulationError{message: "Stabilization result missing".to_string()});
            }
        }
        Err(e) => {
            eprintln!("\n--- Simulation Failed ---");
            eprintln!("Error: {}", e);
            return Err(e); // Propagate error
        }
    }

    Ok(())
}
