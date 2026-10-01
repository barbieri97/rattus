<script setup lang="ts">
// Frame for modal dialogs: title, content and a button row. Escape closes it.
import { onBeforeUnmount, onMounted, ref } from "vue";

defineProps<{ title: string; width?: number }>();
const emit = defineEmits<{ close: [] }>();
const root = ref<HTMLElement | null>(null);

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("close");
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKey, true);
  root.value?.querySelector<HTMLElement>("input, select, button.primary")?.focus();
});
onBeforeUnmount(() => window.removeEventListener("keydown", onKey, true));
</script>

<template>
  <div class="backdrop" @pointerdown.self="emit('close')">
    <div ref="root" class="dialog" role="dialog" aria-modal="true" :style="{ width: `${width ?? 460}px` }">
      <header>{{ title }}</header>
      <div class="content">
        <slot />
      </div>
      <footer>
        <slot name="footer" />
      </footer>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  background: rgba(20, 30, 45, 0.32);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}
.dialog {
  max-width: calc(100vw - 32px);
  max-height: calc(100vh - 32px);
  display: flex;
  flex-direction: column;
  background: var(--panel);
  border: 1px solid var(--panel-border);
  border-radius: 8px;
  box-shadow: 0 12px 40px rgba(20, 30, 45, 0.35);
  user-select: text;
}
header {
  padding: 10px 16px;
  font-weight: 600;
  font-size: 14px;
  border-bottom: 1px solid #dde3ea;
}
.content {
  padding: 12px 16px;
  overflow: auto;
}
footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 10px 16px;
  border-top: 1px solid #dde3ea;
}
footer :deep(.left) {
  margin-right: auto;
}
</style>
