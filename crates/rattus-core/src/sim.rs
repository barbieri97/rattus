//! The simulation: one rat in one operant chamber, advanced in fixed time steps.

use serde::{Deserialize, Serialize};

use crate::behavior::Behavior;
use crate::chamber::{Chamber, MAX_PELLETS};
use crate::classical::{
    CS_DURATION, ClassicalDesign, ClassicalMind, ClassicalRun, CsKind, Phase, SHOCK_DURATION,
    UsLevel, ratio,
};
use crate::operant::{OperantMind, PLACE_BINS};
use crate::protocol::{
    ActionStrength, BehaviorStart, ChamberView, ClassicalProgress, Command, Counters, Delta,
    FullState, HostStatus, MindView, RatTemplate, RatView, Snapshot,
};
use crate::rat::{
    Bout, DEVICE_Z, Facing, HOPPER_X, LEVER_X, Rat, SPOUT_X, WALK_SPEED, X_MAX, X_MIN,
};
use crate::recorder::{
    AssocSample, ExportDataset, RecordEvent, Recorder, ReinforcerKind, TrialRecord,
};
use crate::rng::Rng;
use crate::schedule::{OperantDesign, Reinforcer, ScheduleState};

/// Length of one simulation step, in seconds of program time.
pub const DT: f64 = 0.05;
const SAMPLE_EVERY: f64 = 10.0;
/// Behaviours that ended this long before a reinforcement still get some credit.
const ELIGIBILITY_WINDOW: f64 = 2.0;
/// A click without food extinguishes the sound-food association after this delay.
const SOUND_EXPIRY: f64 = 8.0;
/// Extra choice weight per unit of action strength.
const ACTION_GAIN: f64 = 40.0;
/// Extra weight per unit of place preference when choosing where to walk.
const PLACE_GAIN: f64 = 25.0;
const Z_SPEED: f64 = 0.35;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RecentBout {
    behavior: Behavior,
    ended: f64,
    credited: bool,
}

#[derive(Clone, Copy, Debug, Default)]
struct Cursor {
    events: usize,
    trials: usize,
    samples: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    Manual,
    Schedule,
}

#[derive(Clone, Copy)]
enum Choice {
    Do(Behavior),
    Explore,
    GoHopper,
    GoDrink,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Simulation {
    /// Program time in seconds.
    pub(crate) t: f64,
    pub(crate) rng: Rng,
    pub(crate) rat: Rat,
    pub(crate) chamber: Chamber,
    pub(crate) op: OperantMind,
    pub(crate) cl: ClassicalMind,
    pub(crate) operant_design: OperantDesign,
    pub(crate) schedule: ScheduleState,
    pub(crate) classical_design: ClassicalDesign,
    pub(crate) classical_run: Option<ClassicalRun>,
    pub(crate) recorder: Recorder,
    recent: Vec<RecentBout>,
    last_press_t: f64,
    #[serde(skip)]
    cursor: Cursor,
    #[serde(skip)]
    behavior_starts: Vec<BehaviorStart>,
}

impl Simulation {
    pub(crate) fn assemble(rng: Rng, rat: Rat, op: OperantMind, design: OperantDesign) -> Self {
        let mut rng = rng;
        let schedule = ScheduleState::new(&design.schedule, 0.0, &mut rng);
        let mut sim = Self {
            t: 0.0,
            rng,
            rat,
            chamber: Chamber::default(),
            op,
            cl: ClassicalMind::default(),
            operant_design: design,
            schedule,
            classical_design: ClassicalDesign::default(),
            classical_run: None,
            recorder: Recorder::default(),
            recent: Vec::new(),
            last_press_t: f64::NEG_INFINITY,
            cursor: Cursor::default(),
            behavior_starts: Vec::new(),
        };
        sim.recorder.push(RecordEvent::DesignChange {
            t: 0.0,
            label: design.label(),
        });
        sim.sample();
        sim
    }

    pub fn new(template: RatTemplate, seed: u64) -> Self {
        match template {
            RatTemplate::Naive => Self::naive(seed),
            RatTemplate::BarTrained => Self::bar_trained(seed),
        }
    }

    // ---------------------------------------------------------------- public API

    pub fn time(&self) -> f64 {
        self.t
    }

    pub fn recorder(&self) -> &Recorder {
        &self.recorder
    }

    pub fn operant_design(&self) -> OperantDesign {
        self.operant_design
    }

    pub fn classical_design(&self) -> &ClassicalDesign {
        &self.classical_design
    }

    pub fn classical_running(&self) -> bool {
        self.classical_run.is_some()
    }

    pub fn operant_mind(&self) -> &OperantMind {
        &self.op
    }

    pub fn classical_mind(&self) -> &ClassicalMind {
        &self.cl
    }

    pub fn current_behavior(&self) -> Behavior {
        self.rat.bout.behavior
    }

    /// Advances the simulation by `seconds` of program time.
    pub fn advance(&mut self, seconds: f64) {
        let steps = (seconds / DT).round().max(0.0) as u64;
        for _ in 0..steps {
            self.step();
        }
    }

    pub fn step(&mut self) {
        self.t += DT;
        self.tick_classical();
        let target = self.cs_fear_target();
        self.cl.update_fear(DT, target);
        if let Some(run) = &mut self.classical_run
            && run.in_cs()
        {
            run.fear_peak = run.fear_peak.max(self.cl.fear);
        }
        self.op.decay(DT);
        self.op.expire_sounds(self.t, SOUND_EXPIRY);
        if self.chamber.shock.is_some() && self.t >= self.chamber.shock_until {
            self.chamber.shock = None;
        }
        self.process_recent();
        self.tick_rat();
        self.accumulate_windows();
        if self.t >= self.recorder.next_sample_at {
            self.sample();
        }
    }

    pub fn apply(&mut self, command: Command) -> Result<(), String> {
        match command {
            Command::GivePellet => {
                self.deliver(Source::Manual);
                Ok(())
            }
            Command::MarkRecord { label } => {
                let label = label
                    .map(|l| l.trim().chars().take(60).collect::<String>())
                    .filter(|l| !l.is_empty())
                    .unwrap_or_else(|| "Mark".to_string());
                self.recorder.push(RecordEvent::Mark { t: self.t, label });
                Ok(())
            }
            Command::SetOperantDesign { design } => {
                design.schedule.validate()?;
                self.operant_design = design;
                self.schedule.arm(&design.schedule, self.t, &mut self.rng);
                self.recorder.push(RecordEvent::DesignChange {
                    t: self.t,
                    label: design.label(),
                });
                Ok(())
            }
            Command::SetClassicalDesign { design } => {
                design.validate()?;
                self.classical_design = design;
                Ok(())
            }
            Command::StartClassical => self.start_classical(),
            Command::StopClassical => {
                if self.classical_run.is_some() {
                    self.end_classical(false);
                }
                Ok(())
            }
            Command::TimeOff { hours } => {
                if !hours.is_finite() || !(0.1..=720.0).contains(&hours) {
                    return Err("Time off must be between 0.1 and 720 hours.".into());
                }
                self.time_off(hours);
                Ok(())
            }
            Command::NewRat { template, seed } => {
                *self = Self::new(template, seed.abs() as u64);
                Ok(())
            }
            Command::RandomizePosition => {
                self.place_near_center();
                Ok(())
            }
        }
    }

    /// Puts the rat at a random spot near the middle of the chamber.
    pub fn place_near_center(&mut self) {
        let x = self.rng.range(0.35, 0.6);
        let z = self.rng.range(0.3, 0.7);
        self.rat.x = x;
        self.rat.z = z;
        self.rat.facing = if self.rng.chance(0.5) {
            Facing::Left
        } else {
            Facing::Right
        };
        self.op.approach_pending = false;
        self.start(Behavior::LookAround);
    }

    pub fn snapshot(&self) -> Snapshot {
        let bout = &self.rat.bout;
        Snapshot {
            t: self.t,
            rat: RatView {
                x: self.rat.x,
                z: self.rat.z,
                facing: self.rat.facing,
                behavior: bout.behavior,
                elapsed: bout.elapsed,
                duration: bout.duration,
            },
            chamber: ChamberView {
                pellets: self.chamber.pellets,
                lever_down: self.t < self.chamber.lever_until,
                light_on: self.chamber.light_on,
                tone_db: self.chamber.tone_db,
                bell_on: self.chamber.bell_on,
                shock: self.chamber.shock,
                dispense_count: self.chamber.dispense_count,
                shock_count: self.chamber.shock_count,
            },
            mind: MindView {
                sound_food: self.op.sound_food.level(),
                bar_sound: self.op.bar_sound.level(),
                bar_strength: self.op.action(Behavior::BarPress).level(),
                fear: self.cl.fear,
                pain_sensitivity: self.cl.pain,
                v_light: self.cl.strength(CsKind::Light),
                v_tone: self.cl.strength(CsKind::Tone),
                v_bell: self.cl.strength(CsKind::Bell),
                actions: Behavior::ALL
                    .iter()
                    .filter(|b| b.is_shapeable() || **b == Behavior::BarPress)
                    .map(|b| ActionStrength {
                        behavior: *b,
                        value: self.op.action(*b).level(),
                    })
                    .collect(),
            },
            counters: Counters {
                presses: self.recorder.presses,
                reinforcers: self.recorder.reinforcers,
                pellets_eaten: self.recorder.pellets_eaten,
            },
            operant: self.operant_design,
            classical: self.classical_progress(),
        }
    }

    /// Data recorded since the previous call.
    pub fn drain(&mut self) -> Delta {
        let r = &self.recorder;
        let delta = Delta {
            events: r.events[self.cursor.events.min(r.events.len())..].to_vec(),
            trials: r.trials[self.cursor.trials.min(r.trials.len())..].to_vec(),
            samples: r.samples[self.cursor.samples.min(r.samples.len())..].to_vec(),
            behaviors: std::mem::take(&mut self.behavior_starts),
        };
        self.mark_drained();
        delta
    }

    /// Everything for a fresh interface; also resets the drain cursor.
    pub fn full_state(&mut self, host: HostStatus) -> FullState {
        self.mark_drained();
        self.behavior_starts.clear();
        FullState {
            snapshot: self.snapshot(),
            host,
            events: self.recorder.events.clone(),
            trials: self.recorder.trials.clone(),
            samples: self.recorder.samples.clone(),
            classical_design: self.classical_design.clone(),
        }
    }

    pub fn export_csv(&self, dataset: ExportDataset) -> String {
        self.recorder.to_csv(dataset)
    }

    /// Repairs state loaded from a file.
    pub(crate) fn sanitize(&mut self) {
        self.op.sanitize();
        self.cursor = Cursor::default();
        self.behavior_starts.clear();
    }

    // ---------------------------------------------------------------- internals

    fn mark_drained(&mut self) {
        self.cursor = Cursor {
            events: self.recorder.events.len(),
            trials: self.recorder.trials.len(),
            samples: self.recorder.samples.len(),
        };
    }

    fn sample(&mut self) {
        self.recorder.samples.push(AssocSample {
            t: self.t,
            sound_food: self.op.sound_food.level(),
            bar_sound: self.op.bar_sound.level(),
            bar_strength: self.op.action(Behavior::BarPress).level(),
            fear: self.cl.fear,
            pain_sensitivity: self.cl.pain,
            v_light: self.cl.strength(CsKind::Light),
            v_tone: self.cl.strength(CsKind::Tone),
            v_bell: self.cl.strength(CsKind::Bell),
        });
        self.recorder.next_sample_at = self.t + SAMPLE_EVERY;
    }

    fn classical_progress(&self) -> Option<ClassicalProgress> {
        let run = self.classical_run.as_ref()?;
        let stage = &run.design.stages[run.stage];
        let remaining = match run.phase {
            Phase::Iti { cs_at } => cs_at - self.t,
            Phase::Cs { start, .. } => start + CS_DURATION - self.t,
        };
        Some(ClassicalProgress {
            stage: run.stage as u32 + 1,
            stage_count: run.design.stages.len() as u32,
            trial_in_stage: run.pos as u32 + 1,
            trials_in_stage: stage.trial_count(),
            total_trial: run.total_trial + 1,
            total_trials: run.design.total_trials(),
            in_cs: run.in_cs(),
            remaining: remaining.max(0.0),
        })
    }

    // --- rat activity

    fn start(&mut self, behavior: Behavior) {
        let mut behavior = behavior;
        if behavior == Behavior::Eat && self.chamber.pellets == 0 {
            // Nothing in the cup: search it briefly.
            self.op.checked_cup();
            behavior = Behavior::Sniff;
        }
        let (lo, hi) = behavior.duration_range();
        let duration = self.rng.range(lo, hi);
        self.start_with_duration(behavior, duration);
    }

    fn start_with_duration(&mut self, behavior: Behavior, duration: f64) {
        self.rat.bout = Bout::new(behavior, duration, self.rat.x, self.rat.z);
        self.behavior_starts.push(BehaviorStart {
            t: self.t,
            behavior,
        });
    }

    /// Walks to `(x, z)` and then starts `then`, if given.
    fn walk_to(&mut self, x: f64, z: f64, then: Option<Behavior>) {
        let x = x.clamp(X_MIN, X_MAX);
        let z = z.clamp(0.0, 1.0);
        let dx = x - self.rat.x;
        if dx.abs() < 0.01 && (z - self.rat.z).abs() < 0.02 {
            if let Some(next) = then {
                self.face_device_at(x);
                self.start(next);
            } else {
                self.start(Behavior::LookAround);
            }
            return;
        }
        if dx > 0.0 {
            self.rat.facing = Facing::Right;
        } else if dx < 0.0 {
            self.rat.facing = Facing::Left;
        }
        let duration = (dx.abs() / WALK_SPEED)
            .max((z - self.rat.z).abs() / Z_SPEED)
            .max(0.3);
        self.rat.bout = Bout::new(Behavior::Walk, duration, x, z);
        self.rat.bout.then = then;
        self.behavior_starts.push(BehaviorStart {
            t: self.t,
            behavior: Behavior::Walk,
        });
    }

    /// At the lever the rat faces right, at the spout left.
    fn face_device_at(&mut self, x: f64) {
        if x >= LEVER_X - 0.04 {
            self.rat.facing = Facing::Right;
        } else if x <= SPOUT_X + 0.04 {
            self.rat.facing = Facing::Left;
        }
    }

    fn tick_rat(&mut self) {
        self.rat.bout.elapsed += DT;
        let behavior = self.rat.bout.behavior;
        match behavior {
            Behavior::Walk => {
                let (tx, tz) = (self.rat.bout.target_x, self.rat.bout.target_z);
                let step = WALK_SPEED * DT;
                let dx = tx - self.rat.x;
                self.rat.x = if dx.abs() <= step {
                    tx
                } else {
                    self.rat.x + step * dx.signum()
                };
                let zstep = Z_SPEED * DT;
                let dz = tz - self.rat.z;
                self.rat.z = if dz.abs() <= zstep {
                    tz
                } else {
                    self.rat.z + zstep * dz.signum()
                };
                let arrived = (self.rat.x - tx).abs() < 1e-9 && (self.rat.z - tz).abs() < 1e-9;
                if arrived || self.rat.bout.elapsed > self.rat.bout.duration + 2.0 {
                    self.end_bout();
                }
            }
            Behavior::BarPress => {
                if !self.rat.bout.acted && self.rat.bout.elapsed >= 0.45 * self.rat.bout.duration {
                    self.rat.bout.acted = true;
                    self.register_press();
                }
                if self.rat.bout.elapsed >= self.rat.bout.duration {
                    self.end_bout();
                }
            }
            _ => {
                if self.rat.bout.elapsed >= self.rat.bout.duration {
                    self.end_bout();
                }
            }
        }
    }

    fn end_bout(&mut self) {
        let bout = self.rat.bout.clone();
        if bout.behavior.is_shapeable() {
            self.recent.push(RecentBout {
                behavior: bout.behavior,
                ended: self.t,
                credited: bout.credited,
            });
        }
        if bout.behavior == Behavior::Eat {
            self.eat_pellet();
        }
        if bout.behavior == Behavior::Walk
            && let Some(next) = bout.then
        {
            self.face_device_at(self.rat.x);
            self.start(next);
            return;
        }
        self.choose_next(Some(bout.behavior));
    }

    /// Stops the current bout early (it still counts for reinforcement credit).
    fn interrupt(&mut self) {
        let bout = &self.rat.bout;
        if bout.behavior.is_shapeable() && bout.elapsed > 0.3 {
            self.recent.push(RecentBout {
                behavior: bout.behavior,
                ended: self.t,
                credited: bout.credited,
            });
        }
    }

    fn eat_pellet(&mut self) {
        if self.chamber.pellets == 0 {
            return;
        }
        self.chamber.pellets -= 1;
        self.recorder.pellets_eaten += 1;
        self.op.ate_pellet(self.t);
    }

    /// Picks the next behaviour.
    fn choose_next(&mut self, prev: Option<Behavior>) {
        let at_lever = self.rat.at_lever();

        // Heard the dispenser: go to the food cup.
        if self.op.approach_pending {
            self.op.approach_pending = false;
            if at_lever {
                self.start(Behavior::Eat);
            } else {
                self.walk_to(HOPPER_X, DEVICE_Z, Some(Behavior::Eat));
            }
            return;
        }

        if self.chamber.pellets > 0 && at_lever && self.rng.chance(0.85) {
            self.start(Behavior::Eat);
            return;
        }

        if self.rng.chance(self.cl.fear * 0.9) {
            self.start(Behavior::Freeze);
            return;
        }

        let tendency = self.press_tendency();
        let rb = self.op.ratio_bias;
        if prev == Some(Behavior::BarPress) && at_lever {
            if self.rng.chance(tendency * (0.55 + 0.42 * rb)) {
                self.start(Behavior::BarPress);
                return;
            }
        } else if self.rng.chance(tendency * (0.45 + 0.35 * rb)) {
            if at_lever {
                self.start(Behavior::BarPress);
            } else {
                self.walk_to(LEVER_X, DEVICE_Z, Some(Behavior::BarPress));
            }
            return;
        }

        self.choose_free(prev, at_lever);
    }

    /// Probability-like tendency to press the bar right now.
    fn press_tendency(&self) -> f64 {
        let drive = self.op.bar_drive();
        if drive <= 0.0 {
            return 0.0;
        }
        // Sigmoid: a weak habit produces only occasional presses, a strong one steady pressing.
        let base = drive * drive / (drive * drive + 0.08);
        let schedule = self.op.schedule_modulation(self.t);
        let fear = (1.0 - self.cl.fear).powi(2);
        let frustration = 1.0 + 0.4 * self.op.frustration;
        (base * schedule * fear * frustration).clamp(0.0, 0.98)
    }

    fn weight(&self, b: Behavior, base: f64) -> f64 {
        base + ACTION_GAIN * self.op.action(b).level()
    }

    fn choose_free(&mut self, prev: Option<Behavior>, at_lever: bool) {
        let at_spout = self.rat.at_spout();
        let active = 1.0 - 0.7 * self.cl.fear;
        // Adjunctive drinking: more drinking shortly after reinforcement on intermittent schedules.
        let adjunctive = match self.op.last_rf_t {
            Some(last) if self.t - last < 30.0 && self.op.resp_per_rf.mean > 1.5 => 3.0,
            _ => 1.0,
        };

        let mut items: Vec<(Choice, f64)> = vec![
            (Choice::Explore, self.weight(Behavior::Walk, 22.0) * active),
            (
                Choice::Do(Behavior::Sniff),
                self.weight(Behavior::Sniff, 16.0) * active,
            ),
            (
                Choice::Do(Behavior::LookAround),
                self.weight(Behavior::LookAround, 7.0),
            ),
            (
                Choice::Do(Behavior::Rear),
                self.weight(Behavior::Rear, 6.0) * active,
            ),
            (
                Choice::Do(Behavior::Groom),
                self.weight(Behavior::Groom, 6.0) * active,
            ),
            (
                Choice::Do(Behavior::FaceWipe),
                self.weight(Behavior::FaceWipe, 3.0) * active,
            ),
            (
                Choice::Do(Behavior::Scratch),
                self.weight(Behavior::Scratch, 3.0) * active,
            ),
            (
                Choice::Do(Behavior::Beg),
                self.weight(Behavior::Beg, 0.4) * active,
            ),
            (
                Choice::Do(Behavior::Roll),
                self.weight(Behavior::Roll, 0.25) * active,
            ),
        ];
        if self.rat.at_wall() {
            items.push((
                Choice::Do(Behavior::RearWall),
                self.weight(Behavior::RearWall, 8.0) * active,
            ));
        }
        if at_lever {
            // Operant level: the lever sometimes gets pressed while exploring.
            items.push((Choice::Do(Behavior::BarPress), 0.5 * active));
        }
        if at_spout {
            items.push((
                Choice::Do(Behavior::Drink),
                self.weight(Behavior::Drink, 6.0) * adjunctive * active,
            ));
        } else {
            items.push((
                Choice::GoDrink,
                (2.5 + ACTION_GAIN * 0.5 * self.op.action(Behavior::Drink).level())
                    * adjunctive
                    * active,
            ));
        }
        if self.chamber.pellets > 0 && !at_lever {
            items.push((Choice::GoHopper, 15.0));
        }
        // Avoid repeating one-off displays back to back.
        if let Some(p) = prev {
            for (choice, w) in &mut items {
                if let Choice::Do(b) = choice
                    && *b == p
                    && matches!(p, Behavior::Roll | Behavior::Startle | Behavior::Orient)
                {
                    *w = 0.0;
                }
            }
        }

        match self
            .rng
            .pick_weighted(&items)
            .unwrap_or(Choice::Do(Behavior::LookAround))
        {
            Choice::Do(b) => self.start(b),
            Choice::Explore => {
                let (x, z) = self.explore_target();
                self.walk_to(x, z, None);
            }
            Choice::GoHopper => self.walk_to(HOPPER_X, DEVICE_Z, Some(Behavior::Eat)),
            Choice::GoDrink => self.walk_to(SPOUT_X, DEVICE_Z, Some(Behavior::Drink)),
        }
    }

    /// Where to walk when exploring: anywhere, but preferably where reinforcement happened.
    fn explore_target(&mut self) -> (f64, f64) {
        let weights: Vec<(usize, f64)> = (0..PLACE_BINS)
            .map(|i| (i, 1.0 + PLACE_GAIN * self.op.place[i]))
            .collect();
        let bin = self.rng.pick_weighted(&weights).unwrap_or(PLACE_BINS / 2);
        let width = (X_MAX - X_MIN) / PLACE_BINS as f64;
        let x = X_MIN + width * (bin as f64 + self.rng.f64());
        let z = self.rng.range(0.1, 0.9);
        (x, z)
    }

    // --- bar presses and reinforcement

    fn register_press(&mut self) {
        self.chamber.lever_until = self.t + 0.3;
        self.last_press_t = self.t;
        self.op.presses_since_rf += 1;
        self.recorder.presses += 1;
        if let Some(run) = &mut self.classical_run {
            if run.in_cs() {
                run.cs_presses += 1;
            } else if run.in_pre_window(self.t) {
                run.pre_presses += 1;
            }
        }
        let design = self.operant_design;
        let reinforced = design.reinforcer != Reinforcer::None
            && self
                .schedule
                .on_press(&design.schedule, self.t, &mut self.rng);
        self.recorder.push(RecordEvent::Press {
            t: self.t,
            reinforced,
        });
        if reinforced {
            self.deliver(Source::Schedule);
        } else {
            self.op.unreinforced_press(self.t);
        }
    }

    /// Operates the dispenser and lets the rat learn from it.
    fn deliver(&mut self, source: Source) {
        let food = match source {
            Source::Manual => true,
            Source::Schedule => self.operant_design.reinforcer == Reinforcer::Food,
        };
        self.chamber.dispense_count += 1;
        if food {
            self.chamber.pellets = (self.chamber.pellets + 1).min(MAX_PELLETS);
        }
        self.op.pending_sounds.push(crate::operant::PendingSound {
            t: self.t,
            food,
            checked: false,
        });
        let kind = match (source, food) {
            (Source::Manual, _) => ReinforcerKind::Manual,
            (Source::Schedule, true) => ReinforcerKind::Food,
            (Source::Schedule, false) => ReinforcerKind::Sound,
        };
        self.recorder
            .push(RecordEvent::Reinforcer { t: self.t, kind });
        self.recorder.reinforcers += 1;

        // A click right after a press reinforces pressing, whoever operated the dispenser.
        let press_credit = source == Source::Schedule || self.t - self.last_press_t <= 1.0;
        if press_credit {
            self.op.reinforced_press();
        }
        if source == Source::Schedule {
            self.op
                .on_scheduled_reinforcement(self.t, self.operant_design.schedule.is_ratio());
        }
        self.credit_recent_behaviors();
        self.op.reinforce_place(self.rat.x);

        let value = self.op.sound_value();
        if self.rng.chance(value.max(0.03)) {
            self.op.approach_pending = true;
            let current = self.rat.bout.behavior;
            let heading_to_food =
                current == Behavior::Walk && self.rat.bout.then == Some(Behavior::Eat);
            if !matches!(
                current,
                Behavior::Eat | Behavior::Startle | Behavior::BarPress
            ) && !heading_to_food
            {
                self.interrupt();
                self.choose_next(None);
            } else if heading_to_food || current == Behavior::Eat {
                self.op.approach_pending = false;
            }
        }
    }

    /// Reinforcement strengthens what the rat is doing, and a bit what it just did.
    fn credit_recent_behaviors(&mut self) {
        let current = self.rat.bout.behavior;
        if current.is_shapeable() && !self.rat.bout.credited {
            self.rat.bout.credited = true;
            self.op.credit(current, 1.0);
        }
        let now = self.t;
        let mut to_credit = Vec::new();
        for r in &mut self.recent {
            if !r.credited && now - r.ended <= ELIGIBILITY_WINDOW {
                r.credited = true;
                to_credit.push(r.behavior);
            }
        }
        for b in to_credit {
            self.op.credit(b, 0.5);
        }
    }

    /// Shaped behaviours that went unreinforced lose a little strength.
    fn process_recent(&mut self) {
        let now = self.t;
        let mut expired = Vec::new();
        self.recent.retain(|r| {
            if now - r.ended > ELIGIBILITY_WINDOW + 0.5 {
                if !r.credited {
                    expired.push(r.behavior);
                }
                false
            } else {
                true
            }
        });
        for b in expired {
            self.op.unreinforced_free(b);
        }
    }

    // --- classical conditioning

    fn start_classical(&mut self) -> Result<(), String> {
        if self.classical_run.is_some() {
            return Err("A classical conditioning experiment is already running.".into());
        }
        self.classical_design.validate()?;
        let design = self.classical_design.clone();
        let order = self.stage_order(&design, 0);
        let cs_at = self.t + self.draw_iti(design.stages[0].mean_iti_minutes);
        self.classical_run = Some(ClassicalRun {
            design,
            stage: 0,
            order,
            pos: 0,
            total_trial: 0,
            phase: Phase::Iti { cs_at },
            pre_presses: 0,
            cs_presses: 0,
            pre_move: 0.0,
            cs_move: 0.0,
            fear_peak: 0.0,
        });
        self.recorder
            .push(RecordEvent::ClassicalStart { t: self.t });
        Ok(())
    }

    fn stage_order(&mut self, design: &ClassicalDesign, stage: usize) -> Vec<usize> {
        let mut order = Vec::new();
        for (i, tt) in design.stages[stage].trial_types.iter().enumerate() {
            order.extend(std::iter::repeat_n(i, tt.count as usize));
        }
        self.rng.shuffle(&mut order);
        order
    }

    fn draw_iti(&mut self, mean_minutes: f64) -> f64 {
        (mean_minutes * 60.0 * self.rng.range(0.5, 1.5)).max(60.0)
    }

    fn end_classical(&mut self, completed: bool) {
        if let Some(run) = &self.classical_run
            && run.in_cs()
        {
            self.recorder.push(RecordEvent::CsOff { t: self.t });
        }
        self.classical_run = None;
        self.chamber.clear_stimuli();
        self.recorder.push(RecordEvent::ClassicalEnd {
            t: self.t,
            completed,
        });
    }

    fn tick_classical(&mut self) {
        let Some(mut run) = self.classical_run.take() else {
            return;
        };
        let now = self.t;
        let mut finished = false;
        match run.phase {
            Phase::Iti { cs_at } => {
                if now >= cs_at {
                    run.phase = Phase::Cs {
                        start: now,
                        shocked: false,
                    };
                    run.fear_peak = self.cl.fear;
                    let cs = run.current_type().cs.clone();
                    self.chamber.light_on = cs.light;
                    self.chamber.tone_db = cs.tone_db;
                    self.chamber.bell_on = cs.bell;
                    self.recorder.push(RecordEvent::CsOn {
                        t: now,
                        cs: cs.clone(),
                    });
                    let p = self.cl.orient_probability(&cs);
                    let current = self.rat.bout.behavior;
                    if !matches!(current, Behavior::Eat | Behavior::Startle) && self.rng.chance(p) {
                        self.interrupt();
                        self.start(Behavior::Orient);
                    }
                }
            }
            Phase::Cs { start, shocked } => {
                let us = run.current_type().us;
                if !shocked && us != UsLevel::None && now >= start + CS_DURATION - SHOCK_DURATION {
                    run.phase = Phase::Cs {
                        start,
                        shocked: true,
                    };
                    self.shock(us);
                }
                if now >= start + CS_DURATION {
                    self.chamber.light_on = false;
                    self.chamber.tone_db = None;
                    self.chamber.bell_on = false;
                    self.recorder.push(RecordEvent::CsOff { t: now });
                    finished = self.finish_trial(&mut run, start);
                }
            }
        }
        if finished {
            self.classical_run = Some(run);
            self.end_classical(true);
        } else {
            self.classical_run = Some(run);
        }
    }

    /// Records the trial, applies learning and schedules the next one.
    /// Returns true when the experiment is over.
    fn finish_trial(&mut self, run: &mut ClassicalRun, cs_start: f64) -> bool {
        let tt = run.current_type().clone();
        self.cl.rescorla_wagner(&tt.cs, tt.us);
        let record = TrialRecord {
            trial: run.total_trial + 1,
            stage: run.stage as u32 + 1,
            trial_in_stage: run.pos as u32 + 1,
            t: cs_start,
            cs: tt.cs,
            us: tt.us,
            pre_presses: run.pre_presses,
            cs_presses: run.cs_presses,
            suppression_ratio: ratio(f64::from(run.cs_presses), f64::from(run.pre_presses)),
            pre_move: run.pre_move,
            cs_move: run.cs_move,
            movement_ratio: ratio(run.cs_move, run.pre_move),
            v_light: self.cl.strength(CsKind::Light),
            v_tone: self.cl.strength(CsKind::Tone),
            v_bell: self.cl.strength(CsKind::Bell),
            fear_peak: run.fear_peak,
            pain_sensitivity: self.cl.pain,
        };
        self.recorder.trials.push(record);
        self.sample();

        run.total_trial += 1;
        run.pos += 1;
        run.pre_presses = 0;
        run.cs_presses = 0;
        run.pre_move = 0.0;
        run.cs_move = 0.0;
        run.fear_peak = 0.0;
        if run.pos >= run.order.len() {
            run.stage += 1;
            run.pos = 0;
            if run.stage >= run.design.stages.len() {
                return true;
            }
            let design = run.design.clone();
            run.order = self.stage_order(&design, run.stage);
        }
        let iti = self.draw_iti(run.design.stages[run.stage].mean_iti_minutes);
        run.phase = Phase::Iti {
            cs_at: self.t + iti,
        };
        false
    }

    fn shock(&mut self, us: UsLevel) {
        self.chamber.shock = Some(us);
        self.chamber.shock_until = self.t + SHOCK_DURATION;
        self.chamber.shock_count += 1;
        self.recorder.push(RecordEvent::Shock {
            t: self.t,
            level: us,
        });
        let duration = self.cl.shocked(us);
        self.interrupt();
        self.op.approach_pending = false;
        self.start_with_duration(Behavior::Startle, duration);
    }

    fn cs_fear_target(&self) -> f64 {
        match &self.classical_run {
            Some(run) if run.in_cs() => self.cl.prediction(&run.current_type().cs).clamp(0.0, 1.0),
            _ => 0.0,
        }
    }

    fn accumulate_windows(&mut self) {
        let moving = self.rat.bout.behavior.is_moving();
        let now = self.t;
        if let Some(run) = &mut self.classical_run {
            if !moving {
                return;
            }
            if run.in_cs() {
                run.cs_move += DT;
            } else if run.in_pre_window(now) {
                run.pre_move += DT;
            }
        }
    }

    // --- time off

    fn time_off(&mut self, hours: f64) {
        self.op.rest(hours);
        self.cl.rest(hours);
        self.chamber.pellets = 0;
        self.chamber.shock = None;
        self.recent.clear();
        self.schedule
            .arm(&self.operant_design.schedule, self.t, &mut self.rng);
        self.recorder
            .push(RecordEvent::TimeOff { t: self.t, hours });
        self.place_near_center();
    }
}
