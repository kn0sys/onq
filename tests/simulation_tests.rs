// tests/simulation_tests.rs

// Import necessary types from the onq crate
use onq::{
    Circuit, CircuitBuilder, OnqError, Operation, QduId, simulation::Simulator, simulation::SimulationResult, StableState,
};

use std::f64::consts::PI;

// Helper function to create QduId for tests
fn qid(id: u64) -> QduId {
    QduId(id)
}

// Helper function to check the stable state of a QDU in the result
fn check_stable_state(result: &SimulationResult, qdu_id: QduId, expected_state_val: u64) {
    match result.get_stable_state(&qdu_id) {
        Some(StableState::ResolvedQuality(val)) => {
            assert_eq!(*val, expected_state_val, "Mismatch for QDU {}", qdu_id);
        }
        _ => panic!("QDU {} was not stabilized or result is not ResolvedQuality", qdu_id),
    }
}

#[test]
fn test_empty_circuit() -> Result<(), OnqError> {
    let circuit = Circuit::new();
    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    assert!(result.all_stable_outcomes().is_empty(), "Empty circuit should yield empty results");
    Ok(())
}

#[test]
fn test_initial_state_stabilization() -> Result<(), OnqError> {
    // Test stabilizing the default |0> state for one QDU
    let q0 = qid(0);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::Stabilize { targets: vec![q0] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    assert_eq!(result.all_stable_outcomes().len(), 1, "Should have one result");
    check_stable_state(&result, q0, 0); // Expect |0...0> state -> outcome 0
    Ok(())
}

#[test]
fn test_two_qdus_initial_state_stabilization() -> Result<(), OnqError> {
    // Test stabilizing the default |00> state for two QDUs
    let q0 = qid(0);
    let q1 = qid(1);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::Stabilize { targets: vec![q0, q1] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    assert_eq!(result.all_stable_outcomes().len(), 2, "Should have two results");
    check_stable_state(&result, q0, 0); // Expect |0>
    check_stable_state(&result, q1, 0); // Expect |0>
    Ok(())
}


#[test]
fn test_identity_operation() -> Result<(), OnqError> {
    // Apply placeholder Identity and stabilize
    let q0 = qid(0);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q0,
            pattern_id: "Identity".to_string(), // Use placeholder Identity
        })
        .add_op(Operation::Stabilize { targets: vec![q0] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    check_stable_state(&result, q0, 0); // Identity shouldn't change outcome from |0>
    Ok(())
}

#[test]
fn test_quality_flip_operation() -> Result<(), OnqError> {
    // Apply placeholder QualityFlip (like X gate) and stabilize
    let q0 = qid(0);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q0,
            pattern_id: "QualityFlip".to_string(), // Use placeholder Flip
        })
        .add_op(Operation::Stabilize { targets: vec![q0] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    // Started in |0>, flipped to |1>, stabilized outcome should be 1
    check_stable_state(&result, q0, 1);
    Ok(())
}

#[test]
fn test_phase_shift_operation() -> Result<(), OnqError> {
    // Apply PhaseShift and stabilize. Placeholder stabilization ignores phase.
    let q0 = qid(0);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::PhaseShift { target: q0, theta: PI / 2.0 }) // 90 degree phase shift
        .add_op(Operation::Stabilize { targets: vec![q0] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    // Placeholder stabilization only looks at amplitude magnitude, should still be outcome 0
    check_stable_state(&result, q0, 0);
    Ok(())
}

#[test]
fn test_two_qdus_flip_one() -> Result<(), OnqError> {
    // Flip q1, leave q0 as |0>. State |01>
    let q0 = qid(0);
    let q1 = qid(1);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q1, // Flip only q1
            pattern_id: "QualityFlip".to_string(),
        })
        .add_op(Operation::Stabilize { targets: vec![q0, q1] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    assert_eq!(result.all_stable_outcomes().len(), 2);
    check_stable_state(&result, q0, 0); // q0 should be 0
    check_stable_state(&result, q1, 1); // q1 should be 1
    Ok(())
}

#[test]
fn test_controlled_interaction_placeholder_control0() -> Result<(), OnqError> {
    // Test ControlledInteraction where control is |0> (initial state)
    let q0 = qid(0); // Control
    let q1 = qid(1); // Target
    let circuit = CircuitBuilder::new()
        .add_op(Operation::ControlledInteraction {
            control: q0,
            target: q1,
            pattern_id: "QualityFlip".to_string(), // Attempt to flip target if control is 1
        })
        .add_op(Operation::Stabilize { targets: vec![q0, q1] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    assert_eq!(result.all_stable_outcomes().len(), 2);
    // Control q0 started and stayed as |0>
    check_stable_state(&result, q0, 0);
    // Target q1 started as |0> and should NOT have flipped
    check_stable_state(&result, q1, 0);
    Ok(())
}

#[test]
fn test_controlled_interaction_placeholder_control1() -> Result<(), OnqError> {
    // Test ControlledInteraction where control is |1>
    let q0 = qid(0); // Control
    let q1 = qid(1); // Target
    let circuit = CircuitBuilder::new()
        // 1. Flip control q0 to |1>
        .add_op(Operation::InteractionPattern {
            target: q0,
            pattern_id: "QualityFlip".to_string(),
        })
        // 2. Apply controlled interaction
        .add_op(Operation::ControlledInteraction {
            control: q0,
            target: q1,
            pattern_id: "QualityFlip".to_string(), // Attempt to flip target
        })
        // 3. Stabilize
        .add_op(Operation::Stabilize { targets: vec![q0, q1] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit)?;

    assert_eq!(result.all_stable_outcomes().len(), 2);
    // Control q0 started as |0>, flipped to |1>
    check_stable_state(&result, q0, 1);
    // Target q1 started as |0>, should HAVE flipped because control was |1>
    check_stable_state(&result, q1, 1);
    Ok(())
}

#[test]
fn test_undefined_interaction_pattern() {
    // Test that using an undefined pattern ID results in an error
    let q0 = qid(0);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::InteractionPattern {
            target: q0,
            pattern_id: "ThisPatternDoesNotExist".to_string(), // Undefined ID
        })
        .add_op(Operation::Stabilize { targets: vec![q0] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit);

    assert!(result.is_err(), "Expected an error for undefined pattern ID");
    match result.err().unwrap() {
        OnqError::InvalidOperation { message } => {
            assert!(message.contains("Interaction Pattern 'ThisPatternDoesNotExist' is not defined"), "Incorrect error message: {}", message);
        }
        e => panic!("Expected InvalidOperation error, got {:?}", e),
    }
}

#[test]
fn test_unimplemented_relational_lock() {
     // Test that RelationalLock currently causes an issue (prints warning or errors)
    let q0 = qid(0);
    let q1 = qid(1);
    let circuit = CircuitBuilder::new()
        .add_op(Operation::RelationalLock { // Using placeholder op
            qdu1: q0,
            qdu2: q1,
            lock_params: 0.0, // Placeholder
            establish: true,
        })
        .add_op(Operation::Stabilize { targets: vec![q0, q1] })
        .build();

    let simulator = Simulator::new();
    let result = simulator.run(&circuit);

    // Depending on whether the engine ignores with warning or returns error:
    // Option 1: If it ignores, the simulation should succeed but maybe print warning.
    // Test that it succeeds and stabilization gives default results.
     assert!(result.is_ok(), "Simulation should succeed even if RelationalLock is ignored (currently)");
     if let Ok(res) = result {
        check_stable_state(&res, q0, 0); // Expect default outcome if lock ignored
        check_stable_state(&res, q1, 0);
     }

    // Option 2: If it returns an error (better):
    // assert!(result.is_err(), "Expected an error for unimplemented RelationalLock");
    // match result.err().unwrap() {
    //     OnqError::InvalidOperation { message } => {
    //          assert!(message.contains("RelationalLock simulation not yet implemented"), "Incorrect error message: {}", message);
    //     }
    //     e => panic!("Expected InvalidOperation error, got {:?}", e),
    // }
}
