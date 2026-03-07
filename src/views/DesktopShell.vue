<!-- 桌面环境主视图：负责图标、窗口管理与应用调度。 -->
<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import DesktopWallpaper from "../components/desktop/DesktopWallpaper.vue";
import DesktopIcons from "../components/desktop/DesktopIcons.vue";
import type { DesktopIconItem } from "../components/desktop/DesktopIcon.vue";
import AppWindow from "../components/desktop/AppWindow.vue";
import DesktopDock from "../components/desktop/DesktopDock.vue";
import type { DockItem } from "../components/desktop/DesktopDock.vue";
import FilesApp from "./apps/FilesApp.vue";
import ActivityMonitor from "./apps/ActivityMonitor.vue";
import TextEditorApp from "./apps/TextEditorApp.vue";
import SettingsApp from "./apps/SettingsApp.vue";
import TerminalApp from "./apps/TerminalApp.vue";
import DatabaseManagerApp from "./apps/DatabaseManagerApp.vue";

type AppId = "terminal" | "files" | "monitor" | "settings" | "app-center" | "browser" | "editor" | "database";

const props = defineProps<{
  initialSession?: string;
}>();

const isConnected = ref(true);
const browserError = ref("");
const sessionId = computed(() => {
  return props.initialSession || "default-session";
});

// 桌面图标配置 - macOS 风格图标
const desktopItems: DesktopIconItem[] = [
  { id: "computer", label: "此电脑", icon: "icon-[mdi--laptop]", color: "linear-gradient(135deg, #5e6ad2 0%, #3b82f6 100%)", app: "files" },
  { id: "terminal", label: "终端", icon: "icon-[mdi--console]", color: "linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%)", app: "terminal" },
  { id: "files", label: "访达", icon: "icon-[mdi--folder]", color: "linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%)", app: "files" },
  { id: "database", label: "数据库", icon: "icon-[mdi--database]", color: "linear-gradient(135deg, #f59e0b 0%, #fbbf24 100%)", app: "database" },
  { id: "web", label: "浏览器", icon: "icon-[mdi--compass]", color: "linear-gradient(135deg, #06b6d4 0%, #22d3ee 100%)", app: "browser" },
  { id: "settings", label: "系统设置", icon: "icon-[mdi--cog]", color: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)", app: "settings" },
  { id: "tasks", label: "活动监视器", icon: "icon-[mdi--chart-line]", color: "linear-gradient(135deg, #10b981 0%, #34d399 100%)", app: "monitor" },
  { id: "apps", label: "应用中心", icon: "icon-[mdi--apps]", color: "linear-gradient(135deg, #8b5cf6 0%, #a78bfa 100%)", app: "app-center" },
  { id: "trash", label: "废纸篓", icon: "icon-[mdi--delete]", color: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)" },
];


// Dock 栏配置
const dockItems: DockItem[] = [
  { id: "files", label: "访达", icon: "icon-[mdi--folder]", color: "linear-gradient(135deg, #3b82f6 0%, #60a5fa 100%)", app: "files" },
  { id: "terminal", label: "终端", icon: "icon-[mdi--console]", color: "linear-gradient(135deg, #1e1e1e 0%, #2d2d2d 100%)", app: "terminal" },
  { id: "browser", label: "浏览器", icon: "icon-[mdi--compass]", color: "linear-gradient(135deg, #06b6d4 0%, #22d3ee 100%)", app: "browser" },
  { id: "monitor", label: "活动监视器", icon: "icon-[mdi--chart-line]", color: "linear-gradient(135deg, #10b981 0%, #34d399 100%)", app: "monitor" },
  { id: "settings", label: "系统设置", icon: "icon-[mdi--cog]", color: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)", app: "settings" },
  { id: "trash", label: "废纸篓", icon: "icon-[mdi--delete]", color: "linear-gradient(135deg, #6b7280 0%, #9ca3af 100%)" },
];

// 应用标题
const appTitles: Record<AppId, string> = {
  terminal: "终端",
  files: "文件管理器",
  monitor: "任务管理器",
  settings: "设置",
  "app-center": "应用中心",
  browser: "浏览器",
  editor: "文本编辑器",
  database: "数据库管理",
};

// 编辑器状态
interface EditorFile {
  path: string;
  name: string;
}
const editorFile = ref<EditorFile | null>(null);

// 应用状态管理
const openApps = ref<AppId[]>([]);
const minimizedApps = ref<AppId[]>([]);
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
        url: "https://www.google.com",
        options: { profileMode: "session" },
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
  } else {
    if (minimizedApps.value.includes(appId)) {
      minimizedApps.value = minimizedApps.value.filter((a) => a !== appId);
    } else if (focusedApp.value === appId) {
      minimizeApp(appId);
      return;
    }
  }
  focusApp(appId);
}

// 打开文件编辑器
function openFileInEditor(filePath: string, fileName: string) {
  editorFile.value = { path: filePath, name: fileName };
  if (!openApps.value.includes("editor")) {
    openApps.value.push("editor");
  }
  focusApp("editor");
}

function focusApp(id: AppId) {
  if (!openApps.value.includes(id)) return;
  if (minimizedApps.value.includes(id)) {
    minimizedApps.value = minimizedApps.value.filter((a) => a !== id);
  }
  openApps.value = openApps.value.filter((item) => item !== id).concat(id);
  focusedApp.value = id;
}

function closeApp(id: AppId) {
  openApps.value = openApps.value.filter((item) => item !== id);
  minimizedApps.value = minimizedApps.value.filter((item) => item !== id);
  if (focusedApp.value === id) {
    const visibleApps = openApps.value.filter((a) => !minimizedApps.value.includes(a));
    focusedApp.value = visibleApps.length ? visibleApps[visibleApps.length - 1] : null;
  }
}

function minimizeApp(id: AppId) {
  if (!minimizedApps.value.includes(id)) {
    minimizedApps.value.push(id);
  }
  if (focusedApp.value === id) {
    const visibleApps = openApps.value.filter((a) => !minimizedApps.value.includes(a));
    focusedApp.value = visibleApps.length ? visibleApps[visibleApps.length - 1] : null;
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
    <AppWindow v-for="app in openApps" :key="app" :app-id="app" :title="appTitles[app]" :active="focusedApp === app"
      :offset="getWindowOffset(app)" :z-index="getWindowZIndex(app)" :minimized="minimizedApps.includes(app)"
      @close="closeApp(app)" @minimize="minimizeApp(app)" @focus="focusApp(app)" v-slot="windowProps">
      <!-- 终端应用 -->
      <TerminalApp v-if="app === 'terminal'" :session-id="sessionId" />

      <!-- 文件管理器 -->
      <FilesApp v-else-if="app === 'files'" :session-id="sessionId" @open-file="openFileInEditor" />

      <!-- 活动监视器 -->
      <ActivityMonitor v-else-if="app === 'monitor'" :session-id="sessionId" />

      <!-- 文本编辑器 -->
      <TextEditorApp v-else-if="app === 'editor' && editorFile" :session-id="sessionId" :file-path="editorFile.path"
        :file-name="editorFile.name" />

      <!-- 系统设置 -->
      <SettingsApp v-else-if="app === 'settings'" :session-id="sessionId" :close="() => closeApp(app)"
        :minimize="() => minimizeApp(app)" :maximize="() => focusApp(app)" :start-drag="() => { }" />

      <!-- 数据库管理 -->
      <DatabaseManagerApp v-else-if="app === 'database'" :session-id="sessionId" />

      <!-- 其他应用占位 -->
      <div v-else class="app-empty">
        <h2>正在准备</h2>
        <p>这个模块会作为系统级 App 扩展加入。</p>
      </div>
    </AppWindow>

    <!-- Dock 栏 -->
    <DesktopDock :items="dockItems" :open-apps="openApps" @open-app="openApp" />
  </div>
</template>

<style scoped>
.desktop {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  color: var(--foreground);
}

/* ... existing styles ... */

.app-empty {
  height: 100%;
  display: grid;
  place-content: center;
  gap: 12px;
  color: var(--muted-foreground);
  text-align: center;
}

.app-empty h2 {
  font-size: 1.5rem;
  color: var(--foreground);
}
</style>
