// src/operations/mod.rs

//! Defines operations derived from principles governing
//! state transformation, interaction, and stabilization.
//!
//! These operations represent the fundamental ways in which Qualitative
//! Distinction Units (QDUs) can interact and have their states modified
//! within a simulation, according to the necessary consequences outlined
//! in the framework.

// Import necessary types from the core module
use crate::core::QduId;
use crate::vm::program::LockType;
/// Represents a defined operation within onq framework.
///
/// Operations are derived from principles like:
/// - State Transition & Sequential Ordering
/// - State Influence
/// - Interactive Necessity & Causation
/// - Structural Feedback
///
///   And potentially map to constructs like:
/// - State Transformation `T = P(n+1) ⊗ P(n)`
/// - Field Overlay `F = F₁ ⊗ F₂`
/// - Phase manipulation (`e^(iθ)`, Rotary Junction `⊕─○`)
/// - Relational locks (Phase Lock `○↔○`, Parallel Integration `∥`)
///
/// These operations act upon `PotentialityState` within the simulation engine.
#[derive(Debug, Clone, PartialEq)] // Using PartialEq for simplicity; f64 comparison needs care in practice.
pub enum Operation {
    /// Represents applying a phase shift to a single QDU.
    /// Derived from the inherent phase component `e^(iθ)` in
    /// primary functions (Ω, I) and potentially structural elements
    /// enabling phase manipulation (like Rotary Junction `⊕─○`).
    /// This directly modifies the 'quality' aspect related to phase.
    ///
    /// Analogy: Similar to Rz or Phase gates in quantum computing.
    PhaseShift {
        /// The target QDU whose potentiality state phase is modified.
        target: QduId,
        /// The phase angle `theta` (in radians) to apply. Its value should ideally
        /// be derived from the specific context or interaction being modelled.
        theta: f64,
    },

    /// Represents a fundamental transformation or interaction pattern applied to a single QDU.
    /// This is intended to model the effect of State Transformation
    /// rule `T = P_op ⊗ P(qdu_state)`, where `P_op` is a specific, stable interaction pattern.
    /// The exact set of allowed transformations must be derived from analyzing stable patterns.
    ///
    /// Analogy: Similar to single-qubit gates like X, Y, Z, H in quantum computing, but the
    /// specific transformations available (`transform_id`) must be justified.
    InteractionPattern {
        /// The target QDU undergoing the transformation.
        target: QduId,
        /// Identifier for the specific transformation pattern (`P_op`).
        /// **Placeholder:** Using String. This needs refinement to an enum or struct based on
        /// actual derived stable interaction patterns from analysis.
        pattern_id: String,
        // Future: May include parameters specific to the pattern.
    },

    /// Represents a controlled interaction between two QDUs.
    /// Derived from Interactive Necessity, where frames/distinctions influence
    /// each other, and Integration requirements that link QDUs within a frame.
    /// The state/quality of the `control` QDU determines if/how the `target` QDU is affected,
    /// reflecting State Influence. It must respect Parallel Integration (`∥`) rules
    /// if the QDUs are explicitly linked.
    ///
    /// Analogy: Similar to controlled gates like CNOT or CZ, but the available interactions
    /// (`pattern_id`) must be justified.
    ControlledInteraction {
        /// The QDU whose state/quality determines if the interaction occurs.
        control: QduId,
        /// The QDU that is potentially transformed by the interaction pattern.
        target: QduId,
        /// Identifier for the transformation pattern (`P_op`)
        /// applied to the target QDU, conditioned on the control QDU's state.
        /// **Placeholder:** Using String, needs refinement based on derived patterns.
        pattern_id: String,
    },

    /// Represents establishing, modifying, or breaking a specific phase relationship
    /// or structural lock between two QDUs.
    /// Derived from Reference Structure, Frame Interaction,
    /// Integration Requirement, and potentially simulating
    /// Phase Lock (`○↔○`) or shared reference rules in Parallel Integration (`∥`).
    ///
    /// Analogy: Could be analogous to operations creating/breaking entanglement or
    /// enforcing phase coherence, but details depend on derivation.
    RelationalLock {
        /// The first QDU involved in the lock.
        qdu1: QduId,
        /// The second QDU involved in the lock.
        qdu2: QduId,
        /// The target integrated/entangled state type for the lock.
        lock_type: LockType,
        /// If true, project onto lock state; if false, currently no-op.
        establish: bool,
    },

    /// Represents the Stabilization Protocol (SP).
    /// This operation instructs the simulation engine to attempt resolution
    /// of the `PotentialityState` of the `targets` into a `StableState`.
    /// This simulates the processes of:
    /// - Pattern Formation selecting stable outcomes
    /// - Integration Requirement forcing coherence
    /// - Reality Formation through interaction/feedback
    ///
    /// It should involve checks such as coherence, stability and resonance.
    /// The outcome is probabilistic only if the underlying dynamics
    /// leading to stabilization inherently contain multiple stable endpoints for a given potentiality state.
    ///
    /// Analogy: Similar to measurement in quantum computing.
    Stabilize {
        /// The list of QDU IDs whose states should be resolved.
        targets: Vec<QduId>,
    },

    // Future considerations:
    // - Operations derived from Field Overlay `F = F₁ ⊗ F₂`.
    // - Operations representing explicit Boundary interactions.
    // - No-op or Delay operations if timing/sequence needs explicit pauses.
}

impl Operation {
    /// Returns a list of all QDU IDs directly mentioned in the operation's parameters.
    ///
    /// This helps identify which parts of the state vector might be affected by an operation,
    /// although the actual effect (especially for multi-QDU gates) modifies the global state.
    ///
    /// # Examples
    /// ```
    /// # use onq::{Operation, QduId, LockType};
    /// let q0 = QduId(0);
    /// let q1 = QduId(1);
    /// let op_h = Operation::InteractionPattern { target: q0, pattern_id: "H".to_string() };
    /// let op_cx = Operation::ControlledInteraction { control: q0, target: q1, pattern_id: "X".to_string() };
    /// let op_lock = Operation::RelationalLock { qdu1: q0, qdu2: q1, lock_type: LockType::BellPhiPlus, establish: true };
    /// let op_stab = Operation::Stabilize { targets: vec![q0, q1] };
    ///
    /// assert_eq!(op_h.involved_qdus(), vec![q0]);
    /// // Note: Order might not be guaranteed depending on internal representation if changed from vec!
    /// let mut cx_qdus = op_cx.involved_qdus(); cx_qdus.sort();
    /// assert_eq!(cx_qdus, vec![q0, q1]);
    /// let mut lock_qdus = op_lock.involved_qdus(); lock_qdus.sort();
    /// assert_eq!(lock_qdus, vec![q0, q1]);
    /// let mut stab_qdus = op_stab.involved_qdus(); stab_qdus.sort();
    /// assert_eq!(stab_qdus, vec![q0, q1]);
    /// ```
    pub fn involved_qdus(&self) -> Vec<QduId> {
        match self {
            Operation::PhaseShift { target, .. } => vec![*target],
            Operation::InteractionPattern { target, .. } => vec![*target],
            Operation::ControlledInteraction { control, target, .. } => vec![*control, *target],
            Operation::RelationalLock { qdu1, qdu2, .. } => vec![*qdu1, *qdu2],
            Operation::Stabilize { targets } => targets.clone(),
        }
    }

    // Potential future methods:
    // - `validate(&self, context: &SimulationContext) -> Result<(), OnqError>`
    // - `required_frame_properties(&self) -> FrameProperties`
}
