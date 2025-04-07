// tests/vm_tests.rs

use onq::core::{OnqError, QduId, StableState};
use onq::operations::Operation;
use onq::vm::{Instruction, Program, ProgramBuilder, OnqVm}; // Import VM components

// Helper for QduId creation
fn qid(id: u64) -> QduId {
    QduId(id)
}

#[test]
fn test_vm_classical_loop() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Test: ONQ-VM Classical Loop ---");
    // Program: Counts from 0 up to (but not including) 5 in register "count"
    let program = ProgramBuilder::new()
        .add(Instruction::LoadImmediate { register: "count".to_string(), value: 0 })
        .add(Instruction::LoadImmediate { register: "limit".to_string(), value: 5 })
        .add(Instruction::Label("loop_start".to_string()))
        // Condition check: is count == limit?
        .add(Instruction::CmpEq {
            r_dest: "cond".to_string(),
            r_src1: "count".to_string(),
            r_src2: "limit".to_string(),
        })
        // If cond is NOT zero (i.e., count == limit), branch to end
        // Need BranchIfNotZero, let's use BranchIfZero inverted logic for now:
        // BranchIfZero checks if cond == 0 (i.e. count != limit). Jump *past* the end jump if true.
        .add(Instruction::BranchIfZero { register: "cond".to_string(), label: "continue_loop".to_string() })
        // If cond was 1 (count == limit), we don't branch, so jump to end
        .add(Instruction::Jump("loop_end".to_string()))
        .add(Instruction::Label("continue_loop".to_string()))
        // Increment count
        .add(Instruction::Addi {
            r_dest: "count".to_string(),
            r_src: "count".to_string(),
            value: 1,
        })
        // Jump back to loop start
        .add(Instruction::Jump("loop_start".to_string()))
        .add(Instruction::Label("loop_end".to_string()))
        .add(Instruction::Halt)
        .build()?; // Use ? to propagate potential build error

    println!("Program:\n{}", program);

    let mut vm = OnqVm::new();
    vm.run(&program)?; // Use ? to propagate potential run error

    println!("Final Classical Memory: {:?}", vm.get_classical_memory());

    assert_eq!(vm.get_classical_register("count"), 5, "Counter should reach 5");
    assert_eq!(vm.get_classical_register("limit"), 5, "Limit should be 5");
    assert_eq!(vm.get_classical_register("cond"), 1, "Final condition (count==limit) should be true (1)");

    Ok(())
}

#[test]
fn test_vm_conditional_quantum() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Test: ONQ-VM Conditional Quantum Op ---");
    // Program: Stabilize |> state of q0. If result is 0, flip q1. Check q1 state.
    // Expected (based on prior tests): Stabilizing |> yields 0. Flip *should* occur.
    let program = ProgramBuilder::new()
        // Prepare |> state on q0
        .add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: qid(0),
            pattern_id: "Superposition".to_string(),
        }))
        // Stabilize q0
        .add(Instruction::Stabilize { targets: vec![qid(0)] })
        // Record outcome in register "m0"
        .add(Instruction::Record { qdu: qid(0), register: "m0".to_string() })
        // Branch to "apply_flip" label IF "m0" is Zero
        .add(Instruction::BranchIfZero { register: "m0".to_string(), label: "apply_flip".to_string() })
        // If "m0" was non-zero (i.e., 1), skip the flip by jumping past it
        .add(Instruction::Jump("after_flip".to_string()))
        // --- Apply Flip Block ---
        .add(Instruction::Label("apply_flip".to_string()))
        .add(Instruction::QuantumOp(Operation::InteractionPattern {
            target: qid(1), // Target q1
            pattern_id: "QualityFlip".to_string(),
        }))
        // --- End of Apply Flip Block ---
        .add(Instruction::Label("after_flip".to_string()))
        // Stabilize q1 to see the result
        .add(Instruction::Stabilize { targets: vec![qid(1)] })
        // Record q1's outcome in register "m1"
        .add(Instruction::Record { qdu: qid(1), register: "m1".to_string() })
        .add(Instruction::Halt)
        .build()?;

    println!("Program:\n{}", program);

    let mut vm = OnqVm::new();
    vm.run(&program)?;

    let final_mem = vm.get_classical_memory();
    println!("Final Classical Memory: {:?}", final_mem);

    let m0 = vm.get_classical_register("m0");
    let m1 = vm.get_classical_register("m1");

    // Assert the core logic: The final state of q1 should be the opposite of the measurement outcome of q0.
    // If m0 = 0, branch was taken, flip applied -> m1 = 1.
    // If m0 = 1, branch not taken, flip skipped -> m1 = 0.
    assert_ne!(m0, m1, "m0 and m1 should have opposite values due to conditional flip. m0={}, m1={}", m0, m1);

    Ok(())
}

// Add more tests later:
// - Test other classical ops (And, Or, Xor, CmpGt etc.)
// - Test loops involving quantum state preparation/stabilization inside
// - Test error handling (e.g., undefined labels, invalid record target)
