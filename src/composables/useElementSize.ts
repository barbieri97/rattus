import { onBeforeUnmount, onMounted, ref, type Ref } from "vue";

/** Tracks the content size of an element. */
export function useElementSize(target: Ref<HTMLElement | null>) {
  const width = ref(0);
  const height = ref(0);
  let observer: ResizeObserver | null = null;
  onMounted(() => {
    if (!target.value) return;
    observer = new ResizeObserver(([entry]) => {
      width.value = Math.floor(entry.contentRect.width);
      height.value = Math.floor(entry.contentRect.height);
    });
    observer.observe(target.value);
  });
  onBeforeUnmount(() => observer?.disconnect());
  return { width, height };
}
