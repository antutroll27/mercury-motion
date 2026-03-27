use crate::schema::effects::FcurveModifier;
use crate::schema::Vec2;

/// Apply a chain of modifiers to a scalar value.
pub fn apply_modifiers_f64(
    value: f64,
    frame: u64,
    fps: f64,
    total_frames: u64,
    modifiers: &[FcurveModifier],
) -> f64 {
    let mut v = value;
    for modifier in modifiers {
        v = apply_single_f64(v, frame, fps, total_frames, modifier);
    }
    v
}

/// Apply a chain of modifiers to a Vec2 value (applies component-wise for scalar modifiers).
pub fn apply_modifiers_vec2(
    value: Vec2,
    frame: u64,
    fps: f64,
    total_frames: u64,
    modifiers: &[FcurveModifier],
) -> Vec2 {
    let mut v = value;
    for modifier in modifiers {
        v = apply_single_vec2(v, frame, fps, total_frames, modifier);
    }
    v
}

fn apply_single_f64(
    value: f64,
    frame: u64,
    fps: f64,
    _total_frames: u64,
    modifier: &FcurveModifier,
) -> f64 {
    match modifier {
        FcurveModifier::Wiggle {
            amplitude,
            frequency,
            seed,
        } => {
            let time = frame as f64 / fps;
            let noise = pseudo_noise(time * frequency, *seed);
            value + noise * amplitude
        }
        FcurveModifier::Loop { mode: _ } => {
            // Loop remaps the frame before evaluation, not the value after.
            // At the value level we pass through unchanged; frame remapping
            // would need to be handled upstream if implemented.
            value
        }
        FcurveModifier::Clamp { min, max } => value.clamp(*min, *max),
    }
}

fn apply_single_vec2(
    value: Vec2,
    frame: u64,
    fps: f64,
    _total_frames: u64,
    modifier: &FcurveModifier,
) -> Vec2 {
    match modifier {
        FcurveModifier::Wiggle {
            amplitude,
            frequency,
            seed,
        } => {
            let time = frame as f64 / fps;
            // Use different seed offsets for x and y to avoid correlated noise
            let noise_x = pseudo_noise(time * frequency, *seed);
            let noise_y = pseudo_noise(time * frequency, seed.wrapping_add(7919));
            Vec2 {
                x: value.x + noise_x * amplitude,
                y: value.y + noise_y * amplitude,
            }
        }
        FcurveModifier::Loop { mode: _ } => value,
        FcurveModifier::Clamp { min, max } => Vec2 {
            x: value.x.clamp(*min, *max),
            y: value.y.clamp(*min, *max),
        },
    }
}

/// Deterministic pseudo-noise function. Same inputs always produce the same output.
/// Returns a value in the range `[-1.0, 1.0]`.
fn pseudo_noise(t: f64, seed: u32) -> f64 {
    let x = (t * 12.9898 + seed as f64 * 78.233).sin() * 43758.5453;
    (x - x.floor()) * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::effects::LoopMode;

    #[test]
    fn wiggle_adds_noise() {
        let modifiers = vec![FcurveModifier::Wiggle {
            amplitude: 0.5,
            frequency: 5.0,
            seed: 42,
        }];
        let original = 1.0;
        let modified = apply_modifiers_f64(original, 10, 30.0, 60, &modifiers);
        assert!(
            (modified - original).abs() > 1e-9,
            "wiggle should change the value, got {modified}"
        );
    }

    #[test]
    fn wiggle_deterministic() {
        let modifiers = vec![FcurveModifier::Wiggle {
            amplitude: 0.3,
            frequency: 4.0,
            seed: 123,
        }];
        let a = apply_modifiers_f64(1.0, 15, 30.0, 60, &modifiers);
        let b = apply_modifiers_f64(1.0, 15, 30.0, 60, &modifiers);
        assert_eq!(a, b, "same seed + frame must produce identical result");
    }

    #[test]
    fn clamp_constrains_value() {
        let modifiers = vec![FcurveModifier::Clamp {
            min: 0.0,
            max: 1.0,
        }];
        assert_eq!(apply_modifiers_f64(1.5, 0, 30.0, 60, &modifiers), 1.0);
        assert_eq!(apply_modifiers_f64(-0.5, 0, 30.0, 60, &modifiers), 0.0);
    }

    #[test]
    fn clamp_passes_value_in_range() {
        let modifiers = vec![FcurveModifier::Clamp {
            min: 0.0,
            max: 1.0,
        }];
        assert_eq!(apply_modifiers_f64(0.5, 0, 30.0, 60, &modifiers), 0.5);
    }

    #[test]
    fn no_modifiers_unchanged() {
        let modifiers: Vec<FcurveModifier> = vec![];
        assert_eq!(apply_modifiers_f64(0.75, 10, 30.0, 60, &modifiers), 0.75);
    }

    #[test]
    fn chained_modifiers() {
        let modifiers = vec![
            FcurveModifier::Wiggle {
                amplitude: 10.0,
                frequency: 1.0,
                seed: 0,
            },
            FcurveModifier::Clamp {
                min: 0.0,
                max: 1.0,
            },
        ];
        let result = apply_modifiers_f64(0.5, 5, 30.0, 60, &modifiers);
        assert!(
            (0.0..=1.0).contains(&result),
            "clamp after wiggle should bound the result, got {result}"
        );
    }

    #[test]
    fn vec2_wiggle_applies_to_both_components() {
        let modifiers = vec![FcurveModifier::Wiggle {
            amplitude: 5.0,
            frequency: 2.0,
            seed: 99,
        }];
        let original = Vec2 {
            x: 100.0,
            y: 200.0,
        };
        let modified = apply_modifiers_vec2(original.clone(), 10, 30.0, 60, &modifiers);
        assert!(
            (modified.x - original.x).abs() > 1e-9 || (modified.y - original.y).abs() > 1e-9,
            "wiggle should change at least one component"
        );
    }

    #[test]
    fn loop_modifier_passes_through() {
        let modifiers = vec![FcurveModifier::Loop {
            mode: LoopMode::PingPong,
        }];
        assert_eq!(apply_modifiers_f64(0.5, 10, 30.0, 60, &modifiers), 0.5);
    }

    #[test]
    fn pseudo_noise_is_deterministic() {
        let a = pseudo_noise(1.5, 42);
        let b = pseudo_noise(1.5, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn pseudo_noise_range() {
        for i in 0..100 {
            let n = pseudo_noise(i as f64 * 0.1, 0);
            assert!(
                (-1.0..=1.0).contains(&n),
                "noise out of range at i={i}: {n}"
            );
        }
    }
}
