//! Background thread that advances program time and streams frames to the interface.

use std::thread;
use std::time::{Duration, Instant};

use rattus_core::DT;
use rattus_core::protocol::{Frame, SimMessage};

use crate::SharedHost;

const TICK: Duration = Duration::from_millis(16);
/// Most simulation work done per tick, so commands are never blocked for long.
const BUDGET: Duration = Duration::from_millis(10);
/// Frames per second sent to the interface while the rat is visible / isolated.
const VISIBLE_FPS: f64 = 30.0;
const ISOLATED_FPS: f64 = 10.0;

pub fn spawn(shared: SharedHost) {
    thread::Builder::new()
        .name("simulation".into())
        .spawn(move || run(shared))
        .expect("could not start the simulation thread");
}

fn run(shared: SharedHost) {
    let mut last = Instant::now();
    let mut last_frame = Instant::now();
    // Program time owed to the simulation.
    let mut owed = 0.0_f64;
    loop {
        thread::sleep(TICK);
        let now = Instant::now();
        let real_dt = now.duration_since(last).as_secs_f64().min(0.25);
        last = now;

        let mut host = shared.lock();
        if host.status.paused {
            owed = 0.0;
        } else {
            let speed = if host.status.isolated {
                host.status.isolated_speed
            } else {
                host.status.speed
            };
            owed += real_dt * speed;
            let started = Instant::now();
            while owed >= DT {
                host.sim.step();
                owed -= DT;
                if started.elapsed() >= BUDGET {
                    // The machine cannot keep up with this speed; drop the backlog.
                    owed = owed.min(DT * 4.0);
                    break;
                }
            }
        }

        let fps = if host.status.isolated {
            ISOLATED_FPS
        } else {
            VISIBLE_FPS
        };
        if now.duration_since(last_frame).as_secs_f64() >= 1.0 / fps - 0.002 {
            last_frame = now;
            if host.channel.is_some() {
                let frame = Frame {
                    snapshot: host.sim.snapshot(),
                    host: host.status.clone(),
                    delta: host.sim.drain(),
                };
                let sent = host
                    .channel
                    .as_ref()
                    .map(|c| c.send(SimMessage::Frame(Box::new(frame))).is_ok())
                    .unwrap_or(false);
                if !sent {
                    host.channel = None;
                }
            }
        }
    }
}
