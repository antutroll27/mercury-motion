use serde::{Deserialize, Serialize};

/// Transition specification for sequence mode.
/// Defines how consecutive layers overlap during playback.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransitionSpec {
    Crossfade { duration: u64 },
    Wipe { duration: u64, direction: WipeDirection },
    Slide { duration: u64, direction: WipeDirection },
}

/// Direction for wipe and slide transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WipeDirection {
    Left,
    Right,
    Up,
    Down,
}
