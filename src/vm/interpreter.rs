// src/vm/interpreter.rs

//! Defines the ONQ Virtual Machine (ONQ-VM) interpreter.

use super::program::{Instruction, Program}; // Use super to access sibling module
use crate::core::{QduId, OnqError};
use crate::simulation::engine::SimulationEngine; // Use pub(crate) engine
use crate::simulation::SimulationResult; // Needed temporarily for stabilize call
use std::collections::{HashMap, HashSet};

/// The ONQ Virtual Machine (ONQ-VM).
///
/// Interprets and executes [`Program`](super::program::Program) instructions,
/// managing both quantum state evolution (via an internal [`SimulationEngine`])
/// and classical state (registers stored in a `HashMap`). It enables mixed
/// classical/quantum algorithms with control flow based on intermediate
/// stabilization results.
///
/// # Examples
///
/// ```
/// # use onq::{ProgramBuilder, Instruction, Operation, QduId, vm::OnqVm, OnqError};
/// # fn qid(id: u64) -> QduId { QduId(id) }
/// // Simple program: H(0), Stabilize(0), Record(0, "m") -> Halt
/// let program = ProgramBuilder::new()
///     .pb_add(Instruction::QuantumOp(Operation::InteractionPattern {
///         target: qid(0),
///         pattern_id: "Superposition".to_string()
///     }))
///     .pb_add(Instruction::Stabilize { targets: vec![qid(0)] })
///     .pb_add(Instruction::Record { qdu: qid(0), register: "m".to_string() })
///     .pb_add(Instruction::Halt)
///     .build()
///     .expect("Failed to build program");
///
/// let mut vm = OnqVm::new();
/// match vm.run(&program) {
///     Ok(()) => {
///         let measurement_result = vm.get_classical_register("m");
///         println!("Stabilization outcome for QDU(0): {}", measurement_result);
///         // Outcome (0 or 1) is deterministic for this VM run.
///         assert!(measurement_result == 0 || measurement_result == 1);
///     }
///     Err(e) => eprintln!("VM run failed: {}", e),
/// }
/// ```
#[derive(Debug)]
pub struct OnqVm {
    /// The underlying simulation engine. Initialized during `run`.
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
        println!("[VM RUN START]"); // DEBUG

        // 1. Determine all QDUs involved...
        let all_qdus = Self::collect_qdus(program)?;
        if !all_qdus.is_empty() {
             self.engine = Some(SimulationEngine::init(&all_qdus)?);
             println!("[VM Engine Initialized for {:?}]", all_qdus); // DEBUG
        } else {
             self.engine = None;
             println!("[VM Engine Not Needed (No QDUs)]"); // DEBUG
        }


        // 2. Execution Loop
        let mut executed_instruction_count = 0; // DEBUG loop counter
        const MAX_INSTRUCTIONS: u64 = 1000; // DEBUG limit

        while !self.is_halted {
             // --- DEBUG: Safety break for infinite loops ---
             if executed_instruction_count > MAX_INSTRUCTIONS {
                  return Err(OnqError::SimulationError{ message: format!("Execution exceeded maximum instruction limit ({}) - potential infinite loop?", MAX_INSTRUCTIONS)});
             }
             executed_instruction_count += 1;
             // --- End DEBUG ---


            let pc = self.program_counter;

            // Fetch instruction
             println!("[VM] PC={:04} Fetching...", pc); // DEBUG
            let instruction = program.get_instruction(pc).ok_or_else(|| {
                OnqError::SimulationError{ message: format!("Program Counter ({}) out of bounds (0..{}).", pc, program.instruction_count())}
            })?;
             println!("[VM] PC={:04} Executing: {:?}", pc, instruction); // DEBUG

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
                    if targets.is_empty() {
                        println!("[VM] PC={:04} Stabilize: No targets.", pc); // DEBUG
                        continue;
                    }
                    if let Some(engine) = self.engine.as_mut() {
                         let mut temp_result = SimulationResult::new();
                         println!("[VM] PC={:04} Calling engine.stabilize for {:?}", pc, targets); // DEBUG
                         engine.stabilize(targets, &mut temp_result)?; // This might return Err
                         println!("[VM] PC={:04} engine.stabilize finished. Temp result: {:?}", pc, temp_result); // DEBUG

                         // Store the u64 outcomes for Record instruction
                         self.last_stabilization_outcomes = temp_result.all_stable_outcomes().iter()
                             .filter_map(|(qid, state)| {
                                 // DEBUG: See what get_resolved_value returns
                                 let resolved = state.get_resolved_value();
                                 println!("[VM] PC={:04} Stabilize: QDU {}, State {:?}, Resolved Value: {:?}", pc, qid, state, resolved); // DEBUG
                                 resolved.map(|val| (*qid, val))
                             })
                             .collect();
                         println!("[VM] PC={:04} Stored last_stabilization_outcomes: {:?}", pc, self.last_stabilization_outcomes); // DEBUG
                    } else {
                         return Err(OnqError::InvalidOperation { message: "Cannot execute Stabilize: SimulationEngine not initialized.".to_string() });
                    }
                }
                Instruction::Record { qdu, register } => {
                    println!("[VM] PC={:04} Attempting to record for QDU {}", pc, qdu); // DEBUG
                    println!("[VM] PC={:04} Current last_stabilization_outcomes: {:?}", pc, self.last_stabilization_outcomes); // DEBUG
                    // Attempt to get the value
                    let value_option = self.last_stabilization_outcomes.get(qdu);
                    println!("[VM] PC={:04} Value Option for QDU {}: {:?}", pc, qdu, value_option); // DEBUG

                    let value = value_option.ok_or_else(|| {
                        OnqError::InvalidOperation { message: format!("Cannot Record: QDU {} was not found in the last stabilization results ({:?}). Was Stabilize called immediately prior with this QDU?", qdu, self.last_stabilization_outcomes) }
                    })?;
                    println!("[VM] PC={:04} Recording value {} to register '{}'", pc, value, register); // DEBUG
                    self.classical_memory.insert(register.clone(), *value);
                    println!("[VM] PC={:04} Classical memory now: {:?}", pc, self.classical_memory); // DEBUG
                }
                Instruction::Label(_) => {
                     println!("[VM] PC={:04} Encountered Label (No-Op)", pc); // DEBUG
                    // No operation, labels handled during build/jump resolution
                }
                Instruction::Jump(label) => {
                    let target_pc = program.get_label_pc(label).ok_or_else(|| {
                         OnqError::SimulationError { message: format!("Runtime Error: Jump target label '{}' not found.", label) }
                    })?;
                    println!("[VM] PC={:04} Jumping to label '{}' (PC={})", pc, label, target_pc); // DEBUG
                    self.program_counter = target_pc; // Set PC to target instruction index
                }
                Instruction::BranchIfZero { register, label } => {
                    let reg_value = self.classical_memory.get(register).copied().unwrap_or(0); // Default to 0
                    println!("[VM] PC={:04} BranchIfZero: Reg '{}' = {}", pc, register, reg_value); // DEBUG
                    if reg_value == 0 {
                        let target_pc = program.get_label_pc(label).ok_or_else(|| {
                             OnqError::SimulationError { message: format!("Runtime Error: Branch target label '{}' not found.", label) }
                        })?;
                        println!("[VM] PC={:04} Branch taken to label '{}' (PC={})", pc, label, target_pc); // DEBUG
                        self.program_counter = target_pc;
                    } else {
                         println!("[VM] PC={:04} Branch not taken.", pc); // DEBUG
                    }
                     // If branch not taken, PC remains incremented from before match
                }
                Instruction::LoadImmediate { register, value } => {
                    println!("[VM] PC={:04} LoadImm: Reg '{}' = {}", pc, register, value); // DEBUG
                    self.classical_memory.insert(register.clone(), *value);
                }
                Instruction::Copy { source_reg, dest_reg } => {
                    let value = self.classical_memory.get(source_reg).copied().unwrap_or(0);
                     println!("[VM] PC={:04} Copy: Reg '{}' = {} from Reg '{}'", pc, dest_reg, value, source_reg); // DEBUG
                    self.classical_memory.insert(dest_reg.clone(), value);
                }
                Instruction::OnqAdd { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), val1.wrapping_add(val2));
                }
                Instruction::Addi { r_dest, r_src, value } => {
                    let val_src = self.classical_memory.get(r_src).copied().unwrap_or(0);
                    let result = val_src.wrapping_add(*value);
                     println!("[VM] PC={:04} Addi: Reg '{}' = {} + {} = {}", pc, r_dest, val_src, value, result); // DEBUG
                    self.classical_memory.insert(r_dest.clone(), result);
                }
                Instruction::Sub { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), val1.wrapping_sub(val2));
                }
                Instruction::Mul { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), val1.wrapping_mul(val2));
                 }
                Instruction::OnqNot { r_dest, r_src } => {
                    let val_src = self.classical_memory.get(r_src).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), !val_src); // Bitwise NOT
                }
                Instruction::And { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), val1 & val2); // Bitwise AND
                }
                 Instruction::Or { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), val1 | val2); // Bitwise OR
                }
                 Instruction::Xor { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), val1 ^ val2); // Bitwise XOR
                }
                Instruction::CmpEq { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    let result = if val1 == val2 { 1 } else { 0 };
                     println!("[VM] PC={:04} CmpEq: Reg '{}' = ({} == {}) = {}", pc, r_dest, val1, val2, result); // DEBUG
                    self.classical_memory.insert(r_dest.clone(), result);
                }
                Instruction::CmpLt { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), if val1 < val2 { 1 } else { 0 });
                }
                // Add similar println! for other classical ops if needed
                Instruction::Halt => {
                     println!("[VM] PC={:04} Halting.", pc); // DEBUG
                    self.is_halted = true;
                }
                Instruction::NoOp => {
                     println!("[VM] PC={:04} NoOp.", pc); // DEBUG
                    // Do nothing
                }
                Instruction::CmpGt { r_dest, r_src1, r_src2 } => {
                    let val1 = self.classical_memory.get(r_src1).copied().unwrap_or(0);
                    let val2 = self.classical_memory.get(r_src2).copied().unwrap_or(0);
                    self.classical_memory.insert(r_dest.clone(), if val1 > val2 { 1 } else { 0 });
                }
            } // End match instruction

            // Check if PC ran off the end without halting
            if !self.is_halted && self.program_counter >= program.instruction_count() {
                 println!("[VM] PC={} Reached end of program. Halting.", self.program_counter); // DEBUG
                 self.is_halted = true;
            }

        } // End while !self.is_halted

        println!("[VM RUN END]"); // DEBUG
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

    /// Returns a clone of the current quantum PotentialityState, if the
    /// simulation engine has been initialized (i.e., if the program contained quantum ops).
    /// Returns `None` if no quantum state exists (e.g., purely classical program or before run).
    /// Cloning can be expensive for large state vectors.
    pub fn get_final_state(&self) -> Option<crate::PotentialityState> {
        // Access the engine's state via the pub(crate) get_state method and clone it
        self.engine.as_ref().map(|e| e.get_state().clone())
        // Note: PotentialityState derives Clone, which uses Vec::clone, performing a deep copy.
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
