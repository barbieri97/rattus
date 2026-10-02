//! Ready-made rats.

use crate::assoc::Assoc;
use crate::behavior::Behavior;
use crate::operant::{Ema, OperantMind};
use crate::rat::{DEVICE_Z, Facing, LEVER_X, Rat};
use crate::rng::Rng;
use crate::schedule::{OperantDesign, Reinforcer, Schedule};
use crate::sim::Simulation;

impl Simulation {
    /// A rat that has never been in the chamber. Bar presses are reinforced continuously
    /// with food, so once it presses the bar on its own it is rewarded automatically.
    pub fn naive(seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let x = rng.range(0.35, 0.6);
        let z = rng.range(0.3, 0.7);
        let facing = if rng.chance(0.5) {
            Facing::Left
        } else {
            Facing::Right
        };
        Self::assemble(
            rng,
            Rat::new(x, z, facing),
            OperantMind::default(),
            OperantDesign::default(),
        )
    }

    /// A rat already magazine trained and pressing the bar on a VR-25 schedule.
    /// Classical conditioning of fear (CER) is measured as suppression of its pressing.
    pub fn bar_trained(seed: u64) -> Self {
        let rng = Rng::new(seed);
        let mut op = OperantMind {
            sound_food: Assoc::with_strength(0.95),
            bar_sound: Assoc::with_strength(0.7),
            p_reinf: 0.04,
            p_recent: 0.04,
            resp_per_rf: Ema::seeded(25.0, 14.0),
            secs_per_rf: Ema::seeded(30.0, 18.0),
            ratio_bias: 1.0,
            ..OperantMind::default()
        };
        *op.action_mut(Behavior::BarPress) = Assoc::with_strength(0.8);
        *op.action_mut(Behavior::RearWall) = Assoc::with_strength(0.3);
        let n = op.place.len();
        for (i, p) in op.place.iter_mut().enumerate() {
            *p = if i + 3 >= n { 0.8 } else { 0.0 };
        }
        let design = OperantDesign {
            schedule: Schedule::VariableRatio { n: 25 },
            reinforcer: Reinforcer::Food,
        };
        Self::assemble(rng, Rat::new(LEVER_X, DEVICE_Z, Facing::Right), op, design)
    }
}
