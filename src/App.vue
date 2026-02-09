<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import DesktopShell from "./views/DesktopShell.vue";
import MachineManager from "./views/MachineManager.vue";
import TabBar, { type Tab } from "./components/TabBar.vue";

const globalError = ref<string>("");

// Global error handler
import { onErrorCaptured } from "vue";
onErrorCaptured((err) => {
  console.error("Global Error Captured:", err);
  globalError.value = String(err);
  return false;
});

const reloadApp = () => window.location.reload();

// 主题设置
const mode = useColorMode({
  emitAuto: true,
  storageKey: "vueuse-color-mode",
  attribute: "class",
  modes: { dark: "dark", light: "light", auto: "auto" },
});

const accentColor = useLocalStorage("appearance-accent-color", "#3b82f6");
watch(accentColor, (color) => {
  document.documentElement.style.setProperty("--accent-color", color);
}, { immediate: true });

// --- Tab Management ---
interface AppTab extends Tab {
  sessionId?: string;
}

const tabs = ref<AppTab[]>([
  { id: "tab-1", title: "New Connection", view: "machine-manager" },
]);
const activeTabId = ref("tab-1");

// Computed for template display only (read-only)
const activeTab = computed(() => tabs.value.find((t) => t.id === activeTabId.value));

function getActiveTabIndex(): number {
  return tabs.value.findIndex((t) => t.id === activeTabId.value);
}

function getNewTabId() {
  return "tab-" + Math.random().toString(36).substr(2, 9);
}

function handleNewTab() {
  const newId = getNewTabId();
  tabs.value.push({
    id: newId,
    title: "New Connection",
    view: "machine-manager",
  });
  activeTabId.value = newId;
}

function handleSwitchTab(id: string) {
  activeTabId.value = id;
}

function handleCloseTab(id: string) {
  const index = tabs.value.findIndex((t) => t.id === id);
  if (index === -1) return;

  if (id === activeTabId.value) {
    const nextTab = tabs.value[index + 1] || tabs.value[index - 1];
    if (nextTab) {
      activeTabId.value = nextTab.id;
    } else {
      const newId = getNewTabId();
      tabs.value.push({
        id: newId,
        title: "New Connection",
        view: "machine-manager",
      });
      activeTabId.value = newId;
    }
  }

  tabs.value.splice(index, 1);
}

// Connection events - directly mutate array element
function handleConnect(sessionId: string) {
  const idx = getActiveTabIndex();
  if (idx === -1) return;

  // Direct mutation of array element
  tabs.value[idx].view = "desktop";
  tabs.value[idx].sessionId = sessionId;
  tabs.value[idx].title = "Session " + sessionId.substring(0, 4);
  tabs.value[idx].icon = "icon-[mdi--console]";
}

function handleDisconnect() {
  const idx = getActiveTabIndex();
  if (idx === -1) return;

  tabs.value[idx].view = "machine-manager";
  tabs.value[idx].sessionId = undefined;
  tabs.value[idx].title = "New Connection";
  tabs.value[idx].icon = undefined;
}

function resetToSafeState() {
  globalError.value = "";
  const idx = getActiveTabIndex();
  if (idx !== -1) {
    tabs.value[idx].view = "machine-manager";
    tabs.value[idx].sessionId = undefined;
    tabs.value[idx].title = "New Connection";
  }
}
</script>

<template>
  <div class="app-container">
    <!-- Error Overlay -->
    <div v-if="globalError" class="global-error-overlay">
      <h2 class="text-xl font-bold mb-4">应用崩溃</h2>
      <pre
        class="bg-black/50 p-4 rounded mb-4 overflow-auto max-h-[60vh] text-left whitespace-pre-wrap">{{ globalError }}</pre>
      <div class="flex gap-4">
        <button class="px-4 py-2 bg-blue-600 rounded hover:bg-blue-700 text-white"
          @click="resetToSafeState">返回首页</button>
        <button class="px-4 py-2 bg-gray-600 rounded hover:bg-gray-700 text-white" @click="reloadApp">重新加载</button>
      </div>
    </div>

    <!-- Tab Bar -->
    <TabBar :tabs="tabs" :active-tab-id="activeTabId" @new-tab="handleNewTab" @switch-tab="handleSwitchTab"
      @close-tab="handleCloseTab" />

    <!-- Main Content -->
    <div class="main-content">
      <Suspense>
        <template #default>
          <!-- Simple v-if/v-else without KeepAlive for now -->
          <MachineManager v-if="activeTab?.view === 'machine-manager'" :key="activeTab?.id + '-mm'"
            @connect="handleConnect" />
          <DesktopShell v-else :key="activeTab?.id + '-ds'" :initial-session="activeTab?.sessionId"
            @disconnected="handleDisconnect" />
        </template>
        <template #fallback>
          <div class="flex items-center justify-center h-full w-full text-white">Loading...</div>
        </template>
      </Suspense>
    </div>
  </div>
</template>

<style>
body {
  font-family: "Avenir Next", "Avenir", "PingFang SC", "HarmonyOS Sans SC", "Noto Sans CJK SC", "Source Han Sans SC", sans-serif;
  background-color: var(--background);
  color: var(--foreground);
  overflow: hidden;
  margin: 0;
}

#app {
  width: 100vw;
  height: 100vh;
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.main-content {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.global-error-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(20, 0, 0, 0.95);
  color: #ff6b6b;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 2rem;
  backdrop-filter: blur(10px);
}
</style>
