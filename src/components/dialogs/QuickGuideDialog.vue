<script setup lang="ts">
// Short, step-by-step instructions for the classic exercises.
import { ref } from "vue";
import { useUiStore } from "../../stores/ui";
import ModalDialog from "./ModalDialog.vue";

const ui = useUiStore();

const SECTIONS: { title: string; steps: string[]; look: string }[] = [
  {
    title: "Magazine training",
    steps: [
      "File ▸ New Naive Rat.",
      "Press Space (or click the lever) to give a pellet whenever the rat is away from the food cup. Wait until it has eaten before giving the next one.",
      "Repeat until the rat runs to the cup as soon as it hears the click (about 30–50 pellets).",
    ],
    look: "Operant Associations: Sound–Food rises toward 1. The click has become a conditioned (secondary) reinforcer.",
  },
  {
    title: "Shaping bar pressing",
    steps: [
      "After magazine training, give a pellet when the rat is on the right side of the chamber, facing the lever.",
      "Then only when it rears near the lever, then only when it touches or presses it.",
      "Presses are reinforced automatically (CRF), so stop once the rat presses on its own.",
    ],
    look: "Bar–Sound and Action Strength grow; the cumulative record becomes steep and steady.",
  },
  {
    title: "Extinction and spontaneous recovery",
    steps: [
      "With a rat that presses reliably, open Experiment ▸ Design Operant Conditioning Experiment and choose Reinforcer: None.",
      "Isolate the rat to accelerate time until pressing almost stops.",
      "Experiment ▸ Remove Rat for Time Off (24 h), then watch the first minutes back.",
    ],
    look: "The record flattens during extinction; after the rest, pressing comes back for a while (spontaneous recovery).",
  },
  {
    title: "Secondary reinforcement",
    steps: [
      "Train two rats (or save the trained rat and reopen it) on CRF.",
      "Extinguish one with Reinforcer: None and the other with Reinforcer: Sound only.",
    ],
    look: "With the click alone, the rat keeps pressing much longer: the click still reinforces while Sound–Food fades.",
  },
  {
    title: "Schedules of reinforcement",
    steps: [
      "Train bar pressing, then move gradually to the schedule you want (for example VR-5, VR-10, VR-25).",
      "Try FR, VR, FI and VI and compare the cumulative records.",
      "Extinguish after CRF and after VR-25 to see the partial reinforcement effect.",
    ],
    look: "FR: pause after each reinforcement, then a fast run. VR: high, steady rate. FI: scallops. VI: moderate, steady rate.",
  },
  {
    title: "Shaping other behaviours",
    steps: [
      "Reinforce rearing to make begging more likely; reinforce grooming to get face wiping or rolling over.",
      "As soon as the target behaviour appears, reinforce only that.",
      "Tick “Show the strength of every action” in Operant Associations to follow your progress.",
    ],
    look: "The Behavior Log shows what the rat does; the action strength of the target grows.",
  },
  {
    title: "Classical conditioning of fear (CER)",
    steps: [
      "File ▸ New Bar-Trained Rat: it presses steadily on VR-25.",
      "Experiment ▸ Design Classical Conditioning Experiment: e.g. stage 1, 10 trials Light + Medium shock; stage 2, 30 trials Light alone.",
      "Save and Run, then isolate the rat to accelerate time.",
    ],
    look: "Suppression Ratio falls from about 0.5 toward 0 as the CS gains strength, and recovers during extinction. Louder tones and stronger shocks condition faster.",
  },
];

const index = ref(0);
</script>

<template>
  <ModalDialog title="Quick Guide" :width="720" @close="ui.closeDialog()">
    <div class="guide">
      <nav>
        <button v-for="(s, i) in SECTIONS" :key="s.title" :class="{ primary: i === index }" @click="index = i">
          {{ s.title }}
        </button>
      </nav>
      <article>
        <h3>{{ SECTIONS[index].title }}</h3>
        <ol>
          <li v-for="step in SECTIONS[index].steps" :key="step">{{ step }}</li>
        </ol>
        <p><strong>What to look for:</strong> {{ SECTIONS[index].look }}</p>
        <p class="muted small">
          Shortcuts: Space gives a pellet · P pauses · Ctrl+I isolates the rat (accelerates time) · Ctrl+M marks the
          record · Ctrl+C copies the data of the selected window.
        </p>
      </article>
    </div>
    <template #footer>
      <button class="primary" @click="ui.closeDialog()">Close</button>
    </template>
  </ModalDialog>
</template>

<style scoped>
.guide {
  display: flex;
  gap: 16px;
  min-height: 300px;
}
nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 210px;
  flex-shrink: 0;
}
nav button {
  text-align: left;
}
article {
  flex: 1;
}
h3 {
  margin: 0 0 8px;
}
li {
  margin-bottom: 4px;
}
.small {
  font-size: 12px;
}
</style>
