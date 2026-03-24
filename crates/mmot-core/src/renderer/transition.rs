use crate::schema::TransitionSpec;

/// Calculate opacity modifiers for overlapping layers during a transition.
/// Returns (outgoing_opacity_multiplier, incoming_opacity_multiplier).
pub fn transition_opacity(
    transition: &TransitionSpec,
    progress: f64,
) -> (f64, f64) {
    let progress = progress.clamp(0.0, 1.0);
    match transition {
        TransitionSpec::Crossfade { .. } => {
            (1.0 - progress, progress)
        }
        TransitionSpec::Wipe { .. } | TransitionSpec::Slide { .. } => {
            // Wipe/Slide transitions don't affect opacity — they use spatial masking
            (1.0, 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crossfade_midpoint_is_half() {
        let spec = TransitionSpec::Crossfade { duration: 10 };
        let (out_op, in_op) = transition_opacity(&spec, 0.5);
        assert!((out_op - 0.5).abs() < 1e-6);
        assert!((in_op - 0.5).abs() < 1e-6);
    }

    #[test]
    fn crossfade_start_shows_outgoing() {
        let spec = TransitionSpec::Crossfade { duration: 10 };
        let (out_op, in_op) = transition_opacity(&spec, 0.0);
        assert!((out_op - 1.0).abs() < 1e-6);
        assert!((in_op - 0.0).abs() < 1e-6);
    }

    #[test]
    fn crossfade_end_shows_incoming() {
        let spec = TransitionSpec::Crossfade { duration: 10 };
        let (out_op, in_op) = transition_opacity(&spec, 1.0);
        assert!((out_op - 0.0).abs() < 1e-6);
        assert!((in_op - 1.0).abs() < 1e-6);
    }

    #[test]
    fn crossfade_clamps_negative_progress() {
        let spec = TransitionSpec::Crossfade { duration: 10 };
        let (out_op, in_op) = transition_opacity(&spec, -0.5);
        assert!((out_op - 1.0).abs() < 1e-6);
        assert!((in_op - 0.0).abs() < 1e-6);
    }

    #[test]
    fn crossfade_clamps_over_one_progress() {
        let spec = TransitionSpec::Crossfade { duration: 10 };
        let (out_op, in_op) = transition_opacity(&spec, 1.5);
        assert!((out_op - 0.0).abs() < 1e-6);
        assert!((in_op - 1.0).abs() < 1e-6);
    }

    #[test]
    fn wipe_returns_full_opacity() {
        let spec = TransitionSpec::Wipe {
            duration: 10,
            direction: crate::schema::WipeDirection::Left,
        };
        let (out_op, in_op) = transition_opacity(&spec, 0.5);
        assert!((out_op - 1.0).abs() < 1e-6);
        assert!((in_op - 1.0).abs() < 1e-6);
    }
}
