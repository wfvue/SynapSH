<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import Terminal from "./Terminal.vue";
import DesktopWallpaper from "./desktop/DesktopWallpaper.vue";
import DesktopIcons from "./desktop/DesktopIcons.vue";
import type { DesktopIconItem } from "./desktop/DesktopIcon.vue";
import AppWindow from "./desktop/AppWindow.vue";
import DesktopDock from "./desktop/DesktopDock.vue";
import type { DockItem } from "./desktop/DesktopDock.vue";
import DesktopStatusBar from "./desktop/DesktopStatusBar.vue";
import FilesApp from "./apps/FilesApp.vue";
import ActivityMonitor from "./apps/ActivityMonitor.vue";

type AppId = "terminal" | "files" | "monitor" | "settings" | "app-center" | "browser";

const props = defineProps<{
  initialSession?: string;
}>();

const isConnected = ref(true);
const browserError = ref("");
const sessionId = computed(() => {
  console.log("DesktopShell sessionId:", props.initialSession);
  return props.initialSession || "default-session";
});

// 桌面图标配置 - macOS 风格图标
const desktopItems: DesktopIconItem[] = [
  { id: "computer", label: "此电脑", icon: "icon-[mdi--laptop]", color: "linear-gradient(135deg, #5e6ad2 0%, #3b82f6 100%)", app: "files" },
  { id: "terminal", label: "终端", icon: "icon-[mdi--console]", color: "linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%)", app: "terminal" },
  { id: "files", label: "访达", icon: "icon-[mdi--folder]", color: "linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%)", app: "files" },
  { id: "database", label: "数据库", icon: "icon-[mdi--database]", color: "linear-gradient(135deg, #f59e0b 0%, #fbbf24 100%)" },
  { id: "web", label: "Safari", icon: "icon-[mdi--compass]", color: "linear-gradient(135deg, #06b6d4 0%, #22d3ee 100%)", app: "browser" },
  { id: "settings", label: "系统设置", icon: "icon-[mdi--cog]", color: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)", app: "settings" },
  { id: "tasks", label: "活动监视器", icon: "icon-[mdi--chart-line]", color: "linear-gradient(135deg, #10b981 0%, #34d399 100%)", app: "monitor" },
  { id: "apps", label: "应用中心", icon: "icon-[mdi--apps]", color: "linear-gradient(135deg, #8b5cf6 0%, #a78bfa 100%)", app: "app-center" },
  { id: "trash", label: "废纸篓", icon: "icon-[mdi--delete]", color: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)" },
];


// Dock 栏配置
const dockItems: DockItem[] = [
  { id: "files", label: "访达", icon: "icon-[mdi--folder]", app: "files" },
  { id: "terminal", label: "终端", icon: "icon-[mdi--console]", app: "terminal" },
  { id: "browser", label: "浏览器", icon: "icon-[mdi--compass]", app: "browser" },
  { id: "monitor", label: "活动监视器", icon: "icon-[mdi--chart-line]", app: "monitor" },
  { id: "settings", label: "系统设置", icon: "icon-[mdi--cog]", app: "settings" },
  { id: "trash", label: "废纸篓", icon: "icon-[mdi--delete]" },
];

// 应用标题
const appTitles: Record<AppId, string> = {
  terminal: "终端",
  files: "文件管理器",
  monitor: "任务管理器",
  settings: "设置",
  "app-center": "应用中心",
  browser: "浏览器",
};

// 应用状态管理
const openApps = ref<AppId[]>([]);
const focusedApp = ref<AppId | null>(null);
const desktopIconsRef = ref<InstanceType<typeof DesktopIcons> | null>(null);
let unlistenProxyError: UnlistenFn | null = null;

type BrowserProxyError = {
  sessionId: string;
  host: string;
  port: number;
  message: string;
};

function formatInvokeError(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

async function openApp(id: string) {
  const appId = id as AppId;
  if (appId === "browser") {
    try {
      browserError.value = "";
      await invoke("browser_open", {
        sessionId: sessionId.value,
        url: "https://www.baidu.com",
        options: { profileMode: "new" },
      });
    } catch (error) {
      const message = formatInvokeError(error);
      browserError.value = message || "打开 Chrome 失败";
      console.error("打开 Chrome 失败:", error);
    }
    return;
  }
  if (!openApps.value.includes(appId)) {
    openApps.value.push(appId);
  }
  focusApp(appId);
}

function focusApp(id: AppId) {
  if (!openApps.value.includes(id)) return;
  openApps.value = openApps.value.filter((item) => item !== id).concat(id);
  focusedApp.value = id;
}

function closeApp(id: AppId) {
  openApps.value = openApps.value.filter((item) => item !== id);
  if (focusedApp.value === id) {
    focusedApp.value = openApps.value.length ? openApps.value[openApps.value.length - 1] : null;
  }
}

function getWindowOffset(id: AppId) {
  return openApps.value.indexOf(id) * 18;
}

function getWindowZIndex(id: AppId) {
  const idx = openApps.value.indexOf(id);
  return focusedApp.value === id ? 40 + idx : 20 + idx;
}

function clearDesktopSelection() {
  desktopIconsRef.value?.clearSelection();
}

const connectionStatus = computed(() => (isConnected.value ? "已连接" : "未连接"));

onMounted(async () => {
  unlistenProxyError = await listen<BrowserProxyError>("browser-proxy-error", (event) => {
    if (event.payload.sessionId !== sessionId.value) return;
    browserError.value = `${event.payload.message} (${event.payload.host}:${event.payload.port})`;
  });
});

onUnmounted(() => {
  if (unlistenProxyError) {
    unlistenProxyError();
    unlistenProxyError = null;
  }
});
</script>

<template>
  <div class="desktop" @click.self="clearDesktopSelection">
    <div v-if="browserError" class="browser-error">
      <span class="icon-[mdi--alert-circle]"></span>
      <span class="browser-error-text">{{ browserError }}</span>
      <button class="browser-error-close" @click="browserError = ''">✕</button>
    </div>
    <!-- 壁纸背景 -->
    <DesktopWallpaper />

    <!-- 桌面图标 -->
    <DesktopIcons ref="desktopIconsRef" :items="desktopItems" @open-app="openApp" />

    <!-- 窗口层 -->
    <section class="window-layer">
      <AppWindow v-for="app in openApps" :key="app" :app-id="app" :title="appTitles[app]" :active="focusedApp === app"
        :offset="getWindowOffset(app)" :z-index="getWindowZIndex(app)"
        :status-text="app === 'terminal' ? connectionStatus : undefined"
        :status-online="app === 'terminal' ? isConnected : undefined" @close="closeApp(app)" @focus="focusApp(app)">
        <!-- 终端应用 -->
        <div v-if="app === 'terminal'" class="terminal-shell">
          <Terminal :session-id="sessionId" />
        </div>

        <!-- 文件管理器 -->
        <FilesApp v-else-if="app === 'files'" />

        <!-- 活动监视器 -->
        <ActivityMonitor v-else-if="app === 'monitor'" :session-id="sessionId" />

        <!-- 其他应用占位 -->
        <div v-else class="app-empty">
          <h2>正在准备</h2>
          <p>这个模块会作为系统级 App 扩展加入。</p>
        </div>
      </AppWindow>
    </section>

    <!-- Dock 栏 -->
    <DesktopDock :items="dockItems" :open-apps="openApps" @open-app="openApp" />

    <!-- 状态栏 -->
    <DesktopStatusBar :is-connected="isConnected" />
  </div>
</template>

<style scoped>
.desktop {
  position: relative;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  color: var(--text-primary);
}

.browser-error {
  position: absolute;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 12px;
  background: rgba(20, 14, 14, 0.9);
  border: 1px solid rgba(255, 122, 122, 0.45);
  color: #ffd5d5;
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35);
  z-index: 6;
  max-width: min(920px, 90vw);
}

.browser-error-text {
  font-size: 13px;
  line-height: 1.4;
}

.browser-error-close {
  border: none;
  background: transparent;
  color: #ffd5d5;
  cursor: pointer;
  font-size: 14px;
}

.window-layer {
  position: absolute;
  inset: 0;
  z-index: 3;
  pointer-events: none;
}

.terminal-shell {
  height: 100%;
  padding: 0;
  background: #1e1e1e;
  border-radius: 0 0 16px 16px;
  overflow: hidden;
}

.app-empty {
  height: 100%;
  display: grid;
  place-content: center;
  gap: 12px;
  color: var(--text-muted);
  text-align: center;
}

.app-empty h2 {
  font-size: 1.5rem;
  color: var(--text-primary);
}
</style>
