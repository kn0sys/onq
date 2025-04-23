//! Example: Bernstein-Vazirani Algorithm Analog using the ONQ-VM.
//! Determines a hidden bitstring 's' encoded in an oracle U_f|x>|y> = |x>|y ⊕ s.x>
//! in a single query by leveraging superposition and interference.

use onq::core::QduId;
use onq::operations::Operation;
use onq::vm::{Instruction, OnqVm, ProgramBuilder};

// Helper for QduId creation
fn qid(id: u64) -> QduId {
    QduId(id)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- ONQ-VM Example: Bernstein-Vazirani (n=3, s=101) ---");

    // --- Setup ---
    let n: usize = 3; // Number of bits in the secret string
    let secret_string = "101"; // The hidden string s (s_0=1, s_1=0, s_2=1)
    assert_eq!(secret_string.len(), n, "Secret string length must match n");

    // Define QDUs (n input + 1 output)
    let input_qdus: Vec<QduId> = (0..n).map(|i| qid(i as u64)).collect();
    let output_qdu = qid(n as u64); // qid(3)

    // --- Build the Bernstein-Vazirani Program ---
    let mut builder = ProgramBuilder::new();
    println!("Building Bernstein-Vazirani program...");

    // 1. Initialize output QDU qn to |-> = H(X|0>)
    builder = builder.pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
        target: output_qdu,
        pattern_id: "QualityFlip".to_string(), // X gate analog
    }));
    builder = builder.pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
        target: output_qdu,
        pattern_id: "Superposition".to_string(), // H gate analog
    }));
    println!("  Step 1: Initialized output QDU to |-> state.");

    // 2. Apply Hadamard analog to all input QDUs -> creates superposition of all |x>
    for q_in in &input_qdus {
        builder = builder.pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: *q_in,
            pattern_id: "Superposition".to_string(),
        }));
    }
    println!("  Step 2: Applied H to all input QDUs.");

    // 3. Apply the Oracle U_f implementing f(x) = s.x (mod 2)
    //    This is done by applying CNOT(q_i, q_out) for each i where s_i = 1.
    println!("  Step 3: Applying Oracle U_f for s={}...", secret_string);
    for (i, bit) in secret_string.chars().enumerate() {
        if bit == '1' {
            let control_qdu = input_qdus[i]; // q_i
            builder = builder.pb_add(Instruction::QuantumOp(Operation::ControlledInteraction {
                control: control_qdu,
                target: output_qdu,
                pattern_id: "QualityFlip".to_string(), // CNOT analog
            }));
            println!("    - Added CNOT({}, {}) for s_{}=1", control_qdu, output_qdu, i);
        }
    }

    // 4. Apply Hadamard analog to all input QDUs again
    for q_in in &input_qdus {
        builder = builder.pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: *q_in,
            pattern_id: "Superposition".to_string(),
        }));
    }
     println!("  Step 4: Applied H to all input QDUs again.");

    // 5. Stabilize and Record the input QDUs
    // The state of the input QDUs should now be |s>
    builder = builder.pb_add(Instruction::Stabilize { targets: input_qdus.clone() });
     println!("  Step 5: Added Stabilization for input QDUs.");
    for (i, q_in) in input_qdus.iter().enumerate() {
         let register_name = format!("m{}", i); // m0, m1, m2
         builder = builder.pb_add(Instruction::Record {
             qdu: *q_in,
             register: register_name,
         });
     }
     println!("  Step 6: Added Record instructions for input QDUs.");

    // 7. Halt
    builder = builder.pb_add(Instruction::Halt);

    // Build final program
    let program = builder.build()?;

    println!("\nBernstein-Vazirani Program Instructions:\n{}", program);

    // --- Run the ONQ-VM ---
    let mut vm = OnqVm::new();
    println!("Running ONQ-VM...");
    vm.run(&program)?;

    // --- Analyze Results ---
    println!("\n--- ONQ-VM Execution Finished ---");
    let final_mem = vm.get_classical_memory();
    println!("Final Classical Memory: {:?}", final_mem);

    println!("\nAnalysis:");
    let mut measured_string = String::new();
    for i in 0..n {
        let reg_name = format!("m{}", i);
        let bit = vm.get_classical_register(&reg_name);
        measured_string.push(if bit == 1 { '1' } else { '0' });
    }

    println!("- Secret string s = {}", secret_string);
    println!("- Measured string = {}", measured_string);

    assert_eq!(measured_string, secret_string, "Measured string should match the secret string!");
    println!("- Success! The measured state directly reveals the secret string.");

    Ok(())
}
