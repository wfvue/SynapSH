<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { useColorMode } from "@vueuse/core";

const props = defineProps<{
  isConnected: boolean;
}>();

const timeText = ref("");
const dateText = ref("");
const mode = useColorMode();

let timer: number | undefined;

function updateClock() {
  const now = new Date();
  timeText.value = now.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  dateText.value = now.toLocaleDateString("zh-CN", {
    weekday: "short",
    month: "short",
    day: "numeric",
  });
}

function toggleTheme() {
  mode.value = mode.value === "dark" ? "light" : "dark";
}

onMounted(() => {
  updateClock();
  timer = window.setInterval(updateClock, 1000);
});

onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});
</script>

<template>
  <div class="absolute top-2 inset-x-0 w-full flex justify-center z-50 pointer-events-none">
    <header
      class="pointer-events-auto h-9 bg-black/40 backdrop-blur-xl rounded-full border border-white/10 shadow-[0_4px_24px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,255,255,0.1)] flex justify-between items-center px-5 text-[12px] font-mono text-cyan-50/90 select-none min-w-[500px]"
    >
      <div class="flex items-center gap-5">
        <div
          class="flex items-center text-lg text-cyan-400 hover:text-cyan-300 transition-colors drop-shadow-[0_0_8px_rgba(34,211,238,0.5)]"
        >
          <span class="icon-[mdi--hexagon-multiple]"></span>
        </div>
        <span
          class="font-bold tracking-wider text-cyan-50 drop-shadow-[0_0_5px_rgba(255,255,255,0.5)]"
          >SYNAP_SH</span
        >
        <nav class="flex gap-4 opacity-70">
          <span class="hover:text-cyan-300 hover:opacity-100 transition-colors cursor-pointer"
            >SYS</span
          >
          <span class="hover:text-cyan-300 hover:opacity-100 transition-colors cursor-pointer"
            >NET</span
          >
          <span class="hover:text-cyan-300 hover:opacity-100 transition-colors cursor-pointer"
            >SEC</span
          >
        </nav>
      </div>

      <div class="flex items-center gap-4">
        <div class="flex flex-col items-end opacity-90 leading-none">
          <span class="text-[10px] text-cyan-500">LOCAL_TIME</span>
          <div class="flex gap-2 font-bold tracking-widest text-[#e2e8f0]">
            <span>{{ dateText }}</span>
            <span class="w-[60px] text-right">{{ timeText }}</span>
          </div>
        </div>

        <div class="h-4 w-px bg-white/20 mx-1"></div>

        <div
          class="flex items-center opacity-90 cursor-pointer hover:text-cyan-300 hover:drop-shadow-[0_0_5px_rgba(34,211,238,0.8)] transition-all"
          @click="toggleTheme"
          title="Toggle Theme"
        >
          <span v-if="mode === 'dark'" class="icon-[mdi--weather-night] text-lg"></span>
          <span v-else class="icon-[mdi--weather-sunny] text-lg"></span>
        </div>

        <div
          class="flex items-center opacity-90 cursor-pointer hover:text-cyan-300 transition-all"
          title="Uplink Status"
        >
          <span class="icon-[mdi--satellite-uplink] text-lg"></span>
        </div>

        <div class="flex items-center opacity-90" title="Core Status">
          <span
            class="icon-[mdi--connection] text-lg transition-colors duration-500"
            :class="{
              'text-emerald-400 drop-shadow-[0_0_5px_rgba(52,211,153,0.8)]': isConnected,
              'text-rose-500 drop-shadow-[0_0_5px_rgba(244,63,94,0.8)]': !isConnected,
            }"
          ></span>
        </div>
      </div>
    </header>
  </div>
</template>

/* Scoped styles replaced by Tailwind CSS */
