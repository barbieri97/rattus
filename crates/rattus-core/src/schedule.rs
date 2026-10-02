//! Reinforcement schedules for bar pressing.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Schedule {
    /// Every press is reinforced (CRF).
    Continuous,
    /// Every `n`th press is reinforced.
    FixedRatio { n: u32 },
    /// On average every `n`th press is reinforced.
    VariableRatio { n: u32 },
    /// The first press after `seconds` since the last reinforcement is reinforced.
    FixedInterval { seconds: f64 },
    /// Like fixed interval, but the interval varies around a mean of `seconds`.
    VariableInterval { seconds: f64 },
}

impl Schedule {
    pub fn is_ratio(&self) -> bool {
        matches!(
            self,
            Schedule::Continuous | Schedule::FixedRatio { .. } | Schedule::VariableRatio { .. }
        )
    }

    pub fn label(&self) -> String {
        match self {
            Schedule::Continuous => "CRF".to_string(),
            Schedule::FixedRatio { n } => format!("FR-{n}"),
            Schedule::VariableRatio { n } => format!("VR-{n}"),
            Schedule::FixedInterval { seconds } => format!("FI-{} s", fmt_secs(*seconds)),
            Schedule::VariableInterval { seconds } => format!("VI-{} s", fmt_secs(*seconds)),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        match self {
            Schedule::Continuous => Ok(()),
            Schedule::FixedRatio { n } | Schedule::VariableRatio { n } => {
                if (1..=500).contains(n) {
                    Ok(())
                } else {
                    Err("The ratio must be between 1 and 500 responses.".into())
                }
            }
            Schedule::FixedInterval { seconds } | Schedule::VariableInterval { seconds } => {
                if seconds.is_finite() && (1.0..=3600.0).contains(seconds) {
                    Ok(())
                } else {
                    Err("The interval must be between 1 and 3600 seconds.".into())
                }
            }
        }
    }
}

fn fmt_secs(s: f64) -> String {
    if (s - s.round()).abs() < 1e-9 {
        format!("{}", s.round() as i64)
    } else {
        format!("{s:.1}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Reinforcer {
    /// The dispenser clicks and delivers a pellet.
    Food,
    /// The dispenser clicks but delivers nothing (tests secondary reinforcement).
    SoundOnly,
    /// Nothing happens (extinction).
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct OperantDesign {
    pub schedule: Schedule,
    pub reinforcer: Reinforcer,
}

impl Default for OperantDesign {
    fn default() -> Self {
        Self {
            schedule: Schedule::Continuous,
            reinforcer: Reinforcer::Food,
        }
    }
}

impl OperantDesign {
    pub fn label(&self) -> String {
        match self.reinforcer {
            Reinforcer::Food => format!("{}, food", self.schedule.label()),
            Reinforcer::SoundOnly => format!("{}, sound only", self.schedule.label()),
            Reinforcer::None => "Extinction".to_string(),
        }
    }
}

/// Progress toward the next reinforcement.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleState {
    pub count: u32,
    pub required: u32,
    pub interval_start: f64,
    pub interval_len: f64,
}

impl ScheduleState {
    pub fn new(schedule: &Schedule, now: f64, rng: &mut Rng) -> Self {
        let mut s = Self::default();
        s.arm(schedule, now, rng);
        s
    }

    /// Sets up the requirement for the next reinforcement.
    pub fn arm(&mut self, schedule: &Schedule, now: f64, rng: &mut Rng) {
        self.count = 0;
        self.interval_start = now;
        match *schedule {
            Schedule::Continuous => {
                self.required = 1;
                self.interval_len = 0.0;
            }
            Schedule::FixedRatio { n } => {
                self.required = n.max(1);
                self.interval_len = 0.0;
            }
            Schedule::VariableRatio { n } => {
                let n = n.max(1);
                self.required = rng.int_inclusive(1, 2 * n - 1);
                self.interval_len = 0.0;
            }
            Schedule::FixedInterval { seconds } => {
                self.required = 1;
                self.interval_len = seconds;
            }
            Schedule::VariableInterval { seconds } => {
                self.required = 1;
                self.interval_len = rng.range(0.0, 2.0 * seconds);
            }
        }
    }

    /// Registers a press and says whether it is reinforced.
    pub fn on_press(&mut self, schedule: &Schedule, now: f64, rng: &mut Rng) -> bool {
        let reinforced = match schedule {
            Schedule::Continuous | Schedule::FixedRatio { .. } | Schedule::VariableRatio { .. } => {
                self.count += 1;
                self.count >= self.required
            }
            Schedule::FixedInterval { .. } | Schedule::VariableInterval { .. } => {
                now - self.interval_start >= self.interval_len
            }
        };
        if reinforced {
            self.arm(schedule, now, rng);
        }
        reinforced
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presses_until_reinforced(state: &mut ScheduleState, s: &Schedule, rng: &mut Rng) -> u32 {
        let mut n = 0;
        loop {
            n += 1;
            if state.on_press(s, 0.0, rng) {
                return n;
            }
        }
    }

    #[test]
    fn fixed_ratio_reinforces_every_nth_press() {
        let mut rng = Rng::new(1);
        let s = Schedule::FixedRatio { n: 5 };
        let mut st = ScheduleState::new(&s, 0.0, &mut rng);
        for _ in 0..10 {
            assert_eq!(presses_until_reinforced(&mut st, &s, &mut rng), 5);
        }
    }

    #[test]
    fn variable_ratio_averages_n() {
        let mut rng = Rng::new(2);
        let s = Schedule::VariableRatio { n: 20 };
        let mut st = ScheduleState::new(&s, 0.0, &mut rng);
        let runs = 5000;
        let total: u32 = (0..runs)
            .map(|_| presses_until_reinforced(&mut st, &s, &mut rng))
            .sum();
        let mean = f64::from(total) / f64::from(runs);
        assert!((mean - 20.0).abs() < 0.8, "mean {mean}");
    }

    #[test]
    fn fixed_interval_needs_time() {
        let mut rng = Rng::new(3);
        let s = Schedule::FixedInterval { seconds: 30.0 };
        let mut st = ScheduleState::new(&s, 0.0, &mut rng);
        assert!(!st.on_press(&s, 10.0, &mut rng));
        assert!(!st.on_press(&s, 29.9, &mut rng));
        assert!(st.on_press(&s, 30.0, &mut rng));
        assert!(!st.on_press(&s, 31.0, &mut rng));
        assert!(st.on_press(&s, 60.5, &mut rng));
    }

    #[test]
    fn variable_interval_averages_mean() {
        let mut rng = Rng::new(4);
        let s = Schedule::VariableInterval { seconds: 30.0 };
        let mut st = ScheduleState::new(&s, 0.0, &mut rng);
        let mut sum = 0.0;
        for _ in 0..4000 {
            sum += st.interval_len;
            st.arm(&s, 0.0, &mut rng);
        }
        assert!((sum / 4000.0 - 30.0).abs() < 1.5);
    }

    #[test]
    fn validation() {
        assert!(Schedule::FixedRatio { n: 0 }.validate().is_err());
        assert!(
            Schedule::VariableInterval { seconds: 0.5 }
                .validate()
                .is_err()
        );
        assert!(Schedule::VariableRatio { n: 25 }.validate().is_ok());
    }
}
