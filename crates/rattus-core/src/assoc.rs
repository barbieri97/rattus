//! Associative strength with separate excitatory and inhibitory components.
//!
//! The visible strength is `exc - inh`. Learning that increases strength first removes
//! inhibition and then adds excitation; extinction adds inhibition. Inhibition fades
//! during rest, which is what produces spontaneous recovery after extinction.

use serde::{Deserialize, Serialize};

/// Excitation is capped so long sessions cannot inflate it without bound.
const EXC_MAX: f64 = 1.5;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Assoc {
    pub exc: f64,
    pub inh: f64,
}

impl Assoc {
    pub fn with_strength(value: f64) -> Self {
        Self {
            exc: value.clamp(0.0, EXC_MAX),
            inh: 0.0,
        }
    }

    /// Net strength, which can be negative for a conditioned inhibitor.
    pub fn net(&self) -> f64 {
        self.exc - self.inh
    }

    /// Net strength clamped to `[0, 1]`, for uses where negative values make no sense.
    pub fn level(&self) -> f64 {
        self.net().clamp(0.0, 1.0)
    }

    /// Applies a change to the net strength.
    pub fn adjust(&mut self, delta: f64) {
        if delta > 0.0 {
            let from_inh = delta.min(self.inh);
            self.inh -= from_inh;
            self.exc = (self.exc + delta - from_inh).min(EXC_MAX);
        } else {
            self.inh -= delta;
        }
    }

    /// Error-correction step: moves the net strength toward `target` by `rate * error`.
    pub fn learn(&mut self, rate: f64, target: f64) {
        self.adjust(rate * (target - self.net()));
    }

    /// Lets inhibition fade during `hours` of rest (it halves every `half_life_hours`).
    pub fn rest(&mut self, hours: f64, half_life_hours: f64) {
        self.inh *= 0.5_f64.powf(hours / half_life_hours);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learning_approaches_target() {
        let mut a = Assoc::default();
        for _ in 0..50 {
            a.learn(0.2, 1.0);
        }
        assert!((a.net() - 1.0).abs() < 1e-3);
        assert_eq!(a.inh, 0.0);
    }

    #[test]
    fn extinction_builds_inhibition_and_rest_recovers() {
        let mut a = Assoc::with_strength(1.0);
        for _ in 0..60 {
            a.learn(0.2, 0.0);
        }
        assert!(a.net() < 0.01);
        assert!(a.inh > 0.9);
        a.rest(24.0, 24.0);
        assert!((a.net() - 0.5).abs() < 0.05, "recovered to {}", a.net());
    }

    #[test]
    fn reacquisition_removes_inhibition_first() {
        let mut a = Assoc { exc: 1.0, inh: 0.8 };
        a.adjust(0.5);
        assert!((a.inh - 0.3).abs() < 1e-9);
        assert!((a.exc - 1.0).abs() < 1e-9);
    }

    #[test]
    fn negative_strength_is_possible() {
        let mut a = Assoc::default();
        a.adjust(-0.3);
        assert!((a.net() + 0.3).abs() < 1e-9);
        assert_eq!(a.level(), 0.0);
    }
}
