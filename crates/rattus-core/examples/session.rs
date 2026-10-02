//! Runs the classic exercises headless and prints the numbers, to calibrate the model.
//!
//! cargo run -p rattus-core --example session [scenario] [seed]

use rattus_core::Simulation;
use rattus_core::classical::{ClassicalDesign, CsSpec, Stage, TrialType, UsLevel};
use rattus_core::protocol::{Command, RatTemplate};
use rattus_core::recorder::RecordEvent;
use rattus_core::schedule::{OperantDesign, Reinforcer, Schedule};

fn presses_between(sim: &Simulation, t0: f64, t1: f64) -> usize {
    sim.recorder()
        .events
        .iter()
        .filter(|e| matches!(e, RecordEvent::Press { t, .. } if *t >= t0 && *t < t1))
        .count()
}

fn rate_profile(sim: &Simulation, t0: f64, minutes: u32, bin: u32) -> Vec<f64> {
    (0..minutes / bin)
        .map(|i| {
            let a = t0 + f64::from(i * bin) * 60.0;
            presses_between(sim, a, a + f64::from(bin) * 60.0) as f64 / f64::from(bin)
        })
        .collect()
}

fn fmt(v: &[f64]) -> String {
    v.iter()
        .map(|x| format!("{x:5.1}"))
        .collect::<Vec<_>>()
        .join(" ")
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

fn magazine_train(sim: &mut Simulation, pellets: u32) {
    for _ in 0..pellets {
        sim.apply(Command::GivePellet).unwrap();
        sim.advance(30.0);
    }
}

/// Mean time from each scheduled reinforcement to the next press, and presses/min.
fn pause_stats(sim: &Simulation, t0: f64) -> (f64, f64) {
    let events: Vec<_> = sim
        .recorder()
        .events
        .iter()
        .filter(|e| e.t() >= t0)
        .collect();
    let mut pauses = Vec::new();
    let mut waiting: Option<f64> = None;
    for e in &events {
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
    let mean = pauses.iter().sum::<f64>() / pauses.len().max(1) as f64;
    let presses = events
        .iter()
        .filter(|e| matches!(e, RecordEvent::Press { .. }))
        .count();
    let span = (sim.time() - t0) / 60.0;
    (mean, presses as f64 / span)
}

/// Fraction of presses in the first and last third of each inter-reinforcement interval.
fn thirds(sim: &Simulation, t0: f64) -> (f64, f64) {
    let rf: Vec<f64> = sim
        .recorder()
        .events
        .iter()
        .filter_map(|e| match e {
            RecordEvent::Reinforcer { t, .. } if *t >= t0 => Some(*t),
            _ => None,
        })
        .collect();
    let (mut first, mut last, mut all) = (0usize, 0usize, 0usize);
    for w in rf.windows(2) {
        let (a, b) = (w[0], w[1]);
        let third = (b - a) / 3.0;
        first += presses_between(sim, a, a + third);
        last += presses_between(sim, b - third, b + 0.01);
        all += presses_between(sim, a, b + 0.01);
    }
    let all = all.max(1) as f64;
    (first as f64 / all, last as f64 / all)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scenario = args.get(1).map(String::as_str).unwrap_or("all");
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let all = scenario == "all";

    if all || scenario == "magazine" {
        let mut sim = Simulation::new(RatTemplate::Naive, seed);
        set(&mut sim, Schedule::Continuous, Reinforcer::None);
        print!("magazine training, sound-food by pellet:");
        let _ = magazine_train;
        for i in 0..40 {
            sim.apply(Command::GivePellet).unwrap();
            sim.advance(30.0);
            if i % 5 == 4 {
                print!(" {:.2}", sim.snapshot().mind.sound_food);
            }
        }
        println!();
        set(&mut sim, Schedule::Continuous, Reinforcer::Food);
        let t0 = sim.time();
        sim.advance(90.0 * 60.0);
        println!(
            "acquisition on CRF without shaping, presses/min per 10 min: {}",
            fmt(&rate_profile(&sim, t0, 90, 10))
        );
        println!(
            "  first 20 min per minute: {}",
            fmt(&rate_profile(&sim, t0, 20, 1))
        );
        let m = sim.snapshot().mind;
        println!(
            "  sound-food {:.2} bar-sound {:.2} bar strength {:.2}",
            m.sound_food, m.bar_sound, m.bar_strength
        );
    }

    if all || scenario == "schedules" {
        for (name, schedule) in [
            ("CRF", Schedule::Continuous),
            ("FR-20", Schedule::FixedRatio { n: 20 }),
            ("FR-60", Schedule::FixedRatio { n: 60 }),
            ("VR-25", Schedule::VariableRatio { n: 25 }),
            ("VR-60", Schedule::VariableRatio { n: 60 }),
            ("FI-60", Schedule::FixedInterval { seconds: 60.0 }),
            ("VI-30", Schedule::VariableInterval { seconds: 30.0 }),
            ("VI-60", Schedule::VariableInterval { seconds: 60.0 }),
        ] {
            let mut sim = Simulation::new(RatTemplate::BarTrained, seed);
            set(&mut sim, schedule, Reinforcer::Food);
            sim.advance(20.0 * 60.0);
            let t0 = sim.time();
            sim.advance(30.0 * 60.0);
            let (pause, rate) = pause_stats(&sim, t0);
            let (first, last) = thirds(&sim, t0);
            let m = sim.snapshot().mind;
            println!(
                "{name:6} rate {rate:5.1}/min  pause after rf {pause:5.1}s  first/last third {first:.2}/{last:.2}  bar {:.2} bs {:.2}",
                m.bar_strength, m.bar_sound
            );
        }
    }

    if all || scenario == "extinction" {
        for (name, schedule) in [
            ("CRF", Schedule::Continuous),
            ("VR-25", Schedule::VariableRatio { n: 25 }),
        ] {
            for reinforcer in [Reinforcer::None, Reinforcer::SoundOnly] {
                let mut sim = Simulation::new(RatTemplate::BarTrained, seed);
                set(&mut sim, schedule, Reinforcer::Food);
                sim.advance(30.0 * 60.0);
                set(&mut sim, schedule, reinforcer);
                let t0 = sim.time();
                sim.advance(60.0 * 60.0);
                let total = presses_between(&sim, t0, sim.time());
                println!(
                    "after {name:5} {:?}: total {total:4}, presses/min per 5 min: {}",
                    reinforcer,
                    fmt(&rate_profile(&sim, t0, 60, 5))
                );
                if reinforcer == Reinforcer::None && name == "CRF" {
                    let before = presses_between(&sim, sim.time() - 300.0, sim.time());
                    sim.apply(Command::TimeOff { hours: 24.0 }).unwrap();
                    let t1 = sim.time();
                    sim.advance(20.0 * 60.0);
                    println!(
                        "  spontaneous recovery: last 5 min before rest {before}, after rest per 5 min: {}",
                        fmt(&rate_profile(&sim, t1, 20, 5))
                    );
                }
            }
        }
    }

    if all || scenario == "cer" {
        for (name, cs) in [
            ("light", CsSpec::light()),
            ("tone 70", CsSpec::tone(70.0)),
            ("tone 95", CsSpec::tone(95.0)),
        ] {
            let mut sim = Simulation::new(RatTemplate::BarTrained, seed);
            sim.advance(10.0 * 60.0);
            let design = ClassicalDesign {
                stages: vec![
                    Stage {
                        trial_types: vec![TrialType {
                            cs: cs.clone(),
                            us: UsLevel::Medium,
                            count: 10,
                        }],
                        mean_iti_minutes: 5.0,
                    },
                    Stage {
                        trial_types: vec![TrialType {
                            cs: cs.clone(),
                            us: UsLevel::None,
                            count: 25,
                        }],
                        mean_iti_minutes: 5.0,
                    },
                ],
            };
            sim.apply(Command::SetClassicalDesign { design }).unwrap();
            sim.apply(Command::StartClassical).unwrap();
            while sim.classical_running() {
                sim.advance(60.0);
            }
            let sr: Vec<String> = sim
                .recorder()
                .trials
                .iter()
                .map(|t| {
                    t.suppression_ratio
                        .map(|r| format!("{r:.2}"))
                        .unwrap_or("-".into())
                })
                .collect();
            let mr: Vec<String> = sim
                .recorder()
                .trials
                .iter()
                .map(|t| {
                    t.movement_ratio
                        .map(|r| format!("{r:.2}"))
                        .unwrap_or("-".into())
                })
                .collect();
            println!("CER {name}: suppression {}", sr.join(" "));
            println!("            movement    {}", mr.join(" "));
            let pre: Vec<u32> = sim
                .recorder()
                .trials
                .iter()
                .map(|t| t.pre_presses)
                .collect();
            println!("            pre-CS presses {:?}", &pre[..10]);
        }
    }
}
