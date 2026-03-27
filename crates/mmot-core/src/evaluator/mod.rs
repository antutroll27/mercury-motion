pub mod easing;
pub mod interpolate;
pub mod modifiers;

pub use interpolate::{evaluate_f64, evaluate_vec2};
pub use modifiers::{apply_modifiers_f64, apply_modifiers_vec2};
