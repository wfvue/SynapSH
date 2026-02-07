<script setup lang="ts">
import { ref } from "vue";
import DesktopShell from "./components/DesktopShell.vue";
import MachineManager from "./components/MachineManager.vue";

type ViewState = "machine-manager" | "desktop";

const currentView = ref<ViewState>("machine-manager");
const currentSession = ref<string>("");

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

:root {
  --wallpaper-a: #111827;
  --wallpaper-b: #1f2937;
  --wallpaper-c: #2b1f3b;
  --bg-primary: #0f141f;
  --bg-secondary: rgba(18, 24, 36, 0.9);
  --bg-tertiary: rgba(28, 34, 48, 0.85);
  --text-primary: #e6e9f2;
  --text-secondary: #9aa3b2;
  --text-muted: #a8b0c2;
  --accent: #7dd3fc;
  --border: rgba(255, 255, 255, 0.12);
  --success: #5de4c7;
  --error: #ff7a7a;
  --icon-bg: rgba(255, 255, 255, 0.08);
  --shadow-strong: 0 20px 60px rgba(0, 0, 0, 0.4);
  --shadow-soft: 0 10px 30px rgba(0, 0, 0, 0.25);
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
