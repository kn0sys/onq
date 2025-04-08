// src/vm/mod.rs

//! Defines the structures and interpreter for the ONQ Virtual Machine (ONQ-VM).
//! Enables mixed classical/quantum computation based on UFF principles.

// Declare modules
pub mod program;
mod interpreter;

// Re-export public types from submodules
pub use program::{Instruction, Program, ProgramBuilder};
pub use interpreter::OnqVm;

// --- Keep other contents if any ---
// (The code previously here, like Instruction/Program/Builder, should now be in program.rs)
// Make sure the contents of the old mod.rs are moved to program.rs if needed.
// If Instruction, Program, ProgramBuilder were defined directly in mod.rs before,
// create program.rs and move them there, then use the pub use lines above.
