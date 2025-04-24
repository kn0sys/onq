// src/vm/mod.rs

//! Defines the structures and interpreter for the ONQ Virtual Machine (ONQ-VM).
//! Enables mixed classical/quantum computation based on ONQ principles.

use crate::core::QduId;
use crate::operations::Operation;
use std::collections::HashMap;
use std::fmt;

/// Specifies the target entangled state for a RelationalLock operation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)] // Eq/Hash useful if used as keys later
pub enum LockType {
    /// Target state: |Φ+> = (1/sqrt(2))(|00> + |11>)
    BellPhiPlus,
    /// Target state: |Φ-> = (1/sqrt(2))(|00> - |11>)
    BellPhiMinus,
    /// Target state: |Ψ+> = (1/sqrt(2))(|01> + |10>)
    BellPsiPlus,
    /// Target state: |Ψ-> = (1/sqrt(2))(|01> - |10>)
    BellPsiMinus,
}

// --- Instruction Set Definition ---

/// Represents a single instruction executable by the ONQ-VM.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // --- Quantum Operations ---
    /// Apply a standard quantum operation derived from ONQ.
    QuantumOp(Operation),

    // --- Stabilization & Classical Recording ---
    /// Perform ONQ stabilization on target QDUs. The result is held implicitly
    /// until potentially recorded by a subsequent `Record` instruction.
    Stabilize {
        /// The list of QDU IDs to stabilize.
        targets: Vec<QduId>
    },
    /// Record the `StableState` outcome (interpreted as 0 or 1) of the *most recent*
    /// stabilization of a specific QDU into a named classical register.
    ///
    /// # Errors
    /// Returns `OnqError::InvalidOperation` if the specified `qdu` was not part
    /// of the most recently executed `Stabilize` instruction, or if no
    /// `Stabilize` has occurred yet in the relevant scope.
    Record {
        /// The QDU whose stabilization result should be read.
        qdu: QduId,
        /// The name of the classical register (in the VM's `classical_memory`)
        /// where the outcome (0 or 1) will be stored as a `u64`.
        register: String,
    },

    // --- Control Flow ---
    /// Defines a named label at this point in the instruction sequence.
    /// Does not perform any action during execution, used only as a jump target.
    Label(String),
    // Unconditionally jump execution to the instruction immediately following
    /// the specified `Label`.
    ///
    /// # Errors
    /// Returns `OnqError::SimulationError` during VM execution if the `label` is undefined.
    Jump(String),
    /// Conditionally jump execution to the instruction immediately following the
    /// specified `Label` *if* the value in the classical `register` is zero.
    /// If the register does not exist, its value is treated as zero.
    ///
    /// # Errors
    /// Returns `OnqError::SimulationError` during VM execution if the `label` is undefined.
    BranchIfZero {
        /// The name of the classical register to check.
        register: String,
        /// The target label name to jump to if the register's value is 0.
        label: String,
    },
    // --- Classical Operations (Minimal Initial Set) ---
    /// Load an immediate unsigned 64-bit integer value into a classical register.
    LoadImmediate {
        /// The destination register name.
        register: String,
        /// The `u64` value to load.
        value: u64,
    },
    /// Copy the value from one classical register to another.
    Copy {
        /// The name of the register to read from.
        source_reg: String,
        /// The name of the register to write to.
        dest_reg: String,
    },
    // Future: Add arithmetic/logic (Add, Xor, And, Not, Compare, etc.)

    // --- Execution Control ---
    /// Halt the VM execution.
    Halt,
    /// No operation. Can be useful for padding or explicit delays if timing added later.
    NoOp,
    /// Add `value` to the value in `r_src` and store in `r_dest`.
    Addi {
        /// The destination register name.
        r_dest: String,
        /// The source register name.
        r_src: String,
        /// The immediate value to add.
        value: u64,
    },
    /// Add value in `r_src1` to value in `r_src2` and store in `r_dest`.
    OnqAdd {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Perform bitwise NOT on value in `r_src` and store in `r_dest`.
    OnqNot {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src: String,
    },
    /// Perform bitwise AND on values in `r_src1`, `r_src2` and store in `r_dest`.
    And {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Perform bitwise OR on values in `r_src1`, `r_src2` and store in `r_dest`.
    Or {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Perform bitwise XOR on values in `r_src1`, `r_src2` and store in `r_dest`.
    Xor {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Subtract value in `r_src2` from value in `r_src1` and store in `r_dest` (wrapping).
    Sub {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Multiply value in `r_src1` by value in `r_src2` and store in `r_dest` (wrapping).
    Mul {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Compare for equality: Set `r_dest` to 1 if `r_src1` == `r_src2`, else 0.
    CmpEq {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Compare for greater than (unsigned): Set `r_dest` to 1 if `r_src1` > `r_src2`, else 0.
    CmpGt {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
    },
    /// Compare for less than (unsigned): Set `r_dest` to 1 if `r_src1` < `r_src2`, else 0.
    /// Reads 0 for non-existent source registers.
    CmpLt {
        /// The destination register name.
        r_dest: String,
        /// The first source register name.
        r_src1: String,
        /// The second source register name.
        r_src2: String,
     },
}

// --- Program Structure ---

/// Represents a complete program for the ONQ-VM.
/// Contains instructions and resolved label locations.
#[derive(Debug, Clone)] // PartialEq might be complex due to HashMap order
pub struct Program {
    /// Ordered sequence of instructions.
    pub(crate) instructions: Vec<Instruction>,
    /// Map from label name to instruction index (program counter position).
    pub(crate) label_map: HashMap<String, usize>,
}

impl Program {
    /// Creates an empty program. Internal use; use ProgramBuilder.
    fn _new() -> Self {
        Program {
            instructions: Vec::new(),
            label_map: HashMap::new(),
        }
    }

    /// Gets the instruction at a specific index (program counter).
    pub(crate) fn get_instruction(&self, pc: usize) -> Option<&Instruction> {
        self.instructions.get(pc)
    }

    /// Gets the program counter target for a given label name.
    pub(crate) fn get_label_pc(&self, label: &str) -> Option<usize> {
        self.label_map.get(label).copied()
    }

    /// Returns the total number of instructions.
    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }
}

impl fmt::Display for Program {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "ONQ-VM Program ({} instructions)", self.instruction_count())?;
        // Create reverse map for printing labels
        let pc_to_label: HashMap<usize, &String> = self.label_map.iter().map(|(l, pc)| (*pc, l)).collect();

        for (pc, instruction) in self.instructions.iter().enumerate() {
            if let Some(label) = pc_to_label.get(&pc) {
                // Indent instructions slightly, put label flush left
                 writeln!(f, "{}:", label)?;
            }
            // Print PC and indented instruction
            writeln!(f, "  {:04}: {:?}", pc, instruction)?;
        }
        Ok(())
    }
}


// --- Program Builder ---

/// Facilitates the construction of [`Program`] instances using a fluent API.
/// Handles label definition and resolution.
///
/// # Examples
/// ```
/// # use onq::vm::{Instruction, ProgramBuilder};
/// # use onq::operations::Operation;
/// # use onq::core::QduId;
/// let program_result = ProgramBuilder::new()
///     .pb_add(Instruction::LoadImmediate { register: "r0".to_string(), value: 10 })
///     .pb_add(Instruction::Label("start".to_string()))
///     .pb_add(Instruction::QuantumOp(Operation::InteractionPattern{ target: QduId(0), pattern_id: "H".to_string()})) // Placeholder
///     .pb_add(Instruction::BranchIfZero { register: "r0".to_string(), label: "start".to_string()}) // Example conditional jump
///     .pb_add(Instruction::Halt)
///     .build();
///
/// assert!(program_result.is_ok());
/// let program = program_result.unwrap();
/// println!("{}", program);
/// ```
#[derive(Default)]
pub struct ProgramBuilder {
    instructions: Vec<Instruction>,
    label_map: HashMap<String, usize>,
    pending_labels: HashMap<String, Vec<usize>>, // label -> list of instruction indices needing this label's PC
}

impl ProgramBuilder {
    /// Creates an empty program. Internal use; use ProgramBuilder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an instruction to the program sequence.
    pub fn pb_add(mut self, instruction: Instruction) -> Self {
        // Check if this instruction is a label definition
        if let Instruction::Label(label_name) = &instruction {
            let current_pc = self.instructions.len();
            if self.label_map.insert(label_name.clone(), current_pc).is_some() {
                // Handle duplicate label definition error? Or just overwrite? Overwrite for now.
                eprintln!("Warning: Duplicate label definition '{}' at PC {}", label_name, current_pc);
            }
            // Resolve pending jumps to this label (though labels should ideally be defined before use)
            if let Some(_pcs) = self.pending_labels.remove(label_name) {
                 // This logic isn't right for resolving forward jumps if label defined later.
                 // Let's build first, resolve at the end.
                 // We'll just record the label position here.
            }
            // Don't add the Label instruction itself to the executable list, just record its position.
        } else {
             self.instructions.push(instruction);
        }
        self
    }

     /// Adds multiple instructions from an iterator.
     pub fn add_many<I>(mut self, instructions: I) -> Self
     where
        I: IntoIterator<Item = Instruction>,
     {
         for instruction in instructions {
            self = self.pb_add(instruction); // Reuse single add logic
         }
         self
     }

    /// Builds the final `Program`, resolving all labels.
    /// Returns an error if any jump targets are undefined.
    pub fn build(self) -> Result<Program, String> {
        // Validation: Ensure all jump/branch targets exist in label_map
        let mut undefined_labels = Vec::new();
        for instruction in &self.instructions {
            match instruction {
                Instruction::Jump(label) | Instruction::BranchIfZero { label, .. } => {
                    if !self.label_map.contains_key(label) {
                        // Check if already recorded as undefined to avoid duplicates
                        if !undefined_labels.contains(label) {
                            undefined_labels.push(label.clone());
                        }
                    }
                }
                _ => {} // Other instruction types are fine
            }
        }

        if !undefined_labels.is_empty() {
            Err(format!("Undefined labels found: {:?}", undefined_labels))
        } else {
            Ok(Program {
                instructions: self.instructions,
                label_map: self.label_map,
            })
        }
    }
}
