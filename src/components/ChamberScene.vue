<script setup lang="ts">
// The operant chamber seen from the front: lever, food cup and dispenser on the right wall,
// water spout on the left, stimulus light, speaker and bell, and the shock grid floor.
// Purely presentational: ChamberView animates it, the pose gallery shows it frozen.
import { computed } from "vue";
import type { Pose } from "../animation/pose";
import type { ChamberView } from "../bindings/ChamberView";
import RatSprite from "./RatSprite.vue";

export interface RatDrawing {
  /** Position in the chamber, 0..1 left to right and front to back. */
  x: number;
  z: number;
  /** 1 facing right, -1 facing left. */
  facing: number;
  pose: Pose;
  /** Program time, for idle motion such as the tail. */
  time: number;
  jitterX?: number;
  jitterY?: number;
}

const props = withDefaults(
  defineProps<{
    rat: RatDrawing | null;
    chamber: ChamberView | null;
    /** Show the "isolated" overlay with this acceleration. */
    isolatedSpeed?: number | null;
    /** Position of a falling pellet, if any. */
    drop?: { x: number; y: number } | null;
    /** Scale the drawing to its container (otherwise a fixed size). */
    scale?: boolean;
    /** Part of the scene to show (the pose gallery zooms in on the devices). */
    viewBox?: string;
  }>(),
  { isolatedSpeed: null, drop: null, scale: true, viewBox: "0 0 1000 600" },
);
const emit = defineEmits<{ dispense: [] }>();

const WIDTH = 1000;
const HEIGHT = 600;
const RAT_SCALE = 1.25;
/** Rat x (0..1) of the left- and right-most positions, and where they are drawn. */
const X_MIN = 0.1;
const X_MAX = 0.86;
const PX_MIN = 138;
const PX_MAX = 862;

function ratPx(x: number): number {
  return PX_MIN + ((x - X_MIN) * (PX_MAX - PX_MIN)) / (X_MAX - X_MIN);
}
function groundY(z: number): number {
  return 556 - z * 70;
}
function depthScale(z: number): number {
  return RAT_SCALE * (1 + (0.5 - z) * 0.12);
}

const chamber = computed(() => props.chamber);
const isolated = computed(() => props.isolatedSpeed != null);
const shock = computed(() => chamber.value?.shock != null);
const toneOn = computed(() => chamber.value?.toneDb != null);
const pellets = computed(() => Math.min(6, chamber.value?.pellets ?? 0));
const dropPos = computed(() => props.drop ?? null);

const ratTransform = computed(() => {
  const r = props.rat;
  if (!r) return "";
  const s = depthScale(r.z);
  const x = ratPx(r.x) + (r.jitterX ?? 0);
  const y = groundY(r.z) + (r.jitterY ?? 0);
  return `translate(${x.toFixed(2)}, ${y.toFixed(2)}) scale(${(s * r.facing).toFixed(3)}, ${s.toFixed(3)}) rotate(${r.pose.roll.toFixed(1)}, -10, -28)`;
});

const gridBars = Array.from({ length: 31 }, (_, i) => {
  const xb = 52 + i * 29.8;
  const xf = 500 + (xb - 500) * 1.07;
  return { x1: xb, y1: 484, x2: xf, y2: 556 };
});

const svgStyle = computed(() =>
  props.scale ? { width: "100%", height: "100%" } : { width: `${WIDTH * 0.8}px`, height: `${HEIGHT * 0.8}px` },
);
</script>

<template>
  <svg :viewBox="viewBox" preserveAspectRatio="xMidYMid meet" :style="svgStyle">
    <defs>
      <linearGradient id="wall" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#f4f6f1" />
        <stop offset="1" stop-color="#dfe3dc" />
      </linearGradient>
      <linearGradient id="floor" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#8d96a0" />
        <stop offset="1" stop-color="#b5bcc4" />
      </linearGradient>
      <linearGradient id="glass" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#ffffff" stop-opacity="0.18" />
        <stop offset="0.45" stop-color="#ffffff" stop-opacity="0" />
        <stop offset="0.55" stop-color="#ffffff" stop-opacity="0.08" />
        <stop offset="1" stop-color="#ffffff" stop-opacity="0" />
      </linearGradient>
      <radialGradient id="lamp">
        <stop offset="0" stop-color="#fff7c2" />
        <stop offset="1" stop-color="#ffd84a" stop-opacity="0" />
      </radialGradient>
      <filter id="glow" x="-50%" y="-50%" width="200%" height="200%">
        <feGaussianBlur stdDeviation="4" />
      </filter>
    </defs>

    <!-- room -->
    <rect x="0" y="0" :width="WIDTH" :height="HEIGHT" fill="#c7d0da" />
    <rect x="0" y="560" :width="WIDTH" height="40" fill="#a9b3be" />

    <!-- water bottle outside the left wall -->
    <rect x="6" y="170" width="30" height="190" rx="8" fill="#cfe3f3" stroke="#6b7c8f" stroke-width="2" />
    <rect x="9" y="230" width="24" height="127" rx="6" fill="#7fb2dc" opacity="0.8" />
    <rect x="14" y="358" width="14" height="18" fill="#8a96a5" />
    <path d="M 21 376 L 21 414 L 58 420" fill="none" stroke="#7c8896" stroke-width="7" stroke-linecap="round" />

    <!-- pellet dispenser and tube outside the right wall -->
    <g class="dispenser" @click="emit('dispense')">
      <title>Pellet dispenser: click to give a pellet</title>
      <rect x="963" y="120" width="34" height="96" rx="4" fill="#b9c2cc" stroke="#5f6d7c" stroke-width="2" />
      <rect x="969" y="130" width="22" height="38" rx="2" fill="#6f5a43" opacity="0.75" />
      <circle cx="980" cy="190" r="6" fill="#5f6d7c" />
    </g>
    <path
      d="M 981 214 L 981 470 L 950 497"
      fill="none"
      stroke="#9aa5b1"
      stroke-width="10"
      stroke-linejoin="round"
      opacity="0.8"
    />

    <!-- chamber back wall -->
    <rect x="40" y="50" width="920" height="436" fill="url(#wall)" stroke="#6b7787" stroke-width="3" />
    <rect x="40" y="50" width="920" height="22" fill="#b4bdc7" />
    <!-- house light -->
    <rect x="470" y="72" width="60" height="10" rx="3" fill="#fff6d8" stroke="#a69d7e" />

    <!-- speaker -->
    <g transform="translate(92, 120)">
      <rect x="-26" y="-22" width="52" height="44" rx="6" fill="#4d5866" />
      <circle r="14" fill="#2e3742" />
      <circle r="5" fill="#59667a" />
      <g v-if="toneOn && !isolated" class="waves" fill="none" stroke="#2f6db3" stroke-width="3" stroke-linecap="round">
        <path d="M 34 -12 Q 42 0 34 12" />
        <path d="M 44 -20 Q 56 0 44 20" />
        <path d="M 54 -28 Q 70 0 54 28" />
      </g>
      <text x="0" y="40" text-anchor="middle" class="tag">speaker</text>
    </g>

    <!-- bell -->
    <g transform="translate(200, 104)">
      <g :class="{ ringing: chamber?.bellOn && !isolated }">
        <path d="M -16 14 Q -16 -12 0 -14 Q 16 -12 16 14 Z" fill="#d4a93b" stroke="#7d6220" stroke-width="2" />
        <rect x="-20" y="13" width="40" height="5" rx="2" fill="#a37f22" />
        <circle cy="20" r="3.5" fill="#7d6220" />
      </g>
      <text x="0" y="40" text-anchor="middle" class="tag">bell</text>
    </g>

    <!-- stimulus light above the lever -->
    <g transform="translate(904, 318)">
      <circle v-if="chamber?.lightOn" r="44" fill="url(#lamp)" />
      <circle r="14" :fill="chamber?.lightOn ? '#fff1a0' : '#d9d4c2'" stroke="#857f69" stroke-width="2.5" />
      <text x="0" y="34" text-anchor="middle" class="tag">light</text>
    </g>

    <!-- floor: shock grid -->
    <polygon points="40,484 960,484 994,556 6,556" fill="url(#floor)" />
    <g :class="['grid', { shocked: shock }]">
      <line v-for="(b, i) in gridBars" :key="i" :x1="b.x1" :y1="b.y1" :x2="b.x2" :y2="b.y2" />
    </g>
    <g v-if="shock" class="bolts" fill="#ffd23f" stroke="#a67c00" stroke-width="1.5">
      <path d="M 300 470 l 12 -22 l -4 12 l 12 -3 l -14 24 l 4 -12 Z" />
      <path d="M 640 474 l 12 -22 l -4 12 l 12 -3 l -14 24 l 4 -12 Z" />
    </g>

    <!-- food cup -->
    <g transform="translate(0, 0)">
      <rect x="904" y="500" width="52" height="24" rx="4" fill="#7b8794" stroke="#4d5866" stroke-width="2" />
      <rect x="910" y="504" width="40" height="10" rx="3" fill="#3c4652" />
      <circle
        v-for="i in pellets"
        :key="i"
        :cx="914 + ((i - 1) % 6) * 6.5"
        :cy="510 - Math.floor((i - 1) / 6) * 4"
        r="3.4"
        fill="#8b5a2b"
        stroke="#5c3a19"
        stroke-width="0.8"
      />
      <text x="930" y="540" text-anchor="middle" class="tag">food cup</text>
    </g>

    <!-- lever (behind the rat, so its paws rest on it) -->
    <g class="lever" @click="emit('dispense')">
      <title>Lever: click to give a pellet by hand</title>
      <rect x="936" y="426" width="20" height="44" rx="3" fill="#8e99a6" stroke="#4d5866" stroke-width="2" />
      <g :style="{ transform: chamber?.leverDown ? 'rotate(-12deg)' : 'rotate(0deg)' }" class="lever-arm">
        <rect x="902" y="443" width="44" height="9" rx="3" fill="#e2b04a" stroke="#7d6220" stroke-width="1.5" />
      </g>
      <text x="925" y="418" text-anchor="middle" class="tag">lever</text>
    </g>

    <!-- the rat -->
    <g v-if="rat && !isolated" :transform="ratTransform">
      <RatSprite :pose="rat.pose" :time="rat.time" />
    </g>

    <!-- falling pellet -->
    <circle v-if="dropPos" :cx="dropPos.x" :cy="dropPos.y" r="4" fill="#8b5a2b" />

    <!-- glass front -->
    <rect x="40" y="50" width="920" height="506" fill="url(#glass)" pointer-events="none" />

    <g v-if="isolated">
      <rect x="200" y="210" width="600" height="150" rx="14" fill="#1f2a36" opacity="0.86" />
      <text x="500" y="270" text-anchor="middle" class="overlay-title">Rat isolated</text>
      <text x="500" y="308" text-anchor="middle" class="overlay-text">
        Time runs {{ isolatedSpeed }}× faster than real time.
      </text>
      <text x="500" y="336" text-anchor="middle" class="overlay-text">
        Choose Experiment ▸ Isolate Rat again to watch.
      </text>
    </g>
  </svg>
</template>

<style scoped>
svg {
  display: block;
}
.tag {
  font-size: 13px;
  fill: #6a7685;
}
.grid line {
  stroke: #59636e;
  stroke-width: 3;
}
.grid.shocked line {
  stroke: #ffd23f;
  stroke-width: 4;
}
.lever,
.dispenser {
  cursor: pointer;
}
.lever-arm {
  transform-box: view-box;
  transform-origin: 946px 447px;
  transition: transform 70ms ease-out;
}
.waves path {
  animation: pulse 0.6s infinite alternate;
}
.waves path:nth-child(2) {
  animation-delay: 0.2s;
}
.waves path:nth-child(3) {
  animation-delay: 0.4s;
}
@keyframes pulse {
  from {
    opacity: 0.2;
  }
  to {
    opacity: 1;
  }
}
.ringing {
  animation: ring 0.15s infinite alternate;
  transform-origin: 0 -14px;
}
@keyframes ring {
  from {
    transform: rotate(-14deg);
  }
  to {
    transform: rotate(14deg);
  }
}
.overlay-title {
  font-size: 34px;
  font-weight: 700;
  fill: #ffffff;
}
.overlay-text {
  font-size: 17px;
  fill: #cfdbe8;
}
</style>
