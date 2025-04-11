// src/validation/mod.rs

//! Provides functions to validate `PotentialityState` based on principles.

use crate::core::{PotentialityState, OnqError};
use num_complex::Complex;

// Default tolerance values (can be overridden by caller)
const DEFAULT_NORM_TOLERANCE: f64 = 1e-9;
const DEFAULT_AMPLITUDE_TOLERANCE: f64 = 1e-12;
const DEFAULT_COHERENCE_THRESHOLD: f64 = 0.618;

// --- Helper Functions ---

/// Helper to calculate the interpretive (Phase Coherence) for a specific basis state k.
/// Measures phase agreement with nearest neighbour basis states that have non-negligible amplitude.
/// Returns a score between 0.0 and 1.0. (Internal to this module)
fn calculate_c1_score_for_state_k(
    k: usize,
    state_vector: &[Complex<f64>],
    num_qdus: usize,
    amplitude_tolerance: f64,
) -> f64 {
    if num_qdus == 0 { return 1.0; } // Coherence is perfect for 0 QDUs

    let amp_k = state_vector[k];
    let amp_k_norm_sq = amp_k.norm_sqr();

    // If the amplitude of state k itself is negligible, coherence contribution is considered low/zero.
    if amp_k_norm_sq < amplitude_tolerance {
         return 0.0;
    }
    let phase_k = amp_k.arg();

    let mut total_cos_diff = 0.0;
    let mut num_significant_neighbours = 0;

    for bit_pos in 0..num_qdus {
        let neighbour_k = k ^ (1 << bit_pos); // Index of neighbour differing at bit_pos
        // Check bounds: neighbour index must be less than state_vector length
        if neighbour_k < state_vector.len() {
             let amp_neighbour = state_vector[neighbour_k];
             // Only consider neighbours with significant amplitude for phase comparison
             if amp_neighbour.norm_sqr() > amplitude_tolerance {
                let phase_neighbour = amp_neighbour.arg();
                total_cos_diff += (phase_k - phase_neighbour).cos();
                num_significant_neighbours += 1;
             }
        }
    }

    if num_significant_neighbours == 0 {
         // No significant neighbours to compare phase with - consider it coherent by default?
         // This happens for basis states in N=1 case or states with sparse neighbours.
         return 1.0;
    }

    let avg_cos_diff = total_cos_diff / (num_significant_neighbours as f64);
    // Normalize score to be between 0.0 and 1.0
    let score = (1.0 + avg_cos_diff) / 2.0;
    score.clamp(0.0, 1.0) // Clamp for robustness
}


// --- Public Validation Functions ---

/// Checks if the state vector is normalized (sum of squared amplitudes ≈ 1.0).
/// This relates to (`∑P_n converges`) if P_n is interpreted as probability.
///
/// # Arguments
/// * `state` - The `PotentialityState` to check.
/// * `tolerance` - Allowed deviation from 1.0 (e.g., 1e-9). Defaults are available.
///
/// # Returns
/// * `Ok(())` if normalized within tolerance.
/// * `Err(OnqError::Incoherence)` if normalization fails.
pub fn check_normalization(state: &PotentialityState, tolerance: Option<f64>) -> Result<(), OnqError> {
    let effective_tolerance = tolerance.unwrap_or(DEFAULT_NORM_TOLERANCE);
    let norm_sq: f64 = state.vector().iter().map(|c| c.norm_sqr()).sum();
    if (norm_sq - 1.0).abs() > effective_tolerance {
        Err(OnqError::Incoherence {
            message: format!("State vector normalization failed. Sum(|c_i|^2) = {} (Deviation > {})", norm_sq, effective_tolerance)
        })
    } else {
        Ok(())
    }
}

/// Calculates a global measure of phase coherence.
/// It computes the average C1 score across all basis states, weighted by their amplitude squared.
///
/// # Arguments
/// * `state` - The `PotentialityState` to analyze.
/// * `num_qdus` - The number of QDUs represented by the state vector.
/// * `amplitude_tolerance` - Optional threshold below which amplitudes are considered negligible.
///
/// # Returns
/// * A global coherence score between 0.0 and 1.0.
pub fn calculate_global_phase_coherence(
    state: &PotentialityState,
    num_qdus: usize,
    amplitude_tolerance: Option<f64>,
) -> f64 {
    let effective_amp_tolerance = amplitude_tolerance.unwrap_or(DEFAULT_AMPLITUDE_TOLERANCE);
    let state_vector = state.vector();
    let dim = state.dim();
    if dim == 0 || num_qdus == 0 { return 1.0; } // Empty or single state is coherent

    let mut total_weighted_coherence = 0.0;
    let mut total_norm_sq = 0.0; // Recalculate for weighting, handles potentially unnormalized states

    for k in 0..dim {
        let amplitude_sq = state_vector[k].norm_sqr();
        if amplitude_sq > effective_amp_tolerance {
             let score_c1_k = calculate_c1_score_for_state_k(
                 k,
                 state_vector,
                 num_qdus,
                 effective_amp_tolerance
             );
             total_weighted_coherence += amplitude_sq * score_c1_k;
             total_norm_sq += amplitude_sq;
        }
    }

    if total_norm_sq < effective_amp_tolerance {
        0.0 // State has negligible norm, considered incoherent
    } else {
        // Return the weighted average coherence score
        (total_weighted_coherence / total_norm_sq).clamp(0.0, 1.0)
    }
}

/// Checks if the state meets the Phase Coherence threshold (> threshold).
/// Uses the global weighted average coherence score calculated by `calculate_global_phase_coherence`.
///
/// # Arguments
/// * `state` - The `PotentialityState` to check.
/// * `num_qdus` - The number of QDUs represented by the state vector.
/// * `threshold` - Optional minimum required coherence score (defaults to 0.618).
/// * `amplitude_tolerance` - Optional threshold for negligible amplitudes.
///
/// # Returns
/// * `Ok(())` if the coherence threshold is met.
/// * `Err(OnqError::Incoherence)` if the threshold is not met.
pub fn check_phase_coherence(
    state: &PotentialityState,
    num_qdus: usize,
    threshold: Option<f64>,
    amplitude_tolerance: Option<f64>,
) -> Result<(), OnqError> {
    let effective_threshold = threshold.unwrap_or(DEFAULT_COHERENCE_THRESHOLD);
    let effective_amp_tolerance = amplitude_tolerance.unwrap_or(DEFAULT_AMPLITUDE_TOLERANCE);

    let global_coherence = calculate_global_phase_coherence(state, num_qdus, Some(effective_amp_tolerance));
    if global_coherence > effective_threshold {
        Ok(())
    } else {
        Err(OnqError::Incoherence{
            message: format!("Global Phase Coherence check failed. Score {:.4} <= Threshold {:.4}", global_coherence, effective_threshold)
        })
    }
}

/// Performs basic validation checks on the state based on criteria.
/// Currently only checks normalization. Coherence checks may be applied elsewhere (e.g., pre-stabilization).
/// Uses default tolerance values unless specified.
///
/// # Arguments
/// * `state` - The `PotentialityState` to validate.
/// * `num_qdus` - The number of QDUs represented by the state vector. (Currently unused here)
/// * `norm_tolerance` - Optional allowed deviation from 1.0 for normalization.
/// * `coherence_threshold` - Optional: Currently UNUSED in this function.
/// * `amplitude_tolerance` - Optional: Currently UNUSED in this function.
///
/// # Returns
/// * `Ok(())` if all checks pass.
/// * `Err(OnqError::Incoherence)` if any check fails.
pub fn validate_state(
    state: &PotentialityState,
    _num_qdus: usize, // Keep param for API consistency for now
    norm_tolerance: Option<f64>,
    _coherence_threshold: Option<f64>, // Mark as unused
    _amplitude_tolerance: Option<f64>, // Mark as unused
) -> Result<(), OnqError> {
    // Use provided tolerances or defaults
    let eff_norm_tol = norm_tolerance.unwrap_or(DEFAULT_NORM_TOLERANCE);
    // let eff_coh_thresh = coherence_threshold.unwrap_or(DEFAULT_COHERENCE_THRESHOLD);
    // let eff_amp_tol = amplitude_tolerance.unwrap_or(DEFAULT_AMPLITUDE_TOLERANCE);

    // Perform checks
    check_normalization(state, Some(eff_norm_tol))?;
    // REMOVED: check_phase_coherence(state, num_qdus, Some(eff_coh_thresh), Some(eff_amp_tol))?;

    Ok(())
}
