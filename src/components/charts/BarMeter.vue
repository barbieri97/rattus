<script setup lang="ts">
// A labelled horizontal meter, used by the mind windows.
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    label: string;
    value: number;
    min?: number;
    max?: number;
    color?: string;
    digits?: number;
    hint?: string;
  }>(),
  { min: 0, max: 1, color: "#2f6db3", digits: 2, hint: "" },
);

const span = computed(() => props.max - props.min);
const clamp = (v: number) => Math.min(props.max, Math.max(props.min, v));
const zero = computed(() => ((clamp(0) - props.min) / span.value) * 100);
const end = computed(() => ((clamp(props.value) - props.min) / span.value) * 100);
const left = computed(() => Math.min(zero.value, end.value));
const width = computed(() => Math.abs(end.value - zero.value));
</script>

<template>
  <div class="meter" :title="hint">
    <div class="label">{{ label }}</div>
    <div class="track">
      <div class="fill" :style="{ left: `${left}%`, width: `${width}%`, background: color }" />
      <div v-if="min < 0" class="zero" :style="{ left: `${zero}%` }" />
    </div>
    <div class="value">{{ Number.isFinite(value) ? value.toFixed(digits) : "–" }}</div>
  </div>
</template>

<style scoped>
.meter {
  display: grid;
  grid-template-columns: minmax(90px, 38%) 1fr 44px;
  align-items: center;
  gap: 8px;
  margin: 5px 0;
}
.label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.track {
  position: relative;
  height: 14px;
  background: #e3e8ee;
  border: 1px solid #c3ccd6;
  border-radius: 3px;
  overflow: hidden;
}
.fill {
  position: absolute;
  top: 0;
  bottom: 0;
  transition:
    width 0.12s linear,
    left 0.12s linear;
}
.zero {
  position: absolute;
  top: -2px;
  bottom: -2px;
  width: 1px;
  background: #4d5866;
}
.value {
  text-align: right;
  font-variant-numeric: tabular-nums;
}
</style>
