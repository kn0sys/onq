// src/validation/mod.rs

//! Provides functions to validate `GeometricPotentialityState` based on localized tensor principles.

use crate::core::{OnqError, PotentialityState};

// Default tolerance values
const DEFAULT_NORM_TOLERANCE: f64 = 1e-6; // Slightly relaxed for tensor product accumulation
const DEFAULT_COHERENCE_THRESHOLD: f64 = 0.618; // The Golden Ratio (1/phi)

// --- Public Validation Functions ---

/// Checks if the tensor network is normalized.
/// In a geometric network, global norm is the product of local tensor norms.
pub fn check_normalization(
    state: &PotentialityState,
    tolerance: Option<f64>,
) -> Result<(), OnqError> {
    let effective_tolerance = tolerance.unwrap_or(DEFAULT_NORM_TOLERANCE);
    let norm_sq = state.global_norm_sq();

    if (norm_sq - 1.0).abs() > effective_tolerance {
        Err(OnqError::Incoherence {
            message: format!(
                "State tensor normalization failed. Total norm squared: {}",
                norm_sq
            ),
        })
    } else {
        Ok(())
    }
}

/// Calculates a global measure of phase coherence for the geometric matrix.
/// It computes the average internal phase alignment of all active LocalTensors.
/// Calculates a global measure of phase coherence for the geometric matrix.
/// It computes the average internal phase alignment of all active LocalTensors.
pub fn calculate_global_phase_coherence(state: &PotentialityState) -> f64 {
    if state.network.is_empty() {
        return 1.0;
    }

    let mut total_coherence = 0.0;
    let mut active_nodes = 0;

    for tensor in state.network.values() {
        let amp0 = tensor.core_state[0];
        let amp1 = tensor.core_state[1];

        // A node firmly in |0> or |1> doesn't have an internal phase relationship to measure.
        if amp0.norm_sqr() > 1e-12 && amp1.norm_sqr() > 1e-12 {
            // Local coherence based on phase alignment of the superposition (|0> vs |1>)
            let phase_diff = (amp0.arg() - amp1.arg()).abs();

            // Cosine of phase diff maps perfect alignment (0) to 1.0, and opposition (PI) to 0.0
            let local_coherence = (1.0 + phase_diff.cos()) / 2.0;

            total_coherence += local_coherence;
            active_nodes += 1;
        }
    }

    if active_nodes == 0 {
        return 1.0;
    } // A baseline or fully resolved state is perfectly coherent

    total_coherence / (active_nodes as f64)
}

/// Checks if the state meets the Phase Coherence threshold (> threshold).
pub fn check_phase_coherence(
    state: &PotentialityState,
    threshold: Option<f64>,
) -> Result<(), OnqError> {
    let effective_threshold = threshold.unwrap_or(DEFAULT_COHERENCE_THRESHOLD);
    let global_coherence = calculate_global_phase_coherence(state);

    if global_coherence > effective_threshold {
        Ok(())
    } else {
        Err(OnqError::Incoherence {
            message: format!(
                "Global Geometric Coherence check failed. Score {:.4} <= Threshold {:.4}",
                global_coherence, effective_threshold
            ),
        })
    }
}

/// Performs basic validation checks on the geometric state.
/// We keep the unused parameters prefixed with `_` so we don't break the VM interpreter calls.
pub fn validate_state(
    state: &PotentialityState,
    _num_qdus: usize,
    norm_tolerance: Option<f64>,
    _coherence_threshold: Option<f64>,
    _amplitude_tolerance: Option<f64>,
) -> Result<(), OnqError> {
    check_normalization(state, norm_tolerance)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_complex::Complex;
    use std::f64::consts::FRAC_1_SQRT_2;

    #[test]
    fn test_tensor_normalization_check() {
        let mut state = PotentialityState::new();

        // Baseline state is entirely |0>, so norm should be exactly 1.0
        assert!(check_normalization(&state, None).is_ok());

        // Sabotage the network by artificially corrupting QDU 0's local tensor
        if let Some(tensor) = state.network.get_mut(&0) {
            tensor.core_state[0] = Complex::new(0.5, 0.0); // Drops norm below 1.0
        }

        // The geometric check should catch the localized collapse
        assert!(check_normalization(&state, None).is_err());
    }

    #[test]
    fn test_geometric_coherence_check() {
        let mut state = PotentialityState::new();

        // Put QDU 0 into |+> state (Highly coherent, phases match)
        if let Some(tensor) = state.network.get_mut(&0) {
            tensor.core_state = [
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::new(FRAC_1_SQRT_2, 0.0),
            ];
        }
        // Score will be 1.0 -> Should easily pass the 0.618 Golden Ratio threshold
        assert!(check_phase_coherence(&state, None).is_ok());

        // Put QDU 0 into |-> state (Phases differ by PI, destructive interference)
        if let Some(tensor) = state.network.get_mut(&0) {
            tensor.core_state = [
                Complex::new(FRAC_1_SQRT_2, 0.0),
                Complex::new(-FRAC_1_SQRT_2, 0.0),
            ];
        }
        // Score will be 0.0 -> Should fail the threshold
        assert!(check_phase_coherence(&state, None).is_err());
    }
}
