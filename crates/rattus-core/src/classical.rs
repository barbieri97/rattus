//! Classical conditioning: experiment designs, the conditioned emotional response (CER)
//! and the Rescorla-Wagner learning rule.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::assoc::Assoc;

/// Duration of the period before the CS used as the baseline for the ratios (s).
pub const PRE_CS_WINDOW: f64 = 30.0;
/// Duration of every CS presentation (s).
pub const CS_DURATION: f64 = 30.0;
/// The shock occupies the last second of the CS.
pub const SHOCK_DURATION: f64 = 1.0;
/// Learning rate parameter when the US is present.
const BETA_US: f64 = 0.5;
/// Learning rate parameter when the US is absent (extinction).
const BETA_NO_US: f64 = 0.25;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CsKind {
    Light,
    Tone,
    Bell,
}

impl CsKind {
    pub const ALL: [CsKind; 3] = [CsKind::Light, CsKind::Tone, CsKind::Bell];

    pub fn index(self) -> usize {
        self as usize
    }
}

/// Which conditioned stimuli a trial presents.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CsSpec {
    pub light: bool,
    /// Tone intensity in dB, or no tone.
    pub tone_db: Option<f64>,
    pub bell: bool,
}

impl CsSpec {
    pub fn light() -> Self {
        Self {
            light: true,
            tone_db: None,
            bell: false,
        }
    }

    pub fn tone(db: f64) -> Self {
        Self {
            light: false,
            tone_db: Some(db),
            bell: false,
        }
    }

    pub fn has(&self, cs: CsKind) -> bool {
        match cs {
            CsKind::Light => self.light,
            CsKind::Tone => self.tone_db.is_some(),
            CsKind::Bell => self.bell,
        }
    }

    pub fn is_empty(&self) -> bool {
        CsKind::ALL.iter().all(|cs| !self.has(*cs))
    }

    /// Salience (alpha) of a stimulus. Louder tones are more salient.
    pub fn salience(&self, cs: CsKind) -> f64 {
        match cs {
            CsKind::Light => 0.3,
            CsKind::Bell => 0.35,
            CsKind::Tone => ((self.tone_db.unwrap_or(80.0) - 50.0) / 100.0).clamp(0.05, 0.5),
        }
    }

    pub fn label(&self) -> String {
        let mut parts = Vec::new();
        if self.light {
            parts.push("Light".to_string());
        }
        if let Some(db) = self.tone_db {
            parts.push(format!("Tone {db:.0} dB"));
        }
        if self.bell {
            parts.push("Bell".to_string());
        }
        if parts.is_empty() {
            "None".to_string()
        } else {
            parts.join(" + ")
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum UsLevel {
    None,
    Low,
    Medium,
    High,
}

impl UsLevel {
    /// Maximum associative strength the shock supports (lambda).
    pub fn lambda(self) -> f64 {
        match self {
            UsLevel::None => 0.0,
            UsLevel::Low => 0.55,
            UsLevel::Medium => 0.85,
            UsLevel::High => 1.0,
        }
    }

    pub fn milliamps(self) -> f64 {
        match self {
            UsLevel::None => 0.0,
            UsLevel::Low => 0.5,
            UsLevel::Medium => 1.0,
            UsLevel::High => 1.5,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            UsLevel::None => "No shock",
            UsLevel::Low => "Low shock",
            UsLevel::Medium => "Medium shock",
            UsLevel::High => "High shock",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrialType {
    pub cs: CsSpec,
    pub us: UsLevel,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Stage {
    pub trial_types: Vec<TrialType>,
    /// Mean interval between trials, in minutes. Each interval varies by ±50%.
    pub mean_iti_minutes: f64,
}

impl Stage {
    pub fn trial_count(&self) -> u32 {
        self.trial_types.iter().map(|t| t.count).sum()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ClassicalDesign {
    pub stages: Vec<Stage>,
}

impl Default for ClassicalDesign {
    fn default() -> Self {
        Self {
            stages: vec![Stage {
                trial_types: vec![TrialType {
                    cs: CsSpec::light(),
                    us: UsLevel::Medium,
                    count: 10,
                }],
                mean_iti_minutes: 5.0,
            }],
        }
    }
}

impl ClassicalDesign {
    pub fn total_trials(&self) -> u32 {
        self.stages.iter().map(Stage::trial_count).sum()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.stages.is_empty() {
            return Err("The experiment needs at least one stage.".into());
        }
        if self.stages.len() > 20 {
            return Err("An experiment can have at most 20 stages.".into());
        }
        for (i, stage) in self.stages.iter().enumerate() {
            let n = i + 1;
            if stage.trial_types.is_empty() || stage.trial_types.len() > 4 {
                return Err(format!("Stage {n} must have between 1 and 4 trial types."));
            }
            if !stage.mean_iti_minutes.is_finite()
                || !(1.0..=60.0).contains(&stage.mean_iti_minutes)
            {
                return Err(format!(
                    "Stage {n}: the inter-trial interval must be between 1 and 60 minutes."
                ));
            }
            for tt in &stage.trial_types {
                if !(1..=200).contains(&tt.count) {
                    return Err(format!(
                        "Stage {n}: each trial type needs between 1 and 200 trials."
                    ));
                }
                if tt.cs.is_empty() {
                    return Err(format!("Stage {n}: every trial must present a stimulus."));
                }
                if let Some(db) = tt.cs.tone_db
                    && (!db.is_finite() || !(60.0..=100.0).contains(&db))
                {
                    return Err(format!(
                        "Stage {n}: tone intensity must be between 60 and 100 dB."
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Classical-conditioning state of the rat's mind.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassicalMind {
    /// Associative strength of each CS (CS Response Strength), indexed by `CsKind::index`.
    pub v: [Assoc; 3],
    /// Novelty of each CS; the orienting response habituates as it falls.
    pub novelty: [f64; 3],
    pub fear: f64,
    /// Pain sensitivity: predicts the duration of the next unconditioned response to shock.
    pub pain: f64,
}

impl Default for ClassicalMind {
    fn default() -> Self {
        Self {
            v: [Assoc::default(); 3],
            novelty: [1.0; 3],
            fear: 0.0,
            pain: 1.0,
        }
    }
}

impl ClassicalMind {
    pub fn strength(&self, cs: CsKind) -> f64 {
        self.v[cs.index()].net()
    }

    /// Combined prediction of the stimuli present.
    pub fn prediction(&self, cs: &CsSpec) -> f64 {
        CsKind::ALL
            .iter()
            .filter(|k| cs.has(**k))
            .map(|k| self.strength(*k))
            .sum()
    }

    /// Rescorla-Wagner: every stimulus present changes by
    /// `alpha * beta * (lambda - sum of V of the stimuli present)`.
    pub fn rescorla_wagner(&mut self, cs: &CsSpec, us: UsLevel) {
        let error = us.lambda() - self.prediction(cs);
        let beta = if us == UsLevel::None {
            BETA_NO_US
        } else {
            BETA_US
        };
        for k in CsKind::ALL {
            if cs.has(k) {
                self.v[k.index()].adjust(cs.salience(k) * beta * error);
            }
        }
    }

    /// Fear moves toward `target`: quickly up, slowly down.
    pub fn update_fear(&mut self, dt: f64, target: f64) {
        let tau = if target > self.fear { 0.8 } else { 8.0 };
        self.fear += (target - self.fear) * (1.0 - (-dt / tau).exp());
    }

    /// Strong shocks sensitize, weak ones habituate. Returns the duration of the response.
    pub fn shocked(&mut self, us: UsLevel) -> f64 {
        let ma = us.milliamps();
        self.pain = (self.pain + 0.05 * (ma - 0.75)).clamp(0.4, 2.0);
        self.fear = self.fear.max((0.35 * ma).min(1.0));
        0.8 + 1.2 * self.pain * ma
    }

    /// Orienting response to the onset of `cs`: returns whether the rat orients, and habituates.
    pub fn orient_probability(&mut self, cs: &CsSpec) -> f64 {
        let mut p: f64 = 0.0;
        for k in CsKind::ALL {
            if cs.has(k) {
                p = p.max(self.novelty[k.index()]);
                self.novelty[k.index()] *= 0.7;
            }
        }
        p
    }

    pub fn rest(&mut self, hours: f64) {
        for v in &mut self.v {
            v.rest(hours, 24.0);
        }
        let recovery = 1.0 - 0.5_f64.powf(hours / 24.0);
        for n in &mut self.novelty {
            *n += (1.0 - *n) * recovery;
        }
        self.pain += (1.0 - self.pain) * recovery;
        self.fear = 0.0;
    }
}

/// Where a running experiment is in its current trial.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Phase {
    /// Waiting between trials; the CS comes on at `cs_at`.
    Iti { cs_at: f64 },
    /// The CS is on since `start`.
    Cs { start: f64, shocked: bool },
}

/// A classical conditioning experiment in progress.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassicalRun {
    pub design: ClassicalDesign,
    pub stage: usize,
    /// Trial types of the current stage, in presentation order.
    pub order: Vec<usize>,
    pub pos: usize,
    pub total_trial: u32,
    pub phase: Phase,
    pub pre_presses: u32,
    pub cs_presses: u32,
    pub pre_move: f64,
    pub cs_move: f64,
    pub fear_peak: f64,
}

impl ClassicalRun {
    pub fn current_type(&self) -> &TrialType {
        &self.design.stages[self.stage].trial_types[self.order[self.pos]]
    }

    pub fn in_pre_window(&self, now: f64) -> bool {
        matches!(self.phase, Phase::Iti { cs_at } if now >= cs_at - PRE_CS_WINDOW)
    }

    pub fn in_cs(&self) -> bool {
        matches!(self.phase, Phase::Cs { .. })
    }
}

/// `during / (during + before)`, or nothing when both are zero.
pub fn ratio(during: f64, before: f64) -> Option<f64> {
    let total = during + before;
    if total <= 1e-9 {
        None
    } else {
        Some(during / total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquisition_approaches_lambda() {
        let mut m = ClassicalMind::default();
        let cs = CsSpec::light();
        for _ in 0..60 {
            m.rescorla_wagner(&cs, UsLevel::Medium);
        }
        assert!((m.strength(CsKind::Light) - UsLevel::Medium.lambda()).abs() < 0.01);
    }

    #[test]
    fn louder_tones_condition_faster() {
        let mut soft = ClassicalMind::default();
        let mut loud = ClassicalMind::default();
        for _ in 0..3 {
            soft.rescorla_wagner(&CsSpec::tone(70.0), UsLevel::Medium);
            loud.rescorla_wagner(&CsSpec::tone(95.0), UsLevel::Medium);
        }
        assert!(loud.strength(CsKind::Tone) > soft.strength(CsKind::Tone));
    }

    #[test]
    fn stronger_shock_supports_more_conditioning() {
        let mut low = ClassicalMind::default();
        let mut high = ClassicalMind::default();
        for _ in 0..40 {
            low.rescorla_wagner(&CsSpec::light(), UsLevel::Low);
            high.rescorla_wagner(&CsSpec::light(), UsLevel::High);
        }
        assert!(high.strength(CsKind::Light) > low.strength(CsKind::Light) + 0.3);
    }

    #[test]
    fn extinction_and_spontaneous_recovery() {
        let mut m = ClassicalMind::default();
        for _ in 0..30 {
            m.rescorla_wagner(&CsSpec::light(), UsLevel::High);
        }
        for _ in 0..80 {
            m.rescorla_wagner(&CsSpec::light(), UsLevel::None);
        }
        let extinguished = m.strength(CsKind::Light);
        assert!(extinguished < 0.05);
        m.rest(24.0);
        assert!(m.strength(CsKind::Light) > extinguished + 0.3);
    }

    #[test]
    fn compound_conditioning_blocks() {
        // Light alone first, then light + tone: the tone gains little (blocking).
        let mut m = ClassicalMind::default();
        for _ in 0..40 {
            m.rescorla_wagner(&CsSpec::light(), UsLevel::High);
        }
        let compound = CsSpec {
            light: true,
            tone_db: Some(80.0),
            bell: false,
        };
        for _ in 0..20 {
            m.rescorla_wagner(&compound, UsLevel::High);
        }
        assert!(m.strength(CsKind::Tone) < 0.1);
    }

    #[test]
    fn validation_rejects_bad_designs() {
        let mut d = ClassicalDesign::default();
        assert!(d.validate().is_ok());
        d.stages[0].trial_types[0].count = 0;
        assert!(d.validate().is_err());
        d = ClassicalDesign::default();
        d.stages[0].trial_types[0].cs = CsSpec {
            light: false,
            tone_db: None,
            bell: false,
        };
        assert!(d.validate().is_err());
    }

    #[test]
    fn ratio_handles_zero() {
        assert_eq!(ratio(0.0, 0.0), None);
        assert_eq!(ratio(5.0, 5.0), Some(0.5));
        assert_eq!(ratio(0.0, 10.0), Some(0.0));
    }
}
