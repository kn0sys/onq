// src/circuits/mod.rs

//! Defines structures for representing and building ordered sequences of
//! operations (`onq::operations::Operation`).
//!
//! This module provides the `Circuit` structure, which encapsulates a specific,
//! ordered pathway of interactions and state changes (e.g. Sequential Ordering).

// Import necessary types from other modules
use crate::core::QduId; // OnqError might be used for future validation logic
use crate::operations::Operation;
use std::collections::{HashMap, HashSet}; // Using HashSet to efficiently track unique QDUs involved
use std::fmt;

/// Represents an ordered sequence of Operations applied to a set of QDUs.
///
/// This structure embodies (Sequential Ordering) by defining a precise
/// list of interactions to be simulated. It captures a specific process or algorithm within the framework.
///
/// Analogy: Similar to `cirq.Circuit` or `qiskit.QuantumCircuit`, representing the
/// sequence of gates and measurements applied to qubits.
#[derive(Clone, PartialEq)] // PartialEq useful for testing circuits
pub struct Circuit {
    /// The unique set of QDUs involved across all operations in this circuit.
    /// The `HashSet` ensures uniqueness and provides efficient lookup.
    qdus: HashSet<QduId>,

    /// The ordered sequence of operations defining the circuit's logic.
    /// The order is critical and directly reflects (Sequential Ordering).
    operations: Vec<Operation>,

    // --- Potential Future Fields ---
    // /// Optional name for identification or debugging.
    // name: Option<String>,
    // /// Optional explicit reference to the `ReferenceFrame` providing context.
    // frame: Option<ReferenceFrame>, // Would require ReferenceFrame type from core
}

impl Circuit {
    /// Creates a new, empty circuit.
    pub fn new() -> Self {
        Self {
            qdus: HashSet::new(),
            operations: Vec::new(),
            // name: None,
            // frame: None,
        }
    }

    /// Adds a single operation to the end of the circuit's sequence.
    ///
    /// This method automatically identifies the QDUs involved in the `op`
    /// and adds them to the circuit's set of known QDUs.
    ///
    /// # Arguments
    /// * `op` - The `Operation` to append to the sequence.
    pub fn add_operation(&mut self, op: Operation) {
        // Register the QDUs involved in this operation
        for qdu_id in op.involved_qdus() {
            self.qdus.insert(qdu_id);
        }
        // Add the operation to the ordered list
        self.operations.push(op);
    }

    /// Adds multiple operations from an iterator to the end of the circuit's sequence.
    ///
    /// # Arguments
    /// * `ops` - An iterator yielding `Operation` items to append.
    pub fn add_operations<I>(&mut self, ops: I)
    where
        I: IntoIterator<Item = Operation>,
    {
        for op in ops {
            self.add_operation(op);
        }
    }

    /// Returns a reference to the set of unique QDU IDs involved in this circuit.
    pub fn qdus(&self) -> &HashSet<QduId> {
        &self.qdus
    }

    /// Returns a slice containing the ordered sequence of operations in this circuit.
    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    /// Returns the total number of operations defined in the circuit.
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// Returns `true` if the circuit contains no operations.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    // --- Potential Future Methods ---
    // pub fn set_name(&mut self, name: String) { self.name = Some(name); }
    // pub fn name(&self) -> Option<&str> { self.name.as_deref() }
    // pub fn set_frame(&mut self, frame: ReferenceFrame) { self.frame = Some(frame); }
    // pub fn frame(&self) -> Option<&ReferenceFrame> { self.frame.as_ref() }
    // pub fn validate(&self) -> Result<(), OnqError> { /* Check internal consistency */ Ok(()) }
}

// Implement Default for convenient creation of empty circuits.
impl Default for Circuit {
    fn default() -> Self {
        Self::new()
    }
}

//-------------------------------------------------------------------------
// Circuit Builder
//-------------------------------------------------------------------------

/// A helper struct for programmatically constructing `Circuit` instances using method chaining.
pub struct CircuitBuilder {
    circuit: Circuit,
    // Potential future fields:
    // - qdu_allocator: QduAllocator, // To manage unique QDU ID creation during build
    // - default_frame: Option<ReferenceFrame>,
}

impl CircuitBuilder {
    /// Creates a new, empty CircuitBuilder.
    pub fn new() -> Self {
        Self {
            circuit: Circuit::new(),
            // qdu_allocator: QduAllocator::new(),
            // default_frame: None,
        }
    }

    /// Adds a single operation to the circuit being built.
    ///
    /// Returns `self` to allow for continued method chaining.
    pub fn add_op(mut self, op: Operation) -> Self {
        self.circuit.add_operation(op);
        self
    }

    /// Adds multiple operations from an iterator to the circuit being built.
    ///
    /// Returns `self` to allow for continued method chaining.
    pub fn add_ops<I>(mut self, ops: I) -> Self
    where
        I: IntoIterator<Item = Operation>,
    {
        self.circuit.add_operations(ops);
        self
    }

    // --- Potential Future Builder Methods ---
    // pub fn with_name(mut self, name: String) -> Self { self.circuit.set_name(name); self }
    // pub fn with_frame(mut self, frame: ReferenceFrame) -> Self { self.circuit.set_frame(frame); self }
    // /// Allocates a new QDU within the builder's context (requires allocator field)
    // pub fn allocate_qdu(&mut self) -> QduId { self.qdu_allocator.allocate() }

    /// Finalizes the construction process and returns the built `Circuit`.
    pub fn build(self) -> Circuit {
        // Could potentially run validation checks here before returning
        // self.circuit.validate()?;
        self.circuit
    }
}

// Implement Default for convenient creation of builders.
impl Default for CircuitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Circuit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.operations.is_empty() {
            return writeln!(f, "onq::Circuit[0 operations on 0 QDUs]");
        }

        // --- Setup ---
        let ops = &self.operations;
        let num_ops = ops.len();

        // Get sorted list of unique QDUs and create row map
        let mut sorted_qdus: Vec<QduId> = self.qdus.iter().cloned().collect();
        sorted_qdus.sort(); // Sort numerically for consistent row order
        let num_qdus = sorted_qdus.len();
        let qdu_to_row: HashMap<QduId, usize> = sorted_qdus.iter().enumerate().map(|(i, qid)| (*qid, i)).collect();

        // Determine label width
        let max_label_width = sorted_qdus.iter().map(|qid| format!("{}", qid).len()).max().unwrap_or(0);
        let label_padding = " ".repeat(max_label_width + 2); // Label + ": "

        // Grid dimensions and padding
        const GATE_WIDTH: usize = 7; // e.g., "───H───"
        const WIRE: &str = "───────"; // GATE_WIDTH dashes
        const V_WIRE: char = '│';
        const H_WIRE: char = '─';

        // Initialize grids
        // op_grid[row][time] stores the gate/wire segment string
        let mut op_grid: Vec<Vec<String>> = vec![vec![WIRE.to_string(); num_ops]; num_qdus];
        // v_connect[row][time] stores the vertical connector char below this row at this time
        let mut v_connect: Vec<Vec<char>> = vec![vec![' '; num_ops]; num_qdus]; // Note size N x T

        // Helper to format a gate symbol
        fn format_gate(symbol: &str) -> String {
            let slen = symbol.chars().count(); // Use chars().count() for Unicode width if needed
            if slen >= GATE_WIDTH {
                symbol.chars().take(GATE_WIDTH).collect()
            } else {
                let total_dashes = GATE_WIDTH - slen;
                let pre_dashes = total_dashes / 2;
                let post_dashes = total_dashes - pre_dashes;
                format!("{}{}{}", H_WIRE.to_string().repeat(pre_dashes), symbol, H_WIRE.to_string().repeat(post_dashes))
            }
        }

        // --- Populate Grids ---
        for (t, op) in ops.iter().enumerate() {
            match op {
                Operation::PhaseShift { target, .. } => {
                    if let Some(r) = qdu_to_row.get(target) {
                        op_grid[*r][t] = format_gate("P"); // Represent PhaseShift as P for now
                    }
                }
                Operation::InteractionPattern { target, pattern_id } => {
                    if let Some(r) = qdu_to_row.get(target) {
                        let symbol = match pattern_id.as_str() {
                            "Identity" => continue, // Skip explicit Identity, leave wire
                            "QualityFlip" => "X",
                            "PhaseIntroduce" => "Z",
                            "HalfPhase" => "S",
                            "HalfPhase_Inv" => "S†",
                            "QuarterPhase" => "T",
                            "QuarterPhase_Inv" => "T†",
                            "QualitativeY" => "Y",
                            "PhiRotate" => "ΦR", // Using Φ symbol + R
                            "Superposition" => "H",
                            "SqrtFlip" => "√X", // Using √ symbol + X
                            _ => "?", // Unknown pattern
                        };
                        op_grid[*r][t] = format_gate(symbol);
                    }
                }
                Operation::ControlledInteraction { control, target, pattern_id } => {
                    if let (Some(r_ctrl), Some(r_tgt)) = (qdu_to_row.get(control), qdu_to_row.get(target)) {
                        let target_symbol = match pattern_id.as_str() {
                             "QualityFlip" => "X", // Most common controlled op shown this way
                             // Add other specific symbols if needed, default to generic target
                             _ => "●", // Generic controlled target symbol
                        };
                        op_grid[*r_ctrl][t] = format_gate("@");
                        op_grid[*r_tgt][t] = format_gate(target_symbol);

                        // Add vertical connection lines
                        let r_min = (*r_ctrl).min(*r_tgt);
                        let r_max = (*r_ctrl).max(*r_tgt);
                        for row_vec in v_connect.iter_mut().take(r_max).skip(r_min) {
                             row_vec[t] = V_WIRE;
                         }
                    }
                }
                 Operation::RelationalLock { qdu1, qdu2, .. } => {
                     if let (Some(r1), Some(r2)) = (qdu_to_row.get(qdu1), qdu_to_row.get(qdu2)) {
                        let r_min = (*r1).min(*r2);
                        let r_max = (*r1).max(*r2);
                        op_grid[r_min][t] = format_gate("@"); // Use @ for one end
                        op_grid[r_max][t] = format_gate("●"); // Use ● for other end (like CPhase)

                        // Add vertical connection lines
                        for row_vec in v_connect.iter_mut().take(r_max).skip(r_min) {
                             row_vec[t] = V_WIRE;
                         }
                    }
                }
                Operation::Stabilize { targets } => {
                    for target_qid in targets {
                        if let Some(r) = qdu_to_row.get(target_qid) {
                            op_grid[*r][t] = format_gate("M");
                        }
                    }
                    // How to connect multiple non-adjacent measurements? Cirq doesn't. Let's not for now.
                }
            }
        }

        // --- Format Output String ---
        writeln!(f, "onq::Circuit[{} operations on {} QDUs]", num_ops, num_qdus)?;
        for r in 0..num_qdus {
            // Print QDU label row
            let label = format!("{}: ", sorted_qdus[r]);
            write!(f, "{:<width$}", label, width = max_label_width + 2)?;
            writeln!(f, "{}", op_grid[r].join(""))?;

            // Print vertical connector row (if not the last QDU)
            if r < num_qdus - 1 {
                write!(f, "{}", label_padding)?; // Padding for alignment
                for t in 0..num_ops {
                    let connector = v_connect[r][t];
                    let padding_needed = GATE_WIDTH.saturating_sub(1); // Width minus 1 for the connector char
                    let pre_pad = padding_needed / 2;
                    let post_pad = padding_needed - pre_pad;
                    write!(f, "{}{}{}", " ".repeat(pre_pad), connector, " ".repeat(post_pad))?;
                }
                writeln!(f)?; // Newline after connector row
            }
        }
        Ok(())
    }
}

// Keep the Debug impl delegating to Display
impl fmt::Debug for Circuit {
     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
       fmt::Display::fmt(self, f)
    }
}
