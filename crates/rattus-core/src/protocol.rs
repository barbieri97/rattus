//! Messages exchanged with the user interface.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::behavior::Behavior;
use crate::classical::{ClassicalDesign, UsLevel};
use crate::rat::Facing;
use crate::recorder::{AssocSample, RecordEvent, TrialRecord};
use crate::schedule::OperantDesign;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RatView {
    pub x: f64,
    pub z: f64,
    pub facing: Facing,
    pub behavior: Behavior,
    /// Seconds since the current bout started.
    pub elapsed: f64,
    pub duration: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChamberView {
    pub pellets: u32,
    pub lever_down: bool,
    pub light_on: bool,
    pub tone_db: Option<f64>,
    pub bell_on: bool,
    pub shock: Option<UsLevel>,
    pub dispense_count: u32,
    pub shock_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ActionStrength {
    pub behavior: Behavior,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct MindView {
    pub sound_food: f64,
    pub bar_sound: f64,
    pub bar_strength: f64,
    pub fear: f64,
    pub pain_sensitivity: f64,
    pub v_light: f64,
    pub v_tone: f64,
    pub v_bell: f64,
    pub actions: Vec<ActionStrength>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Counters {
    pub presses: u32,
    pub reinforcers: u32,
    pub pellets_eaten: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClassicalProgress {
    /// Current stage, from 1.
    pub stage: u32,
    pub stage_count: u32,
    /// Current trial within the stage, from 1.
    pub trial_in_stage: u32,
    pub trials_in_stage: u32,
    pub total_trial: u32,
    pub total_trials: u32,
    pub in_cs: bool,
    /// Seconds until the CS comes on (between trials) or goes off (during the CS).
    pub remaining: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Snapshot {
    /// Program time in seconds.
    pub t: f64,
    pub rat: RatView,
    pub chamber: ChamberView,
    pub mind: MindView,
    pub counters: Counters,
    pub operant: OperantDesign,
    pub classical: Option<ClassicalProgress>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BehaviorStart {
    pub t: f64,
    pub behavior: Behavior,
}

/// Starting points for a new rat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RatTemplate {
    /// A rat that has never been in the chamber.
    Naive,
    /// A rat already trained to press the bar on a VR-25 schedule, for CER experiments.
    BarTrained,
}

/// Actions on the simulated experiment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum Command {
    /// Operate the food dispenser by hand (magazine training and shaping).
    GivePellet,
    MarkRecord {
        label: Option<String>,
    },
    SetOperantDesign {
        design: OperantDesign,
    },
    SetClassicalDesign {
        design: ClassicalDesign,
    },
    StartClassical,
    StopClassical,
    /// Take the rat to its home cage for some hours.
    TimeOff {
        hours: f64,
    },
    NewRat {
        template: RatTemplate,
        seed: f64,
    },
    RandomizePosition,
}

/// State of the runner that drives the simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HostStatus {
    pub paused: bool,
    /// Program seconds per real second while the rat is visible.
    pub speed: f64,
    pub isolated: bool,
    /// Program seconds per real second while the rat is isolated.
    pub isolated_speed: f64,
}

impl Default for HostStatus {
    fn default() -> Self {
        Self {
            paused: false,
            speed: 1.0,
            isolated: false,
            isolated_speed: 600.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum HostControl {
    Pause,
    Resume,
    SetSpeed { speed: f64 },
    SetIsolated { isolated: bool },
    SetIsolatedSpeed { speed: f64 },
}

/// Data recorded since the previous frame.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Delta {
    pub events: Vec<RecordEvent>,
    pub trials: Vec<TrialRecord>,
    pub samples: Vec<AssocSample>,
    pub behaviors: Vec<BehaviorStart>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Frame {
    pub snapshot: Snapshot,
    pub host: HostStatus,
    pub delta: Delta,
}

/// Everything the interface needs to rebuild itself (after opening a file, for instance).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FullState {
    pub snapshot: Snapshot,
    pub host: HostStatus,
    pub events: Vec<RecordEvent>,
    pub trials: Vec<TrialRecord>,
    pub samples: Vec<AssocSample>,
    pub classical_design: ClassicalDesign,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum SimMessage {
    Frame(Box<Frame>),
    Reset(Box<FullState>),
}
