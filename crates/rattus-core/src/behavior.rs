//! The rat's behavioural repertoire.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const BEHAVIOR_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Behavior {
    /// Locomotion toward a point in the chamber.
    Walk,
    /// Sniffing the floor and the air.
    Sniff,
    /// Standing still and looking around.
    LookAround,
    /// Rearing up on the hind legs, away from the walls.
    Rear,
    /// Rearing up against a wall.
    RearWall,
    /// Grooming the body.
    Groom,
    /// Wiping a forepaw across the face.
    FaceWipe,
    /// Scratching with a hind leg.
    Scratch,
    /// Drinking from the water spout.
    Drink,
    /// Eating a pellet from the food cup.
    Eat,
    /// Pressing the lever.
    BarPress,
    /// Sitting up facing the observer with the forepaws raised.
    Beg,
    /// Rolling over.
    Roll,
    /// Crouching motionless, the main fear response.
    Freeze,
    /// Unconditioned response to a shock: jumping and flinching.
    Startle,
    /// Orienting response toward a new stimulus.
    Orient,
}

impl Behavior {
    pub const ALL: [Behavior; BEHAVIOR_COUNT] = [
        Behavior::Walk,
        Behavior::Sniff,
        Behavior::LookAround,
        Behavior::Rear,
        Behavior::RearWall,
        Behavior::Groom,
        Behavior::FaceWipe,
        Behavior::Scratch,
        Behavior::Drink,
        Behavior::Eat,
        Behavior::BarPress,
        Behavior::Beg,
        Behavior::Roll,
        Behavior::Freeze,
        Behavior::Startle,
        Behavior::Orient,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    /// Range of bout durations in seconds of program time.
    /// Walking is timed by distance instead; Startle by pain sensitivity.
    pub fn duration_range(self) -> (f64, f64) {
        match self {
            Behavior::Walk => (0.3, 0.3),
            Behavior::Sniff => (1.0, 3.0),
            Behavior::LookAround => (1.0, 2.5),
            Behavior::Rear => (1.0, 2.5),
            Behavior::RearWall => (1.0, 2.5),
            Behavior::Groom => (3.0, 7.0),
            Behavior::FaceWipe => (2.0, 4.0),
            Behavior::Scratch => (1.0, 2.5),
            Behavior::Drink => (2.0, 5.0),
            Behavior::Eat => (2.5, 3.5),
            Behavior::BarPress => (0.6, 0.9),
            Behavior::Beg => (2.0, 3.0),
            Behavior::Roll => (1.5, 2.2),
            Behavior::Freeze => (1.5, 4.0),
            Behavior::Startle => (1.0, 1.0),
            Behavior::Orient => (0.8, 1.5),
        }
    }

    /// Behaviours that can be strengthened by reinforcing them (shaping).
    /// Bar pressing has its own learning rules.
    pub fn is_shapeable(self) -> bool {
        matches!(
            self,
            Behavior::Walk
                | Behavior::Sniff
                | Behavior::LookAround
                | Behavior::Rear
                | Behavior::RearWall
                | Behavior::Groom
                | Behavior::FaceWipe
                | Behavior::Scratch
                | Behavior::Drink
                | Behavior::Beg
                | Behavior::Roll
        )
    }

    /// Whether the rat counts as moving, for the movement ratio.
    pub fn is_moving(self) -> bool {
        !matches!(self, Behavior::Freeze)
    }

    pub fn label(self) -> &'static str {
        match self {
            Behavior::Walk => "Walking",
            Behavior::Sniff => "Sniffing",
            Behavior::LookAround => "Looking around",
            Behavior::Rear => "Rearing",
            Behavior::RearWall => "Rearing against the wall",
            Behavior::Groom => "Grooming",
            Behavior::FaceWipe => "Wiping face",
            Behavior::Scratch => "Scratching",
            Behavior::Drink => "Drinking",
            Behavior::Eat => "Eating",
            Behavior::BarPress => "Pressing the bar",
            Behavior::Beg => "Begging",
            Behavior::Roll => "Rolling over",
            Behavior::Freeze => "Freezing",
            Behavior::Startle => "Startled by shock",
            Behavior::Orient => "Orienting",
        }
    }
}

/// How much reinforcing `a` also strengthens `b`. This generalization is what makes shaping
/// work: reinforcing rearing makes begging more likely, rearing at the wall leads to bar
/// pressing, grooming leads to face wiping and rolling over.
pub fn similarity(a: Behavior, b: Behavior) -> f64 {
    use Behavior::*;
    if a == b {
        return 1.0;
    }
    let pair = |x: Behavior, y: Behavior| (a == x && b == y) || (a == y && b == x);
    if pair(Rear, RearWall) {
        0.5
    } else if pair(RearWall, BarPress) {
        0.35
    } else if pair(Rear, Beg) {
        0.3
    } else if pair(Rear, BarPress) {
        0.15
    } else if pair(Groom, FaceWipe) {
        0.4
    } else if pair(Groom, Scratch) {
        0.3
    } else if pair(FaceWipe, Beg) || pair(Scratch, Roll) {
        0.25
    } else if pair(Groom, Roll) {
        0.2
    } else if pair(Sniff, LookAround) {
        0.3
    } else if pair(Walk, Sniff) {
        0.1
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_match_all() {
        for (i, b) in Behavior::ALL.iter().enumerate() {
            assert_eq!(b.index(), i);
        }
    }

    #[test]
    fn similarity_is_symmetric() {
        for a in Behavior::ALL {
            for b in Behavior::ALL {
                assert_eq!(similarity(a, b), similarity(b, a));
            }
        }
    }
}
