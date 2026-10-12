//! How far a state frame may lie from the motion frame it is proven to be,
//! measured from the motion rather than stated: the mean difference between
//! the motion's adjacent sampled frames. A state no further from its best
//! frame than the motion's own frames are from each other is that frame; a
//! still motion, whose frames do not drift, proves only an identical state.

use super::*;

/// The mean absolute difference of two signatures, per byte.
pub(super) fn mean_abs_diff(left: &Signature, right: &Signature) -> f64 {
    let total: u64 = left
        .iter()
        .zip(right)
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    total as f64 / left.len() as f64
}

/// The motion's own drift between adjacent sampled frames, or nothing when
/// it has fewer than two, since one frame has no neighbour to measure.
pub(super) fn frame_drift(frames: &[Signature]) -> Option<f64> {
    let (_, later) = frames.split_first()?;
    let drifts: Vec<f64> = frames
        .iter()
        .zip(later)
        .map(|(earlier, next)| mean_abs_diff(earlier, next))
        .collect();
    (!drifts.is_empty()).then(|| drifts.iter().sum::<f64>() / drifts.len() as f64)
}
