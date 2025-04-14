//! Example: Grover's Search Analog (2 QDU) using the ONQ-VM.
//! Demonstrates building Oracle and Diffusion operators from derived gates,
//! classical loop control (for potential future scaling), and stabilization.

use onq::core::QduId;
use onq::operations::Operation;
use onq::vm::{Instruction, OnqVm, ProgramBuilder};

// Helper for QduId creation
fn qid(id: u64) -> QduId {
    QduId(id)
}

// Helper to add the Oracle operation (CZ marking |11>)
fn add_oracle(builder: ProgramBuilder, q0: QduId, q1: QduId) -> ProgramBuilder {
    builder.pb_add(Instruction::QuantumOp(Operation::ControlledInteraction {
        control: q0, // Order doesn't matter for CZ
        target: q1,
        pattern_id: "PhaseIntroduce".to_string(), // Z analog -> CZ
    }))
}

// Helper to add the Diffusion operator (H⊗H · X⊗X · CZ · X⊗X · H⊗H)
fn add_diffusion(builder: ProgramBuilder, q0: QduId, q1: QduId) -> ProgramBuilder {
    builder
        // H⊗H
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q0, pattern_id: "Superposition".to_string() }))
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q1, pattern_id: "Superposition".to_string() }))
        // X⊗X
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q0, pattern_id: "QualityFlip".to_string() }))
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q1, pattern_id: "QualityFlip".to_string() }))
        // CZ
        .pb_add(Instruction::QuantumOp(Operation::ControlledInteraction { control: q0, target: q1, pattern_id: "PhaseIntroduce".to_string() }))
        // X⊗X
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q0, pattern_id: "QualityFlip".to_string() }))
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q1, pattern_id: "QualityFlip".to_string() }))
        // H⊗H
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q0, pattern_id: "Superposition".to_string() }))
        .pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q1, pattern_id: "Superposition".to_string() }))
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- ONQ-VM Example: Grover's Search Analog (2 QDU - Find |11>) ---");

    let q0 = qid(0);
    let q1 = qid(1);

    // Optimal iterations for N=4 states (2 QDUs) is approx (π/4)√N = (π/4)√4 = π/2 ≈ 1.57.
    // So, 1 iteration should be nearly optimal. We'll structure for a loop anyway.
    let num_iterations = 1;

    // --- Build the Grover Program ---
    let mut builder = ProgramBuilder::new();

    // Initialize classical loop counter
    builder = builder.pb_add(Instruction::LoadImmediate { register: "k".to_string(), value: 0 });
    builder = builder.pb_add(Instruction::LoadImmediate { register: "limit".to_string(), value: num_iterations });

    // 1. Prepare superposition state |++>
    builder = builder.pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q0, pattern_id: "Superposition".to_string() }));
    builder = builder.pb_add(Instruction::QuantumOp(Operation::InteractionPattern { target: q1, pattern_id: "Superposition".to_string() }));

    // --- Grover Iteration Loop ---
    builder = builder.pb_add(Instruction::Label("grover_loop".to_string()));

    // 2. Apply Oracle (marks |11> with phase -1)
    builder = add_oracle(builder, q0, q1);

    // 3. Apply Diffusion Operator (amplifies marked state)
    builder = add_diffusion(builder, q0, q1);

    // 4. Loop control
    builder = builder.pb_add(Instruction::Addi { r_dest: "k".to_string(), r_src: "k".to_string(), value: 1 });
    builder = builder.pb_add(Instruction::CmpLt { // Check if k < limit
        r_dest: "cond".to_string(),
        r_src1: "k".to_string(),
        r_src2: "limit".to_string(),
    });
    builder = builder.pb_add(Instruction::BranchIfZero { // If cond=0 (k >= limit), jump to end
        register: "cond".to_string(),
        label: "grover_end".to_string(),
    });
    builder = builder.pb_add(Instruction::Jump("grover_loop".to_string())); // If cond=1 (k < limit), loop again

    // --- End of Loop ---
    builder = builder.pb_add(Instruction::Label("grover_end".to_string()));

    // 5. Stabilize and Record results
    builder = builder.pb_add(Instruction::Stabilize { targets: vec![q0, q1] });
    builder = builder.pb_add(Instruction::Record { qdu: q0, register: "m0".to_string() });
    builder = builder.pb_add(Instruction::Record { qdu: q1, register: "m1".to_string() });

    // 6. Halt
    builder = builder.pb_add(Instruction::Halt);

    // Build final program
    let program = builder.build()?;

    println!("\nGrover Program Instructions (1 iteration):\n{}", program);

    // --- Run the ONQ-VM ---
    let mut vm = OnqVm::new();
    println!("Running ONQ-VM...");
    vm.run(&program)?;

    // --- Analyze Results ---
    println!("\n--- ONQ-VM Execution Finished ---");
    let final_mem = vm.get_classical_memory();
    println!("Final Classical Memory: {:?}", final_mem);

    let m0 = vm.get_classical_register("m0");
    let m1 = vm.get_classical_register("m1");

    println!("\nAnalysis:");
    println!("- Target state for search was |11>");
    println!("- After {} Grover iteration(s), stabilized state was |{}{}>", num_iterations, m0, m1);

    // For 1 iteration on 2 qubits, Grover strongly amplifies the marked state |11>.
    // Our stabilization should heavily favor this outcome.
    assert_eq!(m0, 1, "Expected outcome for q0 to be 1");
    assert_eq!(m1, 1, "Expected outcome for q1 to be 1");
    println!("- Success! Stabilized state matches the marked state |11>.");

    Ok(())
}
