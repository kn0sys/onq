// src/vm/mod.rs

//! Defines the structures and interpreter for the **ONQ Virtual Machine (ONQ-VM)**.
//!
//! This module provides the tools to execute programs containing a mix of
//! derived quantum operations ([`Operation`](crate::operations::Operation)),
//! based state stabilization ([`Instruction::Stabilize`]), classical computation,
//! and control flow logic dependent on stabilization outcomes.
//!
//! ## Key Components:
//! * [`Instruction`]: Enum defining all executable VM operations (quantum, classical, control flow, etc.).
//! * [`Program`]: Represents a compiled, executable sequence of instructions with resolved labels.
//! * [`ProgramBuilder`]: A utility for constructing `Program` instances fluently.
//! * [`OnqVm`]: The virtual machine interpreter that manages state (quantum and classical)
//!   and executes `Program` instructions step-by-step according to derived rules.

// Declare modules
pub mod program;
pub mod interpreter;

// Re-export public types from submodules
pub use program::{Instruction, Program, ProgramBuilder};
pub use interpreter::OnqVm;
