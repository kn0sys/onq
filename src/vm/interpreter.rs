// src/vm/interpreter.rs

//! Defines the ONQ Virtual Machine (ONQ-VM) interpreter.

use super::program::{Instruction, Program}; // Use super to access sibling module
use crate::core::{QduId, StableState, PotentialityState, OnqError};
use crate::operations::Operation;
use crate::simulation::engine::SimulationEngine; // Use pub(crate) engine
use crate::simulation::SimulationResult; // Needed temporarily for stabilize call
use std::collections::{HashMap, HashSet};
use std::fmt;

/// The ONQ Virtual Machine (ONQ-VM).
///
/// Interprets and executes `onq::vm::Program` instructions, managing both
/// quantum state evolution (via `SimulationEngine`) and classical state (registers).
#[derive(Debug)]
pub struct OnqVm {
    /// The underlying UFF/Omniscript simulation engine. Initialized during `run`.
    engine: Option<SimulationEngine>,
    /// Named classical registers holding u64 values.
    classical_memory: HashMap<String, u64>,
    /// Stores the outcomes from the most recently executed `Stabilize` instruction.
    /// Keyed by QduId, maps to the resolved StableState value (0 or 1).
    last_stabilization_outcomes: HashMap<QduId, u64>,
    /// Program Counter: index of the next instruction to execute.
    program_counter: usize,
    /// Flag indicating if the VM has halted.
    is_halted: bool,
    // Potential future fields: cycle count, error state details, configuration
}

impl OnqVm {
    /// Creates a new, uninitialized ONQ-VM.
    pub fn new() -> Self {
        Self {
            engine: None,
            classical_memory: HashMap::new(),
            last_stabilization_outcomes: HashMap::new(),
            program_counter: 0,
            is_halted: false,
        }
    }

    /// Resets the VM state (PC, halted flag, memory, engine) for a new run.
    fn reset(&mut self) {
        self.engine = None; // Engine needs re-initialization based on program QDUs
        self.classical_memory.clear();
        self.last_stabilization_outcomes.clear();
        self.program_counter = 0;
        self.is_halted = false;
    }

    /// Runs a given `Program` until it halts or encounters an error.
    ///
    /// # Arguments
    /// * `program` - The `Program` to execute.
    ///
    /// # Returns
    /// * `Ok(())` if the program halts successfully.
    /// * `Err(OnqError)` if a simulation error or runtime error occurs (e.g., label not found, invalid op).
    pub fn run(&mut self, program: &Program) -> Result<(), OnqError> {
        self.reset();

        // 1. Determine all QDUs involved in the program to initialize the engine
        let all_qdus = Self::collect_qdus(program)?;
        if !all_qdus.is_empty() {
             self.engine = Some(SimulationEngine::init(&all_qdus)?);
        } else {
             // No quantum ops? Engine remains None. Allow running purely classical programs.
             self.engine = None;
        }


        // 2. Execution Loop
        while !self.is_halted {
            let pc = self.program_counter;

            // Fetch instruction
            let instruction = program.get_instruction(pc).ok_or_else(|| {
                OnqError::SimulationError{ message: format!("Program Counter ({}) out of bounds (0..{}).", pc, program.instruction_count())}
            })?;

            // Advance PC before execution (simplifies branching)
            self.program_counter += 1;

            // Execute instruction
            match instruction {
                Instruction::QuantumOp(op) => {
                    if let Some(engine) = self.engine.as_mut() {
                         engine.apply_operation(op)?;
                    } else {
                         return Err(OnqError::InvalidOperation { message: "Cannot execute QuantumOp: SimulationEngine not initialized (no QDUs defined in program?).".to_string() });
                    }
                }
                Instruction::Stabilize { targets } => {
                    if targets.is_empty() { continue; } // No-op if no targets
                    if let Some(engine) = self.engine.as_mut() {
                         // Need a temporary SimulationResult structure just to call stabilize
                         // Alternatively, modify stabilize to return HashMap<QduId, StableState> directly?
                         // Let's use the temporary result structure for now.
                         let mut temp_result = SimulationResult::new();
                         engine.stabilize(targets, &mut temp_result)?;
                         // Store the u64 outcomes for Record instruction
                         self.last_stabilization_outcomes = temp_result.all_stable_outcomes().iter()
                             .filter_map(|(qid, state)| state.get_resolved_value().map(|val| (*qid, val)))
                             .collect();
                    } else {
                         return Err(OnqError::InvalidOperation { message: "Cannot execute Stabilize: SimulationEngine not initialized.".to_string() });
                    }
                }
                Instruction::Record { qdu, register } => {
                    let value = self.last_stabilization_outcomes.get(qdu).ok_or_else(|| {
                        OnqError::InvalidOperation { message: format!("Cannot Record: QDU {} was not found in the last stabilization results.", qdu) }
                    })?;
                    self.classical_memory.insert(register.clone(), *value);
                }
                Instruction::Label(_) => {
                    // No operation, labels handled during build/jump resolution
                }
                Instruction::Jump(label) => {
                    let target_pc = program.get_label_pc(label).ok_or_else(|| {
                         OnqError::SimulationError { message: format!("Runtime Error: Jump target label '{}' not found.", label) }
                    })?;
                    self.program_counter = target_pc; // Set PC to instruction *after* label
                }
                Instruction::BranchIfZero { register, label } => {
                    let reg_value = self.classical_memory.get(register).copied().unwrap_or(0); // Default to 0 if register doesn't exist
                    if reg_value == 0 {
                        let target_pc = program.get_label_pc(label).ok_or_else(|| {
                             OnqError::SimulationError { message: format!("Runtime Error: Branch target label '{}' not found.", label) }
                        })?;
                        self.program_counter = target_pc;
                    }
                    // Otherwise, continue to next instruction (PC already incremented)
                }
                Instruction::LoadImmediate { register, value } => {
                    self.classical_memory.insert(register.clone(), *value);
                }
                Instruction::Copy { source_reg, dest_reg } => {
                    let value = self.classical_memory.get(source_reg).copied().unwrap_or(0); // Default to 0 if source doesn't exist
                    self.classical_memory.insert(dest_reg.clone(), value);
                }
                Instruction::Halt => {
                    self.is_halted = true;
                    // Optionally break loop here, or let loop condition handle it
                }
                Instruction::NoOp => {
                    // Do nothing
                }
            } // End match instruction

             // Check if PC ran off the end without halting
            if !self.is_halted && self.program_counter >= program.instruction_count() {
                 // Implicit halt at end of program? Or error? Let's halt.
                 self.is_halted = true;
            }

        } // End while !self.is_halted

        Ok(())
    }

    /// Collects all unique QDU IDs mentioned in a program.
    fn collect_qdus(program: &Program) -> Result<HashSet<QduId>, OnqError> {
         let mut qdus = HashSet::new();
         for instruction in &program.instructions {
             match instruction {
                 Instruction::QuantumOp(op) => {
                     qdus.extend(op.involved_qdus());
                 }
                 Instruction::Stabilize { targets } => {
                     qdus.extend(targets);
                 }
                 Instruction::Record { qdu, .. } => {
                     qdus.insert(*qdu);
                 }
                 // Classical/Control flow ops don't directly involve QDUs
                 _ => {}
             }
         }
         Ok(qdus)
    }


    /// Reads the value of a classical register after a run.
    /// Returns 0 if the register does not exist.
    pub fn get_classical_register(&self, name: &str) -> u64 {
        self.classical_memory.get(name).copied().unwrap_or(0)
    }

    /// Returns a clone of the entire classical memory map.
    pub fn get_classical_memory(&self) -> HashMap<String, u64> {
        self.classical_memory.clone()
    }

    // Potential future methods:
    // - step(): Execute one instruction
    // - get_potentiality_state(): Get a clone of the engine's state (if engine exists)
    // - set_initial_state(...): Allow starting from non-|0...0> state
    // - inject_error(...): For noise simulation
}

// Default implementation
impl Default for OnqVm {
    fn default() -> Self {
        Self::new()
    }
}
