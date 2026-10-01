<script setup lang="ts">
// The rat, drawn in profile facing right in its own coordinates: the origin is on the floor
// under the middle of the body, one unit is about one millimetre of rat. The parent places,
// flips and scales it.
import { computed } from "vue";
import type { Pose } from "../animation/pose";

const props = defineProps<{ pose: Pose; time: number }>();

const HIP = { x: -30, y: -14 };
const SHOULDER = { x: 20, y: -16 };
const NECK = { x: 34, y: -30 };

/** Rotates point (x, y) by `deg` degrees around `at`. */
function rotate(x: number, y: number, deg: number, at: { x: number; y: number }) {
  const r = (deg * Math.PI) / 180;
  const dx = x - at.x;
  const dy = y - at.y;
  return { x: at.x + dx * Math.cos(r) - dy * Math.sin(r), y: at.y + dx * Math.sin(r) + dy * Math.cos(r) };
}

// The tail is drawn outside the pitched body, so that it trails along the floor even when
// the rat rears up.
const tail = computed(() => {
  const p = props.pose;
  const base = rotate(-60, -17, p.pitch, HIP);
  const bx = base.x + p.shift;
  const by = Math.min(base.y + p.lift, -3);
  const c = p.tail;
  const w = p.tailWave * Math.sin(props.time * 3.1);
  const w2 = p.tailWave * Math.sin(props.time * 3.1 - 1.2);
  const endY = Math.min(-2, -4 - 28 * c - 8 * w + p.lift * 0.5);
  return `M ${bx} ${by} C ${bx - 18} ${by + 3 + 6 * w}, ${bx - 38} ${-6 - 16 * c + 10 * w2}, ${bx - 66} ${endY}`;
});

const bodyTransform = computed(
  () => `translate(${props.pose.shift}, ${props.pose.lift}) rotate(${props.pose.pitch}, ${HIP.x}, ${HIP.y})`,
);
const stretchTransform = computed(
  () => `translate(${HIP.x}, 0) scale(${props.pose.stretch}, 1) translate(${-HIP.x}, 0)`,
);
const headTransform = computed(
  () =>
    `translate(${NECK.x + props.pose.headX * props.pose.stretch}, ${NECK.y + props.pose.headY}) rotate(${props.pose.head})`,
);
const limb = (angle: number, at: { x: number; y: number }) => `rotate(${angle}, ${at.x}, ${at.y})`;
const sideOpacity = computed(() => 1 - props.pose.front);
</script>

<template>
  <g class="rat">
    <path class="tail" :d="tail" />
    <g :transform="bodyTransform">
      <!-- far limbs -->
      <g :transform="limb(pose.hindFar, HIP)">
        <ellipse class="far" cx="-24" cy="-11" rx="12" ry="9" />
        <ellipse class="paw far-paw" cx="-14" cy="-1" rx="9" ry="2.6" />
      </g>
      <g :transform="limb(pose.foreFar, SHOULDER)">
        <path class="limb-line far-line" d="M 26 -16 L 29 -2" />
        <path class="limb-fill far-fill" d="M 26 -16 L 29 -2" />
        <ellipse class="paw far-paw" cx="30" cy="-1" rx="4.5" ry="2.4" />
      </g>
      <!-- body -->
      <g :transform="stretchTransform">
        <path
          class="fur"
          d="M -62 -22 C -64 -46 -30 -54 0 -50 C 22 -47 40 -40 42 -28 C 44 -14 32 -6 14 -6 L -40 -6 C -56 -6 -62 -12 -62 -22 Z"
        />
        <path class="shade" d="M -54 -16 C -40 -10 -10 -9 20 -11" />
      </g>
      <!-- head -->
      <g :transform="headTransform">
        <g :opacity="sideOpacity">
          <g :transform="`rotate(${pose.ear}, 4, -8)`">
            <ellipse class="fur" cx="4" cy="-15" rx="7.5" ry="9.5" />
            <ellipse class="pink" cx="4.5" cy="-14.5" rx="4.5" ry="6" />
          </g>
          <path
            class="fur"
            d="M -8 -10 C 2 -20 22 -18 38 -8 C 44 -4 44 2 38 4 C 26 9 6 12 -4 8 C -12 4 -12 -6 -8 -10 Z"
          />
          <path class="mouth" :d="`M 28 6 Q 33 ${7.5 + pose.chew * 2.5} 37 ${5 + pose.chew}`" />
          <circle class="eye" cx="21" cy="-5" r="2.8" />
          <circle cx="21.8" cy="-6" r="0.9" fill="#ffffff" />
          <circle class="nose" cx="41" cy="0" r="2.6" />
          <g class="whiskers">
            <path :d="`M 37 1 L 56 ${-5 + pose.whisker * 2}`" />
            <path :d="`M 37 2 L 58 ${2 + pose.whisker}`" />
            <path :d="`M 36 3 L 54 ${9 - pose.whisker * 2}`" />
          </g>
        </g>
        <g v-if="pose.front > 0.01" :opacity="pose.front" transform="translate(4, -2)">
          <ellipse class="fur" cx="-2" cy="-14" rx="6.5" ry="8.5" />
          <ellipse class="pink" cx="-2" cy="-13.5" rx="4" ry="5.5" />
          <ellipse class="fur" cx="26" cy="-14" rx="6.5" ry="8.5" />
          <ellipse class="pink" cx="26" cy="-13.5" rx="4" ry="5.5" />
          <ellipse class="fur" cx="12" cy="-2" rx="14" ry="12" />
          <circle class="eye" cx="7" cy="-5" r="2.4" />
          <circle class="eye" cx="17" cy="-5" r="2.4" />
          <circle class="nose" cx="12" cy="3" r="2.4" />
          <g class="whiskers">
            <path d="M 9 4 L -6 1" />
            <path d="M 9 5 L -6 8" />
            <path d="M 15 4 L 30 1" />
            <path d="M 15 5 L 30 8" />
          </g>
        </g>
      </g>
      <!-- near limbs -->
      <g :transform="limb(pose.hind, HIP)">
        <ellipse class="thigh" cx="-30" cy="-12" rx="15" ry="11.5" />
        <path class="outline" d="M -44 -8 C -42 2 -22 3 -17 -6" />
        <path class="limb-line" d="M -24 -6 L -18 -1" />
        <path class="limb-fill" d="M -24 -6 L -18 -1" />
        <ellipse class="paw" cx="-16" cy="-1" rx="10" ry="2.8" />
      </g>
      <g :transform="limb(pose.fore, SHOULDER)">
        <path class="limb-line" d="M 20 -16 L 23 -2" />
        <path class="limb-fill" d="M 20 -16 L 23 -2" />
        <ellipse class="paw" cx="24.5" cy="-1" rx="5" ry="2.6" />
      </g>
    </g>
  </g>
</template>

<style scoped>
.fur {
  fill: #fbfcfd;
  stroke: #3a4a5d;
  stroke-width: 1.4;
}
.thigh {
  fill: #fbfcfd;
}
.outline {
  fill: none;
  stroke: #3a4a5d;
  stroke-width: 1.4;
  stroke-linecap: round;
}
.shade {
  fill: none;
  stroke: #d9e0e8;
  stroke-width: 3;
  stroke-linecap: round;
}
.far {
  fill: #dfe5ec;
  stroke: #3a4a5d;
  stroke-width: 1.2;
}
.pink,
.paw {
  fill: #f2a5b5;
}
.paw {
  stroke: #3a4a5d;
  stroke-width: 0.8;
}
.far-paw {
  fill: #dd8f9f;
}
.limb-line,
.limb-fill {
  stroke-linecap: round;
  fill: none;
}
.limb-line {
  stroke: #3a4a5d;
  stroke-width: 7.5;
}
.limb-fill {
  stroke: #fbfcfd;
  stroke-width: 5;
}
.far-line {
  stroke: #56667a;
}
.far-fill {
  stroke: #dfe5ec;
}
.tail {
  fill: none;
  stroke: #eb9aab;
  stroke-width: 4.2;
  stroke-linecap: round;
}
.eye {
  fill: #b8213a;
}
.nose {
  fill: #ee7d94;
}
.mouth {
  fill: none;
  stroke: #3a4a5d;
  stroke-width: 1;
  stroke-linecap: round;
}
.whiskers path {
  stroke: #56667a;
  stroke-width: 0.7;
  stroke-linecap: round;
}
</style>
