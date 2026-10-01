//! State of the operant chamber's devices.

use serde::{Deserialize, Serialize};

use crate::classical::UsLevel;

/// Pellets accumulate in the cup up to this many.
pub const MAX_PELLETS: u32 = 12;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chamber {
    pub pellets: u32,
    /// The lever is held down until this time.
    pub lever_until: f64,
    pub light_on: bool,
    pub tone_db: Option<f64>,
    pub bell_on: bool,
    pub shock: Option<UsLevel>,
    pub shock_until: f64,
    /// Incremented on every dispenser operation, so the frontend can play the click.
    pub dispense_count: u32,
    pub shock_count: u32,
}

impl Chamber {
    pub fn clear_stimuli(&mut self) {
        self.light_on = false;
        self.tone_db = None;
        self.bell_on = false;
        self.shock = None;
    }
}
