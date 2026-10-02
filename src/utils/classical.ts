// Helpers for classical conditioning designs (mirroring the checks done in Rust).
import type { ClassicalDesign } from "../bindings/ClassicalDesign";
import type { Stage } from "../bindings/Stage";
import type { TrialType } from "../bindings/TrialType";

export function defaultTrialType(): TrialType {
  return { cs: { light: true, toneDb: null, bell: false }, us: "medium", count: 10 };
}

export function defaultStage(): Stage {
  return { trialTypes: [defaultTrialType()], meanItiMinutes: 5 };
}

export function defaultDesign(): ClassicalDesign {
  return { stages: [defaultStage()] };
}

export function cloneDesign(d: ClassicalDesign): ClassicalDesign {
  return JSON.parse(JSON.stringify(d)) as ClassicalDesign;
}

export function totalTrials(d: ClassicalDesign): number {
  return d.stages.reduce((sum, s) => sum + s.trialTypes.reduce((n, t) => n + t.count, 0), 0);
}

/** Expected program time of the experiment, in minutes. */
export function estimatedMinutes(d: ClassicalDesign): number {
  return d.stages.reduce((sum, s) => {
    const n = s.trialTypes.reduce((k, t) => k + t.count, 0);
    return sum + n * (Math.max(1, s.meanItiMinutes) + 0.5);
  }, 0);
}

export function validateDesign(d: ClassicalDesign): string | null {
  if (d.stages.length === 0) return "The experiment needs at least one stage.";
  if (d.stages.length > 20) return "An experiment can have at most 20 stages.";
  for (const [i, s] of d.stages.entries()) {
    const n = i + 1;
    if (s.trialTypes.length === 0 || s.trialTypes.length > 4)
      return `Stage ${n} must have between 1 and 4 trial types.`;
    if (!(s.meanItiMinutes >= 1 && s.meanItiMinutes <= 60)) {
      return `Stage ${n}: the inter-trial interval must be between 1 and 60 minutes.`;
    }
    for (const t of s.trialTypes) {
      if (!Number.isInteger(t.count) || t.count < 1 || t.count > 200) {
        return `Stage ${n}: each trial type needs between 1 and 200 trials.`;
      }
      if (!t.cs.light && t.cs.toneDb === null && !t.cs.bell) return `Stage ${n}: every trial must present a stimulus.`;
      if (t.cs.toneDb !== null && !(t.cs.toneDb >= 60 && t.cs.toneDb <= 100)) {
        return `Stage ${n}: tone intensity must be between 60 and 100 dB.`;
      }
    }
  }
  return null;
}
