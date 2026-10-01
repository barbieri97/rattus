<script setup lang="ts">
// A window inside the workspace that can be moved, resized, focused and closed.
import { computed } from "vue";
import { MIN_H, MIN_W, PANEL_TITLES, useLayoutStore, type PanelId } from "../stores/layout";

const props = withDefaults(defineProps<{ id: PanelId; closable?: boolean }>(), { closable: true });
const layout = useLayoutStore();
const state = computed(() => layout.panels[props.id]);
const active = computed(() => layout.active === props.id);

function drag(e: PointerEvent, mode: "move" | "resize") {
  if (e.button !== 0) return;
  layout.focus(props.id);
  const el = e.currentTarget as HTMLElement;
  el.setPointerCapture(e.pointerId);
  const p0 = state.value;
  const start = { px: e.clientX, py: e.clientY, x0: p0.x, y0: p0.y, w: p0.w, h: p0.h };
  const { width, height } = layout.workspace;
  const onMove = (ev: PointerEvent) => {
    const dx = ev.clientX - start.px;
    const dy = ev.clientY - start.py;
    const p = state.value;
    if (mode === "move") {
      p.x = Math.round(Math.min(Math.max(start.x0 + dx, -p.w + 80), width - 80));
      p.y = Math.round(Math.min(Math.max(start.y0 + dy, 0), height - 28));
    } else {
      p.w = Math.round(Math.max(MIN_W, start.w + dx));
      p.h = Math.round(Math.max(MIN_H, start.h + dy));
    }
  };
  const onUp = () => {
    el.removeEventListener("pointermove", onMove);
    el.removeEventListener("pointerup", onUp);
    el.removeEventListener("pointercancel", onUp);
  };
  el.addEventListener("pointermove", onMove);
  el.addEventListener("pointerup", onUp);
  el.addEventListener("pointercancel", onUp);
}
</script>

<template>
  <section
    v-if="state.open"
    class="panel"
    :class="{ active }"
    :style="{
      left: `${state.x}px`,
      top: `${state.y}px`,
      width: `${state.w}px`,
      height: `${state.h}px`,
      zIndex: state.z,
    }"
    @pointerdown="layout.focus(id)"
  >
    <header class="title" @pointerdown="drag($event, 'move')" @dblclick="layout.focus(id)">
      <span>{{ PANEL_TITLES[id] }}</span>
      <span class="title-extra"><slot name="title" /></span>
      <button v-if="closable" class="close" title="Close" @pointerdown.stop @click="layout.close(id)">×</button>
    </header>
    <div class="body">
      <slot />
    </div>
    <div class="resize" title="Resize" @pointerdown.stop="drag($event, 'resize')" />
  </section>
</template>

<style scoped>
.panel {
  position: absolute;
  display: flex;
  flex-direction: column;
  background: var(--panel);
  border: 1px solid var(--panel-border);
  border-radius: 6px;
  box-shadow: 0 2px 8px rgba(20, 30, 45, 0.12);
  overflow: hidden;
}
.panel.active {
  box-shadow: var(--shadow);
  border-color: #6f8aa8;
}
.title {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 26px;
  padding: 0 4px 0 10px;
  background: var(--title);
  border-bottom: 1px solid #c4ccd6;
  font-weight: 600;
  cursor: grab;
  flex-shrink: 0;
}
.active .title {
  background: var(--title-active);
}
.title-extra {
  flex: 1;
  font-weight: normal;
  display: flex;
  justify-content: flex-end;
  gap: 6px;
}
.close {
  border: none;
  background: transparent;
  font-size: 16px;
  line-height: 1;
  padding: 0 6px;
}
.body {
  flex: 1;
  min-height: 0;
  position: relative;
  overflow: hidden;
}
.resize {
  position: absolute;
  right: 0;
  bottom: 0;
  width: 14px;
  height: 14px;
  cursor: nwse-resize;
  background: linear-gradient(
    135deg,
    transparent 50%,
    #9aa6b4 50%,
    #9aa6b4 60%,
    transparent 60%,
    transparent 72%,
    #9aa6b4 72%,
    #9aa6b4 82%,
    transparent 82%
  );
}
</style>
