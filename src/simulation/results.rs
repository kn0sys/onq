// src/simulation/results.rs
use crate::core::{QduId, StableState};
use std::collections::HashMap;
use std::fmt;

/// Holds the results of a circuit simulation.
/// Contains the final `StableState` outcomes for QDUs that underwent stabilization.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationResult {
    /// Maps stabilized QDU IDs to their resulting StableState.
    stable_outcomes: HashMap<QduId, StableState>,
    // Optional: Include the final potentiality states of non-stabilized QDUs
    // final_potentialities: HashMap<QduId, PotentialityState>,
}

impl SimulationResult {
    /// Creates a new, empty result set. (Internal visibility)
    pub(crate) fn new() -> Self {
        Self {
            stable_outcomes: HashMap::new(),
            // final_potentialities: HashMap::new(),
        }
    }

    /// Records a stable outcome for a QDU. (Internal visibility)
    pub(crate) fn record_stable_state(&mut self, qdu_id: QduId, state: StableState) {
        self.stable_outcomes.insert(qdu_id, state);
    }

    /// Gets the stable outcome for a specific QDU, if it was stabilized during the simulation.
    /// Returns `None` if the QDU was not stabilized or not part of the simulation.
    pub fn get_stable_state(&self, qdu_id: &QduId) -> Option<&StableState> {
        self.stable_outcomes.get(qdu_id)
    }

    /// Returns a reference to the map containing all recorded stable outcomes.
    pub fn all_stable_outcomes(&self) -> &HashMap<QduId, StableState> {
        &self.stable_outcomes
    }
}

impl fmt::Display for SimulationResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Simulation Results:")?;
        if self.stable_outcomes.is_empty() {
            writeln!(f, "  No QDUs were stabilized.")?;
        } else {
            // Sort by QduId for consistent and readable output
            let mut sorted_outcomes: Vec<_> = self.stable_outcomes.iter().collect();
            sorted_outcomes.sort_by_key(|(id, _)| *id);
            writeln!(f, "  Stable Outcomes:")?;
            for (id, state) in sorted_outcomes {
                writeln!(f, "    {}: {}", id, state)?;
            }
        }
        // Add display logic here if final_potentialities is included later
        Ok(())
    }
}
