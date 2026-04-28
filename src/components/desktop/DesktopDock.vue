<script setup lang="ts">
import { useAppearance } from "../../composables/useAppearance";
import AppIcon from "./AppIcon.vue";

export interface DockItem {
  id: string;
  label: string;
  icon: string;
  color: string;
  app?: string;
}

defineProps<{
  items: DockItem[];
  openApps: string[];
}>();

const emit = defineEmits<{
  openApp: [app: string];
}>();

const { dockIconSize } = useAppearance();

function handleClick(app: string | undefined) {
  if (app) {
    emit("openApp", app);
  }
}
</script>

<template>
  <section
    class="absolute bottom-0 left-0 w-full h-[48px] px-2 flex items-center justify-center gap-1 bg-background/80 backdrop-blur-2xl border-t border-white/10 z-50"
  >
    <!-- 开始按钮占位 -->
    <button
      class="relative flex items-center justify-center w-[40px] h-[40px] rounded hover:bg-white/10 active:bg-white/15 transition-all duration-150 border-none bg-transparent cursor-pointer group flex-shrink-0 mr-2"
      title="开始"
    >
      <span
        class="icon-[mdi--microsoft-windows] text-2xl text-blue-500 group-hover:scale-105 transition-transform"
      ></span>
    </button>

    <button
      v-for="item in items"
      :key="item.id"
      class="relative flex items-center justify-center w-[40px] h-[40px] rounded hover:bg-white/10 active:bg-white/15 transition-all duration-150 border-none bg-transparent cursor-pointer group flex-shrink-0"
      :class="{ 'bg-white/[0.06]': item.app && openApps.includes(item.app) }"
      :title="item.label"
      @click.stop="handleClick(item.app)"
    >
      <AppIcon
        :icon="item.icon"
        :background="item.color"
        :size="28"
        :no-shadow="true"
        class="transition-transform group-hover:scale-95 group-active:scale-90"
      />

      <!-- Win11 Pill Indicator -->
      <span
        class="absolute bottom-0 left-1/2 -translate-x-1/2 h-[3px] rounded-t-full transition-all duration-300"
        :class="
          item.app && openApps.includes(item.app)
            ? 'bg-[#0078d4] w-4 opacity-100'
            : 'bg-white/40 opacity-0 group-hover:opacity-100 group-hover:w-[6px] w-1.5'
        "
      ></span>
    </button>
  </section>
</template>
