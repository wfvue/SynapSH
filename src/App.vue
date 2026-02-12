<!-- App.vue - 应用根组件，负责全局标签页与会话切换 -->
<script setup lang="ts">
import { computed, onErrorCaptured, ref, watch } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import DesktopShell from "./views/DesktopShell.vue";
import MachineManager from "./views/MachineManager.vue";
import TabBar, { type Tab } from "./components/TabBar.vue";
import { Toaster } from "@/components/ui/toast";

const globalError = ref<string>("");

onErrorCaptured((err) => {
  console.error("Global Error Captured:", err);
  globalError.value = String(err);
  return false;
});

const reloadApp = () => window.location.reload();

useColorMode({
  emitAuto: true,
  storageKey: "vueuse-color-mode",
  attribute: "class",
  modes: { dark: "dark", light: "light", auto: "auto" },
});

const accentColor = useLocalStorage("appearance-accent-color", "#0a84ff");
watch(
  accentColor,
  (color) => {
    document.documentElement.style.setProperty("--accent-color", color);
  },
  { immediate: true }
);

interface ConnectPayload {
  sessionId: string;
  machineId: string;
  machineName: string;
  host: string;
}

interface AppTab extends Tab {
  kind: "manager" | "session";
  closable?: boolean;
  sessionId?: string;
  machineId?: string;
}

const MAIN_TAB_ID = "tab-main";

function createManagerTab(): AppTab {
  return {
    id: MAIN_TAB_ID,
    title: "机器管理",
    view: "machine-manager",
    kind: "manager",
    closable: false,
    icon: "icon-[mdi--server-network]",
  };
}

const tabs = ref<AppTab[]>([createManagerTab()]);
const activeTabId = ref(MAIN_TAB_ID);

const activeTab = computed(() => tabs.value.find((tab) => tab.id === activeTabId.value));

function getActiveTabIndex(): number {
  return tabs.value.findIndex((tab) => tab.id === activeTabId.value);
}

function getNewTabId() {
  return `tab-${Math.random().toString(36).slice(2, 11)}`;
}

function handleNewTab(_id?: string) {
  activeTabId.value = MAIN_TAB_ID;
}

function handleSwitchTab(id: string) {
  activeTabId.value = id;
}

function handleCloseTab(id: string) {
  const index = tabs.value.findIndex((tab) => tab.id === id);
  if (index === -1) return;
  if (tabs.value[index].kind === "manager" || tabs.value[index].closable === false) return;

  const wasActive = activeTabId.value === id;
  tabs.value.splice(index, 1);

  if (wasActive) {
    const nextTab = tabs.value[index] || tabs.value[index - 1];
    activeTabId.value = nextTab?.id || MAIN_TAB_ID;
  }
}

function handleConnect(payload: ConnectPayload) {
  const existingTab = tabs.value.find(
    (tab) => tab.kind === "session" && tab.machineId === payload.machineId
  );

  if (existingTab) {
    existingTab.view = "desktop";
    existingTab.sessionId = payload.sessionId;
    existingTab.title = payload.machineName || payload.host;
    existingTab.icon = "icon-[mdi--console-network-outline]";
    activeTabId.value = existingTab.id;
    return;
  }

  const newId = getNewTabId();
  tabs.value.push({
    id: newId,
    title: payload.machineName || payload.host,
    view: "desktop",
    kind: "session",
    closable: true,
    sessionId: payload.sessionId,
    machineId: payload.machineId,
    icon: "icon-[mdi--console-network-outline]",
  });
  activeTabId.value = newId;
}

function handleDisconnect() {
  const index = getActiveTabIndex();
  if (index === -1) return;

  if (tabs.value[index].kind === "session") {
    tabs.value.splice(index, 1);
  }
  activeTabId.value = MAIN_TAB_ID;
}

function resetToSafeState() {
  globalError.value = "";
  tabs.value = [createManagerTab()];
  activeTabId.value = MAIN_TAB_ID;
}
</script>

<template>
  <div class="app-container">
    <div v-if="globalError" class="global-error-overlay">
      <h2 class="text-xl font-bold mb-4">应用崩溃</h2>
      <pre class="bg-black/50 p-4 rounded mb-4 overflow-auto max-h-[60vh] text-left whitespace-pre-wrap">{{ globalError }}</pre>
      <div class="flex gap-4">
        <button class="px-4 py-2 bg-blue-600 rounded hover:bg-blue-700 text-white" @click="resetToSafeState">返回首页</button>
        <button class="px-4 py-2 bg-gray-600 rounded hover:bg-gray-700 text-white" @click="reloadApp">重新加载</button>
      </div>
    </div>

    <TabBar
      :tabs="tabs"
      :active-tab-id="activeTabId"
      :is-fullscreen="false"
      @new-tab="handleNewTab"
      @switch-tab="handleSwitchTab"
      @close-tab="handleCloseTab"
    />

    <div class="main-content">
      <Suspense>
        <template #default>
          <MachineManager
            v-if="activeTab?.view === 'machine-manager'"
            :key="`${activeTab?.id}-mm`"
            @connect="handleConnect"
          />
          <DesktopShell
            v-else
            :key="`${activeTab?.id}-ds`"
            :initial-session="activeTab?.sessionId"
            @disconnected="handleDisconnect"
          />
        </template>
        <template #fallback>
          <div class="flex items-center justify-center h-full w-full text-white">Loading...</div>
        </template>
      </Suspense>
    </div>
    <Toaster />
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
  background-color: #0b0d10;
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
