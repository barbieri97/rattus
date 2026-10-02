//! Everything that is recorded during a session, and its CSV export.

use std::fmt::Write;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::classical::{CsSpec, UsLevel};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ReinforcerKind {
    /// Delivered by the schedule: click and pellet.
    Food,
    /// Delivered by the schedule: click without pellet.
    Sound,
    /// Pellet given by the user.
    Manual,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum RecordEvent {
    Press { t: f64, reinforced: bool },
    Reinforcer { t: f64, kind: ReinforcerKind },
    Mark { t: f64, label: String },
    CsOn { t: f64, cs: CsSpec },
    CsOff { t: f64 },
    Shock { t: f64, level: UsLevel },
    DesignChange { t: f64, label: String },
    TimeOff { t: f64, hours: f64 },
    ClassicalStart { t: f64 },
    ClassicalEnd { t: f64, completed: bool },
}

impl RecordEvent {
    pub fn t(&self) -> f64 {
        match self {
            RecordEvent::Press { t, .. }
            | RecordEvent::Reinforcer { t, .. }
            | RecordEvent::Mark { t, .. }
            | RecordEvent::CsOn { t, .. }
            | RecordEvent::CsOff { t }
            | RecordEvent::Shock { t, .. }
            | RecordEvent::DesignChange { t, .. }
            | RecordEvent::TimeOff { t, .. }
            | RecordEvent::ClassicalStart { t }
            | RecordEvent::ClassicalEnd { t, .. } => *t,
        }
    }
}

/// Results of one classical conditioning trial.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrialRecord {
    /// Trial number in the whole experiment, from 1.
    pub trial: u32,
    /// Stage number, from 1.
    pub stage: u32,
    pub trial_in_stage: u32,
    /// Time of CS onset.
    pub t: f64,
    pub cs: CsSpec,
    pub us: UsLevel,
    pub pre_presses: u32,
    pub cs_presses: u32,
    pub suppression_ratio: Option<f64>,
    /// Seconds spent moving in the 30 s before the CS.
    pub pre_move: f64,
    /// Seconds spent moving during the CS.
    pub cs_move: f64,
    pub movement_ratio: Option<f64>,
    /// CS response strengths after the trial.
    pub v_light: f64,
    pub v_tone: f64,
    pub v_bell: f64,
    pub fear_peak: f64,
    pub pain_sensitivity: f64,
}

/// Periodic sample of the rat's mind, for the history plots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AssocSample {
    pub t: f64,
    pub sound_food: f64,
    pub bar_sound: f64,
    pub bar_strength: f64,
    pub fear: f64,
    pub pain_sensitivity: f64,
    pub v_light: f64,
    pub v_tone: f64,
    pub v_bell: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ExportDataset {
    /// Every recorded event, with the cumulative number of presses.
    Events,
    /// One row per classical conditioning trial.
    Trials,
    /// Periodic samples of the mind windows.
    Associations,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recorder {
    pub events: Vec<RecordEvent>,
    pub trials: Vec<TrialRecord>,
    pub samples: Vec<AssocSample>,
    pub next_sample_at: f64,
    pub presses: u32,
    pub reinforcers: u32,
    pub pellets_eaten: u32,
}

fn opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.4}")).unwrap_or_default()
}

fn csv_text(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

impl Recorder {
    pub fn push(&mut self, event: RecordEvent) {
        self.events.push(event);
    }

    pub fn to_csv(&self, dataset: ExportDataset) -> String {
        match dataset {
            ExportDataset::Events => self.events_csv(),
            ExportDataset::Trials => self.trials_csv(),
            ExportDataset::Associations => self.samples_csv(),
        }
    }

    fn events_csv(&self) -> String {
        let mut out = String::from("time_s,time_min,event,detail,cumulative_presses\n");
        let mut presses = 0u32;
        for e in &self.events {
            let (name, detail) = match e {
                RecordEvent::Press { reinforced, .. } => {
                    presses += 1;
                    (
                        "press",
                        if *reinforced { "reinforced" } else { "" }.to_string(),
                    )
                }
                RecordEvent::Reinforcer { kind, .. } => (
                    "reinforcer",
                    match kind {
                        ReinforcerKind::Food => "food",
                        ReinforcerKind::Sound => "sound only",
                        ReinforcerKind::Manual => "manual pellet",
                    }
                    .to_string(),
                ),
                RecordEvent::Mark { label, .. } => ("mark", label.clone()),
                RecordEvent::CsOn { cs, .. } => ("cs_on", cs.label()),
                RecordEvent::CsOff { .. } => ("cs_off", String::new()),
                RecordEvent::Shock { level, .. } => ("shock", level.label().to_string()),
                RecordEvent::DesignChange { label, .. } => ("design", label.clone()),
                RecordEvent::TimeOff { hours, .. } => ("time_off", format!("{hours} h")),
                RecordEvent::ClassicalStart { .. } => ("classical_start", String::new()),
                RecordEvent::ClassicalEnd { completed, .. } => (
                    "classical_end",
                    if *completed { "completed" } else { "stopped" }.to_string(),
                ),
            };
            let t = e.t();
            let _ = writeln!(
                out,
                "{t:.2},{:.4},{name},{},{presses}",
                t / 60.0,
                csv_text(&detail)
            );
        }
        out
    }

    fn trials_csv(&self) -> String {
        let mut out = String::from(
            "trial,stage,trial_in_stage,time_s,cs,us,pre_cs_presses,cs_presses,suppression_ratio,\
pre_cs_move_s,cs_move_s,movement_ratio,v_light,v_tone,v_bell,fear_peak,pain_sensitivity\n",
        );
        for r in &self.trials {
            let _ = writeln!(
                out,
                "{},{},{},{:.2},{},{},{},{},{},{:.2},{:.2},{},{:.4},{:.4},{:.4},{:.4},{:.4}",
                r.trial,
                r.stage,
                r.trial_in_stage,
                r.t,
                csv_text(&r.cs.label()),
                r.us.label(),
                r.pre_presses,
                r.cs_presses,
                opt(r.suppression_ratio),
                r.pre_move,
                r.cs_move,
                opt(r.movement_ratio),
                r.v_light,
                r.v_tone,
                r.v_bell,
                r.fear_peak,
                r.pain_sensitivity
            );
        }
        out
    }

    fn samples_csv(&self) -> String {
        let mut out = String::from(
            "time_s,sound_food,bar_sound,bar_press_strength,fear,pain_sensitivity,v_light,v_tone,v_bell\n",
        );
        for s in &self.samples {
            let _ = writeln!(
                out,
                "{:.2},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
                s.t,
                s.sound_food,
                s.bar_sound,
                s.bar_strength,
                s.fear,
                s.pain_sensitivity,
                s.v_light,
                s.v_tone,
                s.v_bell
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_csv_counts_presses() {
        let mut r = Recorder::default();
        r.push(RecordEvent::Press {
            t: 1.0,
            reinforced: false,
        });
        r.push(RecordEvent::Press {
            t: 2.0,
            reinforced: true,
        });
        r.push(RecordEvent::Mark {
            t: 3.0,
            label: "a, \"b\"".into(),
        });
        let csv = r.to_csv(ExportDataset::Events);
        let lines: Vec<_> = csv.lines().collect();
        assert_eq!(lines.len(), 4);
        assert!(lines[2].ends_with(",reinforced,2"));
        assert_eq!(lines[3], "3.00,0.0500,mark,\"a, \"\"b\"\"\",2");
    }
}
