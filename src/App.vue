<script setup lang="ts">
import { ref, watch } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import DesktopShell from "./components/DesktopShell.vue";
import MachineManager from "./components/MachineManager.vue";

type ViewState = "machine-manager" | "desktop";

const currentView = ref<ViewState>("machine-manager");
const currentSession = ref<string>("");

// 在根组件初始化主题模式
const mode = useColorMode({
  emitAuto: true,
  storageKey: "vueuse-color-mode",
  attribute: "class",
  modes: {
    dark: "dark",
    light: "light",
    auto: "auto",
  },
});

// 读取强调色设置并应用为 CSS 变量
const accentColor = useLocalStorage("appearance-accent-color", "#3b82f6");
watch(
  accentColor,
  (color) => {
    document.documentElement.style.setProperty("--accent-color", color);
  },
  { immediate: true }
);

function handleConnect(sessionId: string) {
  currentSession.value = sessionId;
  currentView.value = "desktop";
}

function handleDisconnect() {
  currentSession.value = "";
  currentView.value = "machine-manager";
}
</script>

<template>
  <div class="app-container">
    <Transition name="fade" mode="out-in">
      <component :is="currentView === 'machine-manager' ? MachineManager : DesktopShell"
        :initial-session="currentSession" @connect="handleConnect" @disconnected="handleDisconnect" />
    </Transition>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}


body {
  font-family: "Avenir Next", "Avenir", "PingFang SC", "HarmonyOS Sans SC",
    "Noto Sans CJK SC", "Source Han Sans SC", sans-serif;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  overflow: hidden;
}

#app {
  width: 100vw;
  height: 100vh;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
