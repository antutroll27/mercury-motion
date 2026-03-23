/// Normalised easing input/output: both in [0.0, 1.0].
#[derive(Debug, Clone, Copy)]
pub enum EasingKind {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
}

/// Apply an easing function to a normalised `t` in [0.0, 1.0].
pub fn apply(kind: EasingKind, t: f64) -> f64 {
    let t = t.clamp(0.0, 1.0);
    match kind {
        EasingKind::Linear => t,
        EasingKind::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
        EasingKind::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
        EasingKind::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
        EasingKind::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(x1, y1, x2, y2, t),
    }
}

/// Solve a CSS-style cubic Bezier for `y` given `x = t`.
/// Uses Newton's method to find the parameter, then evaluates y.
pub fn cubic_bezier(x1: f64, y1: f64, x2: f64, y2: f64, t: f64) -> f64 {
    let s = solve_t(x1, x2, t);
    bezier_component(y1, y2, s)
}

fn bezier_component(p1: f64, p2: f64, t: f64) -> f64 {
    let c1 = 3.0 * p1;
    let c2 = 3.0 * (p2 - p1) - c1;
    let c3 = 1.0 - c1 - c2;
    ((c3 * t + c2) * t + c1) * t
}

fn bezier_slope(p1: f64, p2: f64, t: f64) -> f64 {
    let c1 = 3.0 * p1;
    let c2 = 3.0 * (p2 - p1) - c1;
    let c3 = 1.0 - c1 - c2;
    (3.0 * c3 * t + 2.0 * c2) * t + c1
}

fn solve_t(x1: f64, x2: f64, x: f64) -> f64 {
    let mut t = x;
    for _ in 0..8 {
        let x_est = bezier_component(x1, x2, t) - x;
        let slope = bezier_slope(x1, x2, t);
        if slope.abs() < 1e-6 {
            break;
        }
        t -= x_est / slope;
        t = t.clamp(0.0, 1.0);
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn linear_midpoint() {
        assert!(approx(apply(EasingKind::Linear, 0.5), 0.5));
    }

    #[test]
    fn linear_endpoints() {
        assert!(approx(apply(EasingKind::Linear, 0.0), 0.0));
        assert!(approx(apply(EasingKind::Linear, 1.0), 1.0));
    }

    #[test]
    fn ease_in_starts_slow() {
        let p = apply(EasingKind::EaseIn, 0.25);
        assert!(p < 0.25, "ease_in should start slow: got {}", p);
    }

    #[test]
    fn ease_out_ends_slow() {
        let p = apply(EasingKind::EaseOut, 0.75);
        assert!(p > 0.75, "ease_out should end slow: got {}", p);
    }

    #[test]
    fn cubic_bezier_identity() {
        let p = cubic_bezier(0.0, 0.0, 1.0, 1.0, 0.5);
        assert!(approx(p, 0.5), "expected ~0.5, got {}", p);
    }

    #[test]
    fn all_easings_start_and_end_at_zero_and_one() {
        for kind in [
            EasingKind::Linear,
            EasingKind::EaseIn,
            EasingKind::EaseOut,
            EasingKind::EaseInOut,
        ] {
            assert!(approx(apply(kind, 0.0), 0.0));
            assert!(approx(apply(kind, 1.0), 1.0));
        }
    }
}
