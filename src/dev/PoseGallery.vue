<script setup lang="ts">
// Development aid (open the dev server with ?gallery): every behaviour drawn in the chamber
// at a chosen moment of its bout, to tune the drawing and the positions of the devices.
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { poseFor } from "../animation/pose";
import type { Behavior } from "../bindings/Behavior";
import type { ChamberView } from "../bindings/ChamberView";
import ChamberScene from "../components/ChamberScene.vue";
import { BEHAVIOR_LABELS } from "../utils/labels";

const progress = ref(0.5);
const zoomed = new URLSearchParams(location.search).has("zoom");
const playing = ref(false);
const time = ref(1);

const PLACES: Partial<Record<Behavior, { x: number; z: number; facing: number }>> = {
  barPress: { x: 0.86, z: 0.45, facing: 1 },
  rearWall: { x: 0.86, z: 0.45, facing: 1 },
  eat: { x: 0.86, z: 0.45, facing: 1 },
  drink: { x: 0.1, z: 0.45, facing: -1 },
};

const behaviors = Object.keys(BEHAVIOR_LABELS) as Behavior[];
const chamber: ChamberView = {
  pellets: 3,
  leverDown: false,
  lightOn: false,
  toneDb: null,
  bellOn: false,
  shock: null,
  dispenseCount: 0,
  shockCount: 0,
};

const cards = computed(() =>
  behaviors.map((b) => {
    const place = PLACES[b] ?? { x: 0.5, z: 0.5, facing: 1 };
    const view = !zoomed
      ? undefined
      : place.x > 0.8
        ? "640 300 360 280"
        : place.x < 0.2
          ? "0 300 360 280"
          : "330 300 360 280";
    const duration = 2;
    return {
      behavior: b,
      view,
      chamber: { ...chamber, leverDown: b === "barPress" && Math.abs(progress.value - 0.5) < 0.15 },
      rat: { ...place, pose: poseFor(b, progress.value * duration, progress.value, time.value * 6), time: time.value },
    };
  }),
);

let raf = 0;
let start = 0;
function tick(now: number) {
  raf = requestAnimationFrame(tick);
  if (!playing.value) return;
  if (!start) start = now;
  const s = (now - start) / 1000;
  time.value = s;
  progress.value = (s / 2.5) % 1;
}
onMounted(() => (raf = requestAnimationFrame(tick)));
onBeforeUnmount(() => cancelAnimationFrame(raf));
</script>

<template>
  <div class="gallery">
    <header class="row">
      <strong>Pose gallery</strong>
      <label class="row"
        >progress <input v-model.number="progress" type="range" min="0" max="1" step="0.01" />
        {{ progress.toFixed(2) }}</label
      >
      <label><input v-model="playing" type="checkbox" /> play</label>
    </header>
    <div class="grid">
      <figure v-for="c in cards" :key="c.behavior">
        <div class="scene"><ChamberScene :rat="c.rat" :chamber="c.chamber" :view-box="c.view" /></div>
        <figcaption>{{ BEHAVIOR_LABELS[c.behavior] }}</figcaption>
      </figure>
    </div>
  </div>
</template>

<style scoped>
.gallery {
  height: 100%;
  overflow: auto;
  padding: 10px;
  user-select: none;
}
.grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}
figure {
  margin: 0;
  min-width: 0;
  background: #ffffff;
  border: 1px solid #c4ccd6;
}
.scene {
  height: 230px;
}
figcaption {
  padding: 2px 6px;
}
header {
  margin-bottom: 8px;
}
</style>
