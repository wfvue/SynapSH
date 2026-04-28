<script setup lang="ts">
import { computed } from "vue";

interface Props {
  size?: number;
  icon: string;
  background?: string;
  title?: string;
  noShadow?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  size: 56,
  background: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)",
  noShadow: false,
});

const iconSize = computed(() => {
  // Icon size is roughly 55% of container size
  return Math.floor(props.size * 0.55);
});
</script>

<template>
  <div
    class="app-icon relative flex items-center justify-center transition-all duration-200"
    :class="[noShadow ? '' : 'shadow-[0_2px_8px_rgba(0,0,0,0.2)]']"
    :style="{
      width: `${props.size}px`,
      height: `${props.size}px`,
      background: props.background,
      borderRadius: `${Math.max(6, props.size * 0.15)}px`,
    }"
    :title="title"
  >
    <span
      :class="props.icon"
      class="text-white/95 drop-shadow-sm transition-transform duration-200"
      :style="{ fontSize: `${iconSize}px` }"
    ></span>
    <slot></slot>
  </div>
</template>
