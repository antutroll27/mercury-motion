use crate::schema::easing::NamedEasing;
use crate::schema::{AnimatableValue, EasingValue, Keyframe, Vec2};

use super::easing::{apply as apply_easing, EasingKind};

fn easing_kind(e: &EasingValue) -> EasingKind {
    match e {
        EasingValue::Named(NamedEasing::Linear) => EasingKind::Linear,
        EasingValue::Named(NamedEasing::EaseIn) => EasingKind::EaseIn,
        EasingValue::Named(NamedEasing::EaseOut) => EasingKind::EaseOut,
        EasingValue::Named(NamedEasing::EaseInOut) => EasingKind::EaseInOut,
        EasingValue::CubicBezier {
            x1, y1, x2, y2, ..
        } => EasingKind::CubicBezier {
            x1: *x1,
            y1: *y1,
            x2: *x2,
            y2: *y2,
        },
    }
}

fn normalised_t(from: u64, to: u64, frame: u64, easing: &EasingValue) -> f64 {
    if to == from {
        return 1.0;
    }
    let raw = (frame - from) as f64 / (to - from) as f64;
    apply_easing(easing_kind(easing), raw.clamp(0.0, 1.0))
}

fn find_segment<T>(kfs: &[Keyframe<T>], frame: u64) -> (usize, usize) {
    let idx = kfs.partition_point(|k| k.t <= frame);
    if idx == 0 {
        return (0, 0);
    }
    if idx >= kfs.len() {
        return (kfs.len() - 1, kfs.len() - 1);
    }
    (idx - 1, idx)
}

/// Evaluate an `AnimatableValue<f64>` at the given frame.
pub fn evaluate_f64(av: &AnimatableValue<f64>, frame: u64) -> f64 {
    match av {
        AnimatableValue::Static(v) => *v,
        AnimatableValue::Animated(kfs) => {
            if kfs.is_empty() {
                return 0.0;
            }
            let (from, to) = find_segment(kfs, frame);
            if from == to {
                return kfs[from].v;
            }
            let t = normalised_t(kfs[from].t, kfs[to].t, frame, &kfs[from].easing);
            kfs[from].v + (kfs[to].v - kfs[from].v) * t
        }
    }
}

/// Evaluate an `AnimatableValue<Vec2>` at the given frame.
pub fn evaluate_vec2(av: &AnimatableValue<Vec2>, frame: u64) -> Vec2 {
    match av {
        AnimatableValue::Static(v) => v.clone(),
        AnimatableValue::Animated(kfs) => {
            if kfs.is_empty() {
                return Vec2 { x: 0.0, y: 0.0 };
            }
            let (from, to) = find_segment(kfs, frame);
            if from == to {
                return kfs[from].v.clone();
            }
            let t = normalised_t(kfs[from].t, kfs[to].t, frame, &kfs[from].easing);
            Vec2 {
                x: kfs[from].v.x + (kfs[to].v.x - kfs[from].v.x) * t,
                y: kfs[from].v.y + (kfs[to].v.y - kfs[from].v.y) * t,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn kf(t: u64, v: f64) -> Keyframe<f64> {
        Keyframe {
            t,
            v,
            easing: EasingValue::linear(),
        }
    }

    #[test]
    fn static_value_returns_as_is() {
        let av = AnimatableValue::Static(42.0_f64);
        assert_eq!(evaluate_f64(&av, 0), 42.0);
        assert_eq!(evaluate_f64(&av, 99), 42.0);
    }

    #[test]
    fn before_first_keyframe_holds_first_value() {
        let av = AnimatableValue::Animated(vec![kf(10, 0.0), kf(20, 1.0)]);
        assert!(approx(evaluate_f64(&av, 0), 0.0));
        assert!(approx(evaluate_f64(&av, 9), 0.0));
    }

    #[test]
    fn after_last_keyframe_holds_last_value() {
        let av = AnimatableValue::Animated(vec![kf(0, 0.0), kf(15, 1.0)]);
        assert!(approx(evaluate_f64(&av, 16), 1.0));
        assert!(approx(evaluate_f64(&av, 999), 1.0));
    }

    #[test]
    fn on_keyframe_returns_exact_value() {
        let av = AnimatableValue::Animated(vec![kf(0, 0.0), kf(10, 0.5), kf(20, 1.0)]);
        assert!(approx(evaluate_f64(&av, 10), 0.5));
    }

    #[test]
    fn linear_midpoint_interpolates() {
        let av = AnimatableValue::Animated(vec![kf(0, 0.0), kf(10, 1.0)]);
        assert!(approx(evaluate_f64(&av, 5), 0.5));
    }

    #[test]
    fn single_keyframe_always_returns_its_value() {
        let av = AnimatableValue::Animated(vec![kf(5, 0.75)]);
        assert!(approx(evaluate_f64(&av, 0), 0.75));
        assert!(approx(evaluate_f64(&av, 5), 0.75));
        assert!(approx(evaluate_f64(&av, 100), 0.75));
    }

    #[test]
    fn vec2_linear_interpolates() {
        let av = AnimatableValue::Animated(vec![
            Keyframe {
                t: 0,
                v: Vec2 { x: 0.0, y: 0.0 },
                easing: EasingValue::linear(),
            },
            Keyframe {
                t: 10,
                v: Vec2 {
                    x: 100.0,
                    y: 200.0,
                },
                easing: EasingValue::linear(),
            },
        ]);
        let v = evaluate_vec2(&av, 5);
        assert!(approx(v.x, 50.0));
        assert!(approx(v.y, 100.0));
    }
}
