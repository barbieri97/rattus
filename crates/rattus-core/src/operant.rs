//! The rat's operant learning: the associations shown in the Operant Associations window,
//! action strengths for every behaviour, place preferences and what it has learned about
//! the current reinforcement schedule.

use serde::{Deserialize, Serialize};

use crate::assoc::Assoc;
use crate::behavior::{BEHAVIOR_COUNT, Behavior, similarity};
use crate::rat::{HOPPER_X, X_MAX, X_MIN};

pub const PLACE_BINS: usize = 10;

/// Learning rate of action strengths when a behaviour is reinforced.
const ALPHA_ACTION: f64 = 0.2;
/// Learning rate of the bar-sound association when a press produces the click.
const ALPHA_BAR_SOUND: f64 = 0.15;
/// Learning rate of the sound-food association when a pellet is eaten right after the click.
const ALPHA_SOUND_FOOD: f64 = 0.25;
/// Extinction rate of the sound-food association for a click that brings no food.
const BETA_SOUND_FOOD: f64 = 0.04;
/// Learning rate of bar pressing when a press is reinforced.
const ALPHA_PRESS: f64 = 0.2;
/// Extinction rate of bar pressing for an unreinforced press, scaled by surprise.
const BETA_BAR: f64 = 0.1;
/// Extinction rate of other shaped behaviours when they go unreinforced.
const BETA_FREE: f64 = 0.03;
/// Baseline surprise of an unreinforced press, so even very lean schedules extinguish.
const SURPRISE_FLOOR: f64 = 0.01;
/// How quickly the long-term expectation of reinforcement per press follows experience.
const P_LONG_RATE: f64 = 0.01;
/// How quickly the recent rate of reinforcement per press follows experience.
const P_RECENT_RATE: f64 = 0.15;
/// Time constant (s) of the delay between the click and eating in magazine training.
pub const CONTIGUITY_TAU: f64 = 8.0;
/// How fast the rat adapts to a new schedule (weight of each reinforcement in the averages).
const SCHEDULE_ADAPT: f64 = 0.25;
/// Post-reinforcement pause per response of a predictable ratio, in seconds.
const PAUSE_PER_RESPONSE: f64 = 0.3;
/// Place preferences fade with this time constant (s).
const PLACE_TAU: f64 = 900.0;
const FRUSTRATION_TAU: f64 = 20.0;

/// Exponential moving average with variance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ema {
    pub mean: f64,
    pub var: f64,
    pub n: u32,
}

impl Ema {
    pub fn seeded(mean: f64, sd: f64) -> Self {
        Self {
            mean,
            var: sd * sd,
            n: 10,
        }
    }

    pub fn update(&mut self, x: f64, weight: f64) {
        if self.n == 0 {
            self.mean = x;
            self.var = 0.0;
        } else {
            let d = x - self.mean;
            self.mean += weight * d;
            self.var = (1.0 - weight) * (self.var + weight * d * d);
        }
        self.n = self.n.saturating_add(1);
    }

    /// Coefficient of variation. Unknown (few samples) counts as unpredictable.
    pub fn cv(&self) -> f64 {
        if self.n < 3 || self.mean <= 1e-6 {
            1.0
        } else {
            self.var.max(0.0).sqrt() / self.mean
        }
    }

    /// 1 when perfectly regular, 0 when as irregular as a variable schedule.
    pub fn predictability(&self) -> f64 {
        (1.0 - self.cv() / 0.5).clamp(0.0, 1.0)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingSound {
    pub t: f64,
    pub food: bool,
    /// The rat went to the food cup after this click.
    pub checked: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperantMind {
    /// The dispenser click predicts food (built by magazine training).
    pub sound_food: Assoc,
    /// Pressing the bar produces the click.
    pub bar_sound: Assoc,
    /// Action strength of every behaviour, indexed by `Behavior::index`.
    pub actions: Vec<Assoc>,
    /// Preference for regions of the chamber, left to right.
    pub place: Vec<f64>,
    /// Long-term expected probability that a press is reinforced.
    pub p_reinf: f64,
    /// Recent probability that a press was reinforced.
    pub p_recent: f64,
    /// Responses per scheduled reinforcement.
    pub resp_per_rf: Ema,
    /// Seconds between scheduled reinforcements.
    pub secs_per_rf: Ema,
    /// 1 when reinforcement seems to depend on the number of responses, 0 on elapsed time.
    pub ratio_bias: f64,
    pub last_rf_t: Option<f64>,
    pub last_food_t: Option<f64>,
    pub presses_since_rf: u32,
    pub pending_sounds: Vec<PendingSound>,
    /// Rises when expected reinforcement fails to come; produces the extinction burst.
    pub frustration: f64,
    /// The rat heard the click and will go to the food cup.
    pub approach_pending: bool,
}

impl Default for OperantMind {
    fn default() -> Self {
        Self {
            sound_food: Assoc::default(),
            bar_sound: Assoc::default(),
            actions: vec![Assoc::default(); BEHAVIOR_COUNT],
            place: vec![0.0; PLACE_BINS],
            p_reinf: 0.5,
            p_recent: 0.5,
            resp_per_rf: Ema::default(),
            secs_per_rf: Ema::default(),
            ratio_bias: 1.0,
            last_rf_t: None,
            last_food_t: None,
            presses_since_rf: 0,
            pending_sounds: Vec::new(),
            frustration: 0.0,
            approach_pending: false,
        }
    }
}

impl OperantMind {
    /// Repairs vectors with a wrong length (e.g. from a hand-edited file).
    pub fn sanitize(&mut self) {
        self.actions.resize(BEHAVIOR_COUNT, Assoc::default());
        self.place.resize(PLACE_BINS, 0.0);
    }

    pub fn action(&self, b: Behavior) -> Assoc {
        self.actions[b.index()]
    }

    pub fn action_mut(&mut self, b: Behavior) -> &mut Assoc {
        &mut self.actions[b.index()]
    }

    /// Value of the click as a conditioned reinforcer.
    pub fn sound_value(&self) -> f64 {
        self.sound_food.level()
    }

    /// How strongly the rat is inclined to press the bar, before schedule and fear effects.
    pub fn bar_drive(&self) -> f64 {
        self.action(Behavior::BarPress).level() * (0.4 + 0.6 * self.bar_sound.level())
    }

    /// Multiplier on bar pressing from what the rat has learned about the schedule:
    /// a pause after reinforcement on predictable ratios and the scallop on fixed intervals.
    pub fn schedule_modulation(&self, now: f64) -> f64 {
        let Some(last) = self.last_rf_t else {
            return 1.0;
        };
        let since = (now - last).max(0.0);

        let pause = PAUSE_PER_RESPONSE * self.resp_per_rf.mean * self.resp_per_rf.predictability();
        let ratio = if pause < 0.5 || since >= pause {
            1.0
        } else {
            0.03 + 0.97 * (since / pause).powi(3)
        };

        let predictable = self.secs_per_rf.predictability();
        let mean_t = self.secs_per_rf.mean.max(1.0);
        let scallop = (since / mean_t).clamp(0.0, 1.0).powf(2.0).max(0.03);
        let interval = 1.0 - predictable + predictable * scallop;

        self.ratio_bias * ratio + (1.0 - self.ratio_bias) * interval
    }

    /// A press produced the click.
    pub fn reinforced_press(&mut self) {
        let value = self.sound_value();
        self.bar_sound.learn(ALPHA_BAR_SOUND, 1.0);
        self.action_mut(Behavior::BarPress)
            .learn(ALPHA_PRESS, value);
        self.generalize(Behavior::BarPress, 1.0, value);
        self.p_reinf += P_LONG_RATE * (1.0 - self.p_reinf);
        self.p_recent += P_RECENT_RATE * (1.0 - self.p_recent);
        self.frustration *= 0.5;
    }

    /// How surprising it is that a press went unreinforced: reinforcement used to come more
    /// often than it does now, or it has not come for much longer than usual. After lean
    /// schedules little is surprising, so extinction is slower (partial reinforcement effect).
    pub fn nonreinforcement_surprise(&self, now: f64) -> f64 {
        let drop = (self.p_reinf - self.p_recent).max(0.0);
        let overdue = match self.last_rf_t {
            Some(last) => {
                let usual = if self.secs_per_rf.n > 0 {
                    self.secs_per_rf.mean.max(5.0)
                } else {
                    30.0
                };
                (((now - last) / usual - 2.0) / 10.0).clamp(0.0, 1.0)
            }
            None => 0.0,
        };
        drop + SURPRISE_FLOOR + 0.05 * overdue
    }

    /// A press produced nothing.
    pub fn unreinforced_press(&mut self, now: f64) {
        let surprise = self.nonreinforcement_surprise(now);
        self.bar_sound.learn(BETA_BAR * surprise, 0.0);
        self.action_mut(Behavior::BarPress)
            .learn(BETA_BAR * surprise, 0.0);
        let drop = (self.p_reinf - self.p_recent).max(0.0);
        self.frustration = (self.frustration + 0.3 * drop).min(1.0);
        self.p_reinf += P_LONG_RATE * (0.0 - self.p_reinf);
        self.p_recent += P_RECENT_RATE * (0.0 - self.p_recent);
    }

    /// Updates what the rat knows about the schedule after a scheduled reinforcement.
    pub fn on_scheduled_reinforcement(&mut self, now: f64, is_ratio: bool) {
        self.resp_per_rf
            .update(f64::from(self.presses_since_rf.max(1)), SCHEDULE_ADAPT);
        if let Some(last) = self.last_rf_t {
            self.secs_per_rf.update(now - last, SCHEDULE_ADAPT);
        }
        self.last_rf_t = Some(now);
        self.presses_since_rf = 0;
        let target = if is_ratio { 1.0 } else { 0.0 };
        self.ratio_bias += SCHEDULE_ADAPT * (target - self.ratio_bias);
    }

    /// Reinforces a behaviour (with the given eligibility weight) and, more weakly,
    /// the behaviours similar to it.
    pub fn credit(&mut self, b: Behavior, weight: f64) {
        let value = self.sound_value();
        self.action_mut(b).learn(ALPHA_ACTION * weight, value);
        self.generalize(b, weight, value);
    }

    fn generalize(&mut self, b: Behavior, weight: f64, value: f64) {
        for other in Behavior::ALL {
            if other == b {
                continue;
            }
            let s = similarity(b, other);
            if s <= 0.0 {
                continue;
            }
            let target = value * s;
            let a = self.action_mut(other);
            if target > a.net() {
                a.learn(ALPHA_ACTION * weight * s, target);
            }
        }
    }

    /// A shaped behaviour was performed and not reinforced.
    pub fn unreinforced_free(&mut self, b: Behavior) {
        if self.action(b).net() > 0.0 {
            self.action_mut(b).learn(BETA_FREE, 0.0);
        }
    }

    pub fn place_bin(x: f64) -> usize {
        let f = ((x - X_MIN) / (X_MAX - X_MIN)).clamp(0.0, 0.9999);
        (f * PLACE_BINS as f64) as usize
    }

    /// Reinforcement makes the rat prefer the place where it happened.
    pub fn reinforce_place(&mut self, x: f64) {
        let value = self.sound_value();
        let bin = Self::place_bin(x);
        for (i, p) in self.place.iter_mut().enumerate() {
            let w = match bin.abs_diff(i) {
                0 => 1.0,
                1 => 0.5,
                _ => continue,
            };
            *p += 0.15 * w * (value - *p).max(0.0);
        }
    }

    /// Eating makes the area around the food cup attractive.
    pub fn food_place(&mut self) {
        let bin = Self::place_bin(HOPPER_X);
        self.place[bin] += 0.04 * (1.0 - self.place[bin]);
    }

    /// The rat ate a pellet: the click that announced it gains value, the more so the shorter
    /// the delay. Returns the delay, if a click announced this pellet.
    pub fn ate_pellet(&mut self, now: f64) -> Option<f64> {
        self.last_food_t = Some(now);
        self.food_place();
        let i = self.pending_sounds.iter().position(|s| s.food)?;
        let sound = self.pending_sounds.remove(i);
        let delay = now - sound.t;
        let contiguity = (-delay / CONTIGUITY_TAU).exp();
        self.sound_food.learn(ALPHA_SOUND_FOOD * contiguity, 1.0);
        Some(delay)
    }

    /// The rat looked into the food cup: any click it was answering is now known to be empty.
    pub fn checked_cup(&mut self) {
        for s in &mut self.pending_sounds {
            if !s.food {
                s.checked = true;
            }
        }
    }

    /// Clicks after which the rat found the cup empty extinguish the sound-food association.
    /// Clicks it ignored teach nothing, so a weakening sound keeps some value for a long time
    /// (secondary reinforcement).
    pub fn expire_sounds(&mut self, now: f64, expiry: f64) {
        let mut extinguished = 0;
        self.pending_sounds.retain(|s| {
            if s.food {
                // Uneaten pellets stop counting after two minutes.
                now - s.t < 120.0
            } else if now - s.t >= expiry {
                if s.checked {
                    extinguished += 1;
                }
                false
            } else {
                true
            }
        });
        for _ in 0..extinguished {
            self.sound_food.learn(BETA_SOUND_FOOD, 0.0);
        }
    }

    pub fn decay(&mut self, dt: f64) {
        let place_keep = (-dt / PLACE_TAU).exp();
        for p in &mut self.place {
            *p *= place_keep;
        }
        self.frustration *= (-dt / FRUSTRATION_TAU).exp();
    }

    /// Time away from the chamber: extinction (inhibition) fades.
    pub fn rest(&mut self, hours: f64) {
        const HALF_LIFE: f64 = 24.0;
        self.sound_food.rest(hours, HALF_LIFE);
        self.bar_sound.rest(hours, HALF_LIFE);
        for a in &mut self.actions {
            a.rest(hours, HALF_LIFE);
        }
        self.frustration = 0.0;
        self.pending_sounds.clear();
        self.approach_pending = false;
        self.presses_since_rf = 0;
        self.last_rf_t = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ema_tracks_regular_and_irregular_series() {
        let mut fixed = Ema::default();
        let mut variable = Ema::default();
        let mut rng = crate::rng::Rng::new(5);
        for _ in 0..60 {
            fixed.update(20.0, 0.25);
            variable.update(f64::from(rng.int_inclusive(1, 39)), 0.25);
        }
        assert!(fixed.predictability() > 0.99);
        assert!(variable.predictability() < 0.3);
    }

    #[test]
    fn eating_soon_after_the_click_builds_sound_food() {
        let mut quick = OperantMind::default();
        let mut slow = OperantMind::default();
        for i in 0..10 {
            let t = f64::from(i) * 60.0;
            quick.pending_sounds.push(PendingSound {
                t,
                food: true,
                checked: false,
            });
            quick.ate_pellet(t + 1.0);
            slow.pending_sounds.push(PendingSound {
                t,
                food: true,
                checked: false,
            });
            slow.ate_pellet(t + 30.0);
        }
        assert!(quick.sound_food.net() > 0.8);
        assert!(slow.sound_food.net() < 0.2);
    }

    #[test]
    fn partial_reinforcement_slows_extinction() {
        let mut crf = OperantMind::default();
        let mut lean = OperantMind::default();
        crf.p_reinf = 1.0;
        crf.p_recent = 1.0;
        lean.p_reinf = 0.04;
        lean.p_recent = 0.04;
        for m in [&mut crf, &mut lean] {
            *m.action_mut(Behavior::BarPress) = Assoc::with_strength(1.0);
            m.bar_sound = Assoc::with_strength(1.0);
        }
        for _ in 0..30 {
            crf.unreinforced_press(0.0);
            lean.unreinforced_press(0.0);
        }
        assert!(crf.bar_drive() < lean.bar_drive());
    }

    #[test]
    fn fixed_ratio_produces_a_pause() {
        let mut m = OperantMind::default();
        for i in 0..20 {
            m.presses_since_rf = 50;
            m.on_scheduled_reinforcement(f64::from(i) * 40.0, true);
        }
        let last = m.last_rf_t.unwrap();
        assert!(m.schedule_modulation(last + 1.0) < 0.1);
        assert!(m.schedule_modulation(last + 20.0) > 0.99);
    }

    #[test]
    fn fixed_interval_produces_a_scallop() {
        let mut m = OperantMind::default();
        for i in 0..20 {
            m.presses_since_rf = 10;
            m.on_scheduled_reinforcement(f64::from(i) * 60.0, false);
        }
        let last = m.last_rf_t.unwrap();
        let early = m.schedule_modulation(last + 10.0);
        let late = m.schedule_modulation(last + 55.0);
        assert!(early < 0.1 && late > 0.7, "early {early} late {late}");
    }
}
