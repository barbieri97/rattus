//! Position and current activity of the rat.
//!
//! The chamber is seen from the side. `x` runs from the left wall (water spout) to the right
//! wall (lever and food cup); `z` is depth, used only to vary the drawing.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::behavior::Behavior;

/// Leftmost position of the rat's body centre (at the water spout).
pub const X_MIN: f64 = 0.10;
/// Rightmost position of the rat's body centre (at the lever and food cup).
pub const X_MAX: f64 = 0.86;
pub const LEVER_X: f64 = X_MAX;
pub const HOPPER_X: f64 = X_MAX;
pub const SPOUT_X: f64 = X_MIN;
/// Depth at which the rat uses the lever, the food cup and the spout.
pub const DEVICE_Z: f64 = 0.45;
/// Walking speed in chamber widths per second.
pub const WALK_SPEED: f64 = 0.28;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bout {
    pub behavior: Behavior,
    pub elapsed: f64,
    pub duration: f64,
    pub target_x: f64,
    pub target_z: f64,
    /// Behaviour to start when a walk arrives.
    pub then: Option<Behavior>,
    /// Set once the bout's one-off action (the lever press) has happened.
    pub acted: bool,
    /// Set when this bout has been reinforced.
    pub credited: bool,
}

impl Bout {
    pub fn new(behavior: Behavior, duration: f64, x: f64, z: f64) -> Self {
        Self {
            behavior,
            elapsed: 0.0,
            duration,
            target_x: x,
            target_z: z,
            then: None,
            acted: false,
            credited: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rat {
    pub x: f64,
    pub z: f64,
    pub facing: Facing,
    pub bout: Bout,
}

impl Rat {
    pub fn new(x: f64, z: f64, facing: Facing) -> Self {
        Self {
            x,
            z,
            facing,
            bout: Bout::new(Behavior::LookAround, 1.5, x, z),
        }
    }

    /// At the right wall facing it: can press the lever or eat from the cup.
    pub fn at_lever(&self) -> bool {
        self.x >= LEVER_X - 0.04 && self.facing == Facing::Right
    }

    /// At the left wall facing it: can drink.
    pub fn at_spout(&self) -> bool {
        self.x <= SPOUT_X + 0.04 && self.facing == Facing::Left
    }

    pub fn at_wall(&self) -> bool {
        self.at_lever() || self.at_spout()
    }
}
