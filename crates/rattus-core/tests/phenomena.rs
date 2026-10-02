//! The learning phenomena of the classic exercises, checked headless with fixed seeds.

use rattus_core::Simulation;
use rattus_core::behavior::Behavior;
use rattus_core::classical::{ClassicalDesign, CsSpec, Stage, TrialType, UsLevel};
use rattus_core::protocol::{Command, RatTemplate};
use rattus_core::recorder::{RecordEvent, TrialRecord};
use rattus_core::schedule::{OperantDesign, Reinforcer, Schedule};

const MIN: f64 = 60.0;

fn presses_between(sim: &Simulation, t0: f64, t1: f64) -> usize {
    sim.recorder()
        .events
        .iter()
        .filter(|e| matches!(e, RecordEvent::Press { t, .. } if *t >= t0 && *t < t1))
        .count()
}

fn rate(sim: &Simulation, t0: f64, t1: f64) -> f64 {
    presses_between(sim, t0, t1) as f64 / ((t1 - t0) / MIN)
}

fn set(sim: &mut Simulation, schedule: Schedule, reinforcer: Reinforcer) {
    sim.apply(Command::SetOperantDesign {
        design: OperantDesign {
            schedule,
            reinforcer,
        },
    })
    .unwrap();
}

fn magazine_trained(seed: u64) -> Simulation {
    let mut sim = Simulation::new(RatTemplate::Naive, seed);
    set(&mut sim, Schedule::Continuous, Reinforcer::None);
    for _ in 0..40 {
        sim.apply(Command::GivePellet).unwrap();
        sim.advance(30.0);
    }
    sim
}

/// A bar-trained rat moved to `schedule` for 30 minutes.
fn trained_on(seed: u64, schedule: Schedule) -> Simulation {
    let mut sim = Simulation::new(RatTemplate::BarTrained, seed);
    set(&mut sim, schedule, Reinforcer::Food);
    sim.advance(30.0 * MIN);
    sim
}

fn extinction_presses(seed: u64, schedule: Schedule, reinforcer: Reinforcer) -> usize {
    let mut sim = trained_on(seed, schedule);
    set(&mut sim, schedule, reinforcer);
    let t0 = sim.time();
    sim.advance(60.0 * MIN);
    presses_between(&sim, t0, sim.time())
}

/// Seconds from each reinforcement to the next press.
fn mean_pause(sim: &Simulation, t0: f64) -> f64 {
    let mut pauses = Vec::new();
    let mut waiting = None;
    for e in sim.recorder().events.iter().filter(|e| e.t() >= t0) {
        match e {
            RecordEvent::Reinforcer { t, .. } => waiting = Some(*t),
            RecordEvent::Press { t, .. } => {
                if let Some(r) = waiting.take() {
                    pauses.push(t - r);
                }
            }
            _ => {}
        }
    }
    pauses.iter().sum::<f64>() / pauses.len().max(1) as f64
}

#[test]
fn magazine_training_builds_the_sound_food_association() {
    let sim = magazine_trained(1);
    assert!(sim.snapshot().mind.sound_food > 0.8);
}

#[test]
fn bar_pressing_is_acquired_on_crf_after_magazine_training() {
    let mut sim = magazine_trained(1);
    set(&mut sim, Schedule::Continuous, Reinforcer::Food);
    let t0 = sim.time();
    sim.advance(60.0 * MIN);
    assert!(rate(&sim, t0 + 40.0 * MIN, t0 + 60.0 * MIN) > 8.0);
    assert!(sim.snapshot().mind.bar_strength > 0.8);
}

/// Reinforces successive approximations to bar pressing by hand, like a student would.
fn time_to_presses(seed: u64, shape: bool, target: usize) -> f64 {
    let mut sim = magazine_trained(seed);
    set(&mut sim, Schedule::Continuous, Reinforcer::Food);
    let t0 = sim.time();
    let mut last_pellet = f64::NEG_INFINITY;
    while sim.time() - t0 < 120.0 * MIN {
        sim.step();
        if presses_between(&sim, t0, sim.time() + 1.0) >= target {
            break;
        }
        if shape && sim.time() - last_pellet > 8.0 {
            let snap = sim.snapshot();
            let near = snap.rat.x > 0.7 && snap.rat.facing == rattus_core::rat::Facing::Right;
            let rearing = matches!(snap.rat.behavior, Behavior::RearWall | Behavior::Rear);
            if near && rearing {
                sim.apply(Command::GivePellet).unwrap();
                last_pellet = sim.time();
            }
        }
    }
    sim.time() - t0
}

#[test]
fn shaping_speeds_up_acquisition() {
    let seeds = 1..=6;
    let shaped: f64 = seeds.clone().map(|s| time_to_presses(s, true, 30)).sum();
    let unshaped: f64 = seeds.map(|s| time_to_presses(s, false, 30)).sum();
    eprintln!("shaped {shaped:.0} s, unshaped {unshaped:.0} s");
    assert!(
        shaped < unshaped,
        "shaped {:.0} s vs unshaped {:.0} s",
        shaped,
        unshaped
    );
}

#[test]
fn shaping_rearing_leads_to_begging() {
    let mut sim = magazine_trained(3);
    set(&mut sim, Schedule::Continuous, Reinforcer::None);
    let begs_before = count_bouts(&mut sim, Behavior::Beg, 20.0 * MIN, |_| false);
    // Reinforce rearing, then begging itself as soon as it appears.
    let begs_after = count_bouts(&mut sim, Behavior::Beg, 40.0 * MIN, |b| {
        matches!(b, Behavior::Rear | Behavior::Beg)
    });
    assert!(
        begs_after > begs_before * 3 + 5,
        "{begs_before} -> {begs_after}"
    );
}

/// Runs for `duration`, reinforcing (at most every 6 s) when `reinforce` approves the
/// current behaviour, and counts bouts of `target`.
fn count_bouts(
    sim: &mut Simulation,
    target: Behavior,
    duration: f64,
    reinforce: impl Fn(Behavior) -> bool,
) -> usize {
    let end = sim.time() + duration;
    let mut last = sim.current_behavior();
    let mut count = 0;
    let mut last_pellet = f64::NEG_INFINITY;
    let mut last_bout_start = sim.time();
    while sim.time() < end {
        sim.step();
        let b = sim.current_behavior();
        let snap = sim.snapshot();
        let new_bout = b != last || snap.rat.elapsed < 0.06;
        if new_bout && sim.time() - last_bout_start > 0.04 {
            last_bout_start = sim.time();
            if b == target {
                count += 1;
            }
        }
        last = b;
        if snap.rat.elapsed > 0.5 && sim.time() - last_pellet > 6.0 && reinforce(b) {
            sim.apply(Command::GivePellet).unwrap();
            last_pellet = sim.time();
        }
    }
    count
}

#[test]
fn extinction_after_crf_is_fast() {
    let mut sim = trained_on(2, Schedule::Continuous);
    let trained_rate = rate(&sim, sim.time() - 10.0 * MIN, sim.time());
    set(&mut sim, Schedule::Continuous, Reinforcer::None);
    let t0 = sim.time();
    sim.advance(60.0 * MIN);
    let late = rate(&sim, t0 + 50.0 * MIN, t0 + 60.0 * MIN);
    assert!(trained_rate > 10.0);
    assert!(late < trained_rate * 0.1, "{trained_rate} -> {late}");
}

#[test]
fn partial_reinforcement_slows_extinction() {
    for seed in [1, 2] {
        let after_crf = extinction_presses(seed, Schedule::Continuous, Reinforcer::None);
        let after_vr =
            extinction_presses(seed, Schedule::VariableRatio { n: 25 }, Reinforcer::None);
        assert!(
            after_vr > 2 * after_crf,
            "CRF {after_crf}, VR-25 {after_vr}"
        );
    }
}

#[test]
fn the_click_alone_is_a_secondary_reinforcer() {
    for seed in [1, 2] {
        let none = extinction_presses(seed, Schedule::Continuous, Reinforcer::None);
        let sound = extinction_presses(seed, Schedule::Continuous, Reinforcer::SoundOnly);
        assert!(sound > 2 * none, "none {none}, sound only {sound}");
    }
}

#[test]
fn spontaneous_recovery_after_time_off() {
    let mut sim = trained_on(4, Schedule::Continuous);
    set(&mut sim, Schedule::Continuous, Reinforcer::None);
    sim.advance(60.0 * MIN);
    let before = presses_between(&sim, sim.time() - 10.0 * MIN, sim.time());
    sim.apply(Command::TimeOff { hours: 24.0 }).unwrap();
    let t0 = sim.time();
    sim.advance(10.0 * MIN);
    let after = presses_between(&sim, t0, sim.time());
    assert!(after >= 2 * before + 5, "before {before}, after {after}");
}

#[test]
fn schedules_produce_their_typical_patterns() {
    let measure = |schedule| {
        let mut sim = trained_on(1, schedule);
        let t0 = sim.time();
        sim.advance(30.0 * MIN);
        (rate(&sim, t0, sim.time()), mean_pause(&sim, t0), sim)
    };
    let (fr_rate, fr_pause, _) = measure(Schedule::FixedRatio { n: 50 });
    let (vr_rate, vr_pause, _) = measure(Schedule::VariableRatio { n: 50 });
    let (vi_rate, _, _) = measure(Schedule::VariableInterval { seconds: 30.0 });
    let (_, _, fi) = measure(Schedule::FixedInterval { seconds: 60.0 });

    assert!(
        fr_pause > vr_pause * 1.5,
        "FR pause {fr_pause}, VR pause {vr_pause}"
    );
    assert!(
        vr_rate > vi_rate * 1.5,
        "VR {vr_rate}/min, VI {vi_rate}/min"
    );
    assert!(fr_rate > 15.0);

    // Fixed interval scallop: more presses late in the interval than early.
    let rf: Vec<f64> = fi
        .recorder()
        .events
        .iter()
        .filter_map(|e| match e {
            RecordEvent::Reinforcer { t, .. } if *t > 30.0 * MIN => Some(*t),
            _ => None,
        })
        .collect();
    let (mut early, mut late) = (0, 0);
    for w in rf.windows(2) {
        let third = (w[1] - w[0]) / 3.0;
        early += presses_between(&fi, w[0], w[0] + third);
        late += presses_between(&fi, w[1] - third, w[1] + 0.01);
    }
    assert!(late > early * 3 / 2, "FI early {early}, late {late}");
}

fn run_cer(seed: u64, cs: CsSpec, acquisition: u32, extinction: u32) -> Vec<TrialRecord> {
    let mut sim = Simulation::new(RatTemplate::BarTrained, seed);
    sim.advance(10.0 * MIN);
    let mut stages = vec![Stage {
        trial_types: vec![TrialType {
            cs: cs.clone(),
            us: UsLevel::Medium,
            count: acquisition,
        }],
        mean_iti_minutes: 5.0,
    }];
    if extinction > 0 {
        stages.push(Stage {
            trial_types: vec![TrialType {
                cs,
                us: UsLevel::None,
                count: extinction,
            }],
            mean_iti_minutes: 5.0,
        });
    }
    sim.apply(Command::SetClassicalDesign {
        design: ClassicalDesign { stages },
    })
    .unwrap();
    sim.apply(Command::StartClassical).unwrap();
    while sim.classical_running() {
        sim.advance(MIN);
    }
    sim.recorder().trials.clone()
}

fn mean_ratio(trials: &[TrialRecord]) -> f64 {
    let v: Vec<f64> = trials.iter().filter_map(|t| t.suppression_ratio).collect();
    v.iter().sum::<f64>() / v.len().max(1) as f64
}

#[test]
fn cer_acquisition_and_extinction() {
    let trials = run_cer(1, CsSpec::light(), 10, 30);
    assert_eq!(trials.len(), 40);
    let first = mean_ratio(&trials[..1]);
    let acquired = mean_ratio(&trials[7..10]);
    let extinguished = mean_ratio(&trials[36..40]);
    assert!(first > 0.3, "first trial {first}");
    assert!(acquired < 0.2, "end of acquisition {acquired}");
    assert!(
        extinguished > acquired + 0.1,
        "end of extinction {extinguished}"
    );
    let movement: Vec<f64> = trials[7..10]
        .iter()
        .filter_map(|t| t.movement_ratio)
        .collect();
    assert!(movement.iter().all(|m| *m < 0.45));
}

#[test]
fn louder_cs_conditions_faster() {
    let soft = run_cer(2, CsSpec::tone(70.0), 3, 0);
    let loud = run_cer(2, CsSpec::tone(95.0), 3, 0);
    assert!(loud[2].v_tone > soft[2].v_tone + 0.1);
}

#[test]
fn shock_startles_and_cs_raises_fear() {
    let mut sim = Simulation::new(RatTemplate::BarTrained, 5);
    sim.apply(Command::SetClassicalDesign {
        design: ClassicalDesign {
            stages: vec![Stage {
                trial_types: vec![TrialType {
                    cs: CsSpec::light(),
                    us: UsLevel::High,
                    count: 3,
                }],
                mean_iti_minutes: 1.0,
            }],
        },
    })
    .unwrap();
    sim.apply(Command::StartClassical).unwrap();
    let mut startled = false;
    let mut max_fear: f64 = 0.0;
    while sim.classical_running() {
        sim.step();
        let snap = sim.snapshot();
        if snap.chamber.shock.is_some() && snap.rat.behavior == Behavior::Startle {
            startled = true;
        }
        if snap.chamber.light_on {
            max_fear = max_fear.max(snap.mind.fear);
        }
    }
    assert!(startled);
    assert!(max_fear > 0.3);
    assert!(
        sim.snapshot().mind.pain_sensitivity > 1.0,
        "strong shocks sensitize"
    );
}
