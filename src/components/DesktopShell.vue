<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import Terminal from "./Terminal.vue";
import ConnectionPanel from "./ConnectionPanel.vue";

type AppId = "terminal" | "files" | "monitor" | "settings" | "app-center";

const props = defineProps<{
  initialSession?: string;
}>();

const isConnected = ref(!!props.initialSession);
const sessionId = ref(props.initialSession || "");

function onConnected(id: string) {
  sessionId.value = id;
  isConnected.value = true;
}

function onDisconnected() {
  isConnected.value = false;
  sessionId.value = "";
  emit("disconnected");
}

const emit = defineEmits<{
  disconnected: [];
}>();

const desktopItems = [
  { id: "computer", label: "此电脑", icon: "icon-computer", app: "files" as AppId },
  { id: "utilities", label: "实用工具", icon: "icon-toolbox", app: "app-center" as AppId },
  { id: "terminal", label: "终端", icon: "icon-terminal", app: "terminal" as AppId },
  { id: "database", label: "数据库", icon: "icon-database" },
  { id: "trash", label: "回收站", icon: "icon-trash" },
  { id: "web", label: "网站搭建", icon: "icon-web" },
  { id: "files", label: "文件", icon: "icon-files", app: "files" as AppId },
  { id: "settings", label: "设置", icon: "icon-settings", app: "settings" as AppId },
  { id: "tasks", label: "任务管理器", icon: "icon-monitor", app: "monitor" as AppId },
  { id: "apps", label: "应用中心", icon: "icon-apps", app: "app-center" as AppId },
];

const dockItems = [
  { id: "files", label: "文件", icon: "icon-files", app: "files" as AppId },
  { id: "terminal", label: "终端", icon: "icon-terminal", app: "terminal" as AppId },
  { id: "monitor", label: "任务", icon: "icon-monitor", app: "monitor" as AppId },
  { id: "trash", label: "回收站", icon: "icon-trash" },
  { id: "settings", label: "设置", icon: "icon-settings", app: "settings" as AppId },
];

const openApps = ref<AppId[]>([]);
const focusedApp = ref<AppId | null>(null);
const selectedIcon = ref<string | null>(null);

const appTitles: Record<AppId, string> = {
  terminal: "终端",
  files: "文件管理器",
  monitor: "任务管理器",
  settings: "设置",
  "app-center": "应用中心",
};

function openApp(id?: AppId) {
  if (!id) return;
  if (!openApps.value.includes(id)) {
    openApps.value.push(id);
  }
  focusApp(id);
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

function windowStyle(id: AppId) {
  const idx = openApps.value.indexOf(id);
  const offset = idx * 18;
  return {
    top: `calc(8vh + ${offset}px)`,
    left: `calc(50% + ${offset}px)`,
    zIndex: focusedApp.value === id ? 40 + idx : 20 + idx,
  };
}

function selectDesktopIcon(id: string) {
  selectedIcon.value = id;
}

function clearDesktopSelection() {
  selectedIcon.value = null;
}

const timeText = ref("");
const dateText = ref("");
let timer: number | undefined;

function updateClock() {
  const now = new Date();
  timeText.value = now.toLocaleTimeString("zh-CN", {
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  });
  dateText.value = now.toLocaleDateString("zh-CN", {
    month: "2-digit",
    day: "2-digit",
  });
}

onMounted(() => {
  updateClock();
  timer = window.setInterval(updateClock, 30000);
});

onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});

const connectionStatus = computed(() => (isConnected.value ? "已连接" : "未连接"));

const fileRows = [
  { name: "home", type: "文件夹", size: "--", modified: "2026-02-07 18:10" },
  { name: "etc", type: "文件夹", size: "--", modified: "2026-02-07 17:42" },
  { name: "var", type: "文件夹", size: "--", modified: "2026-02-07 16:08" },
  { name: "deploy.sh", type: "脚本", size: "12 KB", modified: "2026-02-06 23:12" },
  { name: "report.log", type: "日志", size: "4.2 MB", modified: "2026-02-06 21:05" },
];
</script>

<template>
  <div class="desktop" @click.self="clearDesktopSelection">
    <div class="wallpaper">
      <span class="glow glow-a"></span>
      <span class="glow glow-b"></span>
      <span class="glow glow-c"></span>
      <span class="wave wave-a"></span>
      <span class="wave wave-b"></span>
    </div>

    <section class="desktop-icons">
      <button v-for="item in desktopItems" :key="item.id" class="desktop-icon"
        :class="{ selected: selectedIcon === item.id }" @click.stop="selectDesktopIcon(item.id)"
        @dblclick.stop="openApp(item.app)">
        <span class="icon-visual" :class="item.icon"></span>
        <span class="icon-label">{{ item.label }}</span>
      </button>
    </section>

    <section class="window-layer">
      <div v-for="app in openApps" :key="app" class="app-window"
        :class="[`app-window--${app}`, { active: focusedApp === app }]" :style="windowStyle(app)"
        @mousedown="focusApp(app)">
        <header class="app-titlebar">
          <div class="window-controls">
            <button class="control control--close" @click.stop="closeApp(app)"></button>
            <button class="control control--min"></button>
            <button class="control control--max"></button>
          </div>
          <div class="app-title">{{ appTitles[app] }}</div>
          <div class="title-actions">
            <span v-if="app === 'terminal'" class="status-pill" :class="{ online: isConnected }">
              {{ connectionStatus }}
            </span>
          </div>
        </header>

        <div class="app-body">
          <div v-if="app === 'terminal'" class="terminal-shell">
            <aside class="terminal-side">
              <div class="glass-panel">
                <ConnectionPanel :is-connected="isConnected" @connected="onConnected" @disconnected="onDisconnected" />
              </div>
            </aside>
            <section class="terminal-main">
              <Terminal v-if="isConnected" :session-id="sessionId" />
              <div v-else class="app-empty">
                <h2>欢迎进入光析终端</h2>
                <p>在左侧完成连接后，这里会显示远程 Shell。</p>
              </div>
            </section>
          </div>

          <div v-else-if="app === 'files'" class="files-shell">
            <div class="files-toolbar">
              <div class="path">/home/ops</div>
              <div class="files-actions">
                <button>上传</button>
                <button>下载</button>
                <button>新建文件夹</button>
                <button>刷新</button>
              </div>
            </div>
            <div class="files-content">
              <aside class="files-tree">
                <div class="tree-title">位置</div>
                <div class="tree-item active">主目录</div>
                <div class="tree-item">下载</div>
                <div class="tree-item">备份</div>
                <div class="tree-item">项目</div>
              </aside>
              <section class="files-list">
                <div class="list-header">
                  <span>名称</span>
                  <span>类型</span>
                  <span>大小</span>
                  <span>修改时间</span>
                </div>
                <div v-for="row in fileRows" :key="row.name" class="list-row">
                  <span class="name">{{ row.name }}</span>
                  <span>{{ row.type }}</span>
                  <span>{{ row.size }}</span>
                  <span>{{ row.modified }}</span>
                </div>
              </section>
            </div>
            <div class="files-status">共 {{ fileRows.length }} 项 · 已同步</div>
          </div>

          <div v-else class="app-empty">
            <h2>正在准备</h2>
            <p>这个模块会作为系统级 App 扩展加入。</p>
          </div>
        </div>
      </div>
    </section>

    <section class="dock">
      <button v-for="item in dockItems" :key="item.id" class="dock-item"
        :class="{ active: item.app && openApps.includes(item.app) }" @click.stop="openApp(item.app)">
        <span class="icon-visual" :class="item.icon"></span>
      </button>
    </section>

    <section class="status-bar">
      <div class="status-item">
        <span class="status-dot"></span>
        <span>SSH</span>
        <span>{{ isConnected ? "1" : "0" }}</span>
      </div>
      <div class="status-item">
        <span>{{ timeText }}</span>
        <span class="status-muted">{{ dateText }}</span>
      </div>
      <div class="status-item">
        <span>ZH</span>
      </div>
    </section>
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

.wallpaper {
  position: absolute;
  inset: 0;
  background: radial-gradient(circle at 20% 20%, rgba(125, 211, 252, 0.14), transparent 45%),
    radial-gradient(circle at 70% 30%, rgba(251, 191, 36, 0.12), transparent 50%),
    linear-gradient(140deg, var(--wallpaper-a), var(--wallpaper-b) 45%, var(--wallpaper-c) 90%);
  z-index: 0;
}

.glow {
  position: absolute;
  width: 380px;
  height: 380px;
  border-radius: 50%;
  filter: blur(60px);
  opacity: 0.7;
}

.glow-a {
  top: -60px;
  left: 10%;
  background: radial-gradient(circle, rgba(94, 234, 212, 0.6), transparent 70%);
}

.glow-b {
  right: 12%;
  top: 22%;
  background: radial-gradient(circle, rgba(125, 211, 252, 0.5), transparent 70%);
}

.glow-c {
  bottom: -120px;
  right: 18%;
  background: radial-gradient(circle, rgba(251, 191, 36, 0.35), transparent 70%);
}

.wave {
  position: absolute;
  width: 120%;
  height: 240px;
  left: -10%;
  border-radius: 999px;
  opacity: 0.22;
  filter: blur(20px);
}

.wave-a {
  bottom: 32%;
  background: linear-gradient(90deg, rgba(125, 211, 252, 0.6), rgba(94, 234, 212, 0.2));
}

.wave-b {
  bottom: 18%;
  background: linear-gradient(90deg, rgba(59, 130, 246, 0.2), rgba(251, 191, 36, 0.5));
}

.desktop-icons {
  position: relative;
  z-index: 2;
  display: grid;
  grid-auto-rows: 86px;
  gap: 18px;
  padding: 28px 24px;
  width: 130px;
}

.desktop-icon {
  background: transparent;
  border: none;
  display: grid;
  gap: 8px;
  justify-items: center;
  color: var(--text-primary);
  cursor: pointer;
  padding: 6px;
  border-radius: 12px;
  transition: transform 0.2s ease, background 0.2s ease;
}

.desktop-icon.selected {
  background: rgba(255, 255, 255, 0.08);
}

.desktop-icon:hover {
  transform: translateY(-2px);
}

.icon-visual {
  width: 54px;
  height: 54px;
  border-radius: 16px;
  background: var(--icon-bg);
  position: relative;
  box-shadow: var(--shadow-soft);
  display: grid;
  place-items: center;
}

.icon-label {
  font-size: 0.78rem;
  color: var(--text-muted);
  text-align: center;
}

.window-layer {
  position: absolute;
  inset: 0;
  z-index: 3;
  pointer-events: none;
}

.app-window {
  pointer-events: auto;
  position: absolute;
  transform: translateX(-50%);
  border-radius: 18px;
  background: rgba(14, 18, 28, 0.88);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: var(--shadow-strong);
  backdrop-filter: blur(20px);
  overflow: hidden;
  transition: box-shadow 0.2s ease, transform 0.2s ease;
}

.app-window.active {
  box-shadow: 0 28px 80px rgba(0, 0, 0, 0.5);
  transform: translateX(-50%) translateY(-2px);
}

.app-window--terminal {
  width: min(1120px, 92vw);
  height: min(720px, 80vh);
}

.app-window--files {
  width: min(980px, 90vw);
  height: min(680px, 76vh);
}

.app-window--monitor,
.app-window--settings,
.app-window--app-center {
  width: min(860px, 88vw);
  height: min(560px, 70vh);
}

.app-titlebar {
  display: grid;
  grid-template-columns: 120px 1fr 160px;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  background: rgba(18, 22, 32, 0.8);
}

.window-controls {
  display: flex;
  gap: 8px;
}

.control {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: none;
  background: rgba(255, 255, 255, 0.25);
}

.control--close {
  background: #ff6b6b;
}

.control--min {
  background: #ffd166;
}

.control--max {
  background: #9ae66e;
}

.app-title {
  text-align: center;
  font-size: 0.9rem;
  letter-spacing: 0.08em;
  color: var(--text-muted);
  text-transform: uppercase;
}

.title-actions {
  display: flex;
  justify-content: flex-end;
}

.status-pill {
  font-size: 0.72rem;
  padding: 4px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  color: var(--text-muted);
}

.status-pill.online {
  background: rgba(94, 234, 212, 0.18);
  color: #bff4ea;
}

.app-body {
  height: calc(100% - 48px);
}

.terminal-shell {
  display: grid;
  grid-template-columns: 320px 1fr;
  height: 100%;
}

.terminal-side {
  padding: 16px;
  border-right: 1px solid rgba(255, 255, 255, 0.08);
}

.glass-panel {
  height: 100%;
  background: rgba(18, 22, 32, 0.8);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 16px;
  overflow: hidden;
}

.terminal-main {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 12px;
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

.files-shell {
  height: 100%;
  display: grid;
  grid-template-rows: auto 1fr auto;
}

.files-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.files-toolbar .path {
  font-size: 0.85rem;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.06);
  padding: 6px 12px;
  border-radius: 10px;
}

.files-actions {
  display: flex;
  gap: 8px;
}

.files-actions button {
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-primary);
  padding: 6px 10px;
  border-radius: 10px;
  font-size: 0.8rem;
  cursor: pointer;
  transition: border 0.2s ease, transform 0.2s ease;
}

.files-actions button:hover {
  border-color: rgba(125, 211, 252, 0.6);
  transform: translateY(-1px);
}

.files-content {
  display: grid;
  grid-template-columns: 220px 1fr;
  height: 100%;
}

.files-tree {
  padding: 16px;
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  display: grid;
  gap: 10px;
  color: var(--text-muted);
}

.tree-title {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-secondary);
}

.tree-item {
  padding: 8px 10px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.03);
  cursor: pointer;
}

.tree-item.active {
  background: rgba(125, 211, 252, 0.18);
  color: #c8ecff;
}

.files-list {
  padding: 16px;
  display: grid;
  gap: 10px;
  color: var(--text-primary);
}

.list-header,
.list-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1.4fr;
  gap: 12px;
  align-items: center;
}

.list-header {
  font-size: 0.75rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.list-row {
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
}

.list-row .name {
  color: #c8ecff;
}

.files-status {
  padding: 10px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 0.8rem;
  color: var(--text-muted);
}

.dock {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 18px;
  background: rgba(12, 16, 24, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(16px);
  z-index: 4;
}

.dock-item {
  width: 46px;
  height: 46px;
  border-radius: 14px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: transform 0.2s ease;
}

.dock-item.active {
  transform: translateY(-4px);
}

.status-bar {
  position: absolute;
  bottom: 16px;
  right: 18px;
  display: flex;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 16px;
  background: rgba(12, 16, 24, 0.72);
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(16px);
  z-index: 4;
  color: var(--text-muted);
  font-size: 0.8rem;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: rgba(94, 234, 212, 0.8);
}

.status-muted {
  color: var(--text-secondary);
}

.icon-computer {
  background: linear-gradient(135deg, rgba(148, 163, 184, 0.3), rgba(59, 130, 246, 0.35));
}

.icon-computer::before {
  content: "";
  width: 28px;
  height: 18px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 4px;
  position: absolute;
}

.icon-toolbox {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.4), rgba(248, 113, 113, 0.35));
}

.icon-toolbox::before {
  content: "";
  width: 26px;
  height: 16px;
  border: 2px solid rgba(255, 250, 227, 0.9);
  border-radius: 4px;
  position: absolute;
  top: 18px;
}

.icon-toolbox::after {
  content: "";
  width: 14px;
  height: 6px;
  border: 2px solid rgba(255, 250, 227, 0.9);
  border-bottom: none;
  border-radius: 4px 4px 0 0;
  position: absolute;
  top: 12px;
}

.icon-terminal {
  background: linear-gradient(135deg, rgba(34, 211, 238, 0.35), rgba(56, 189, 248, 0.4));
}

.icon-terminal::before {
  content: "›_";
  font-size: 14px;
  color: rgba(226, 232, 240, 0.9);
  font-weight: 600;
}

.icon-database {
  background: linear-gradient(135deg, rgba(167, 139, 250, 0.35), rgba(56, 189, 248, 0.25));
}

.icon-database::before {
  content: "";
  width: 24px;
  height: 18px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 12px / 6px;
  position: absolute;
}

.icon-trash {
  background: linear-gradient(135deg, rgba(148, 163, 184, 0.35), rgba(94, 234, 212, 0.25));
}

.icon-trash::before {
  content: "";
  width: 20px;
  height: 22px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 4px;
  position: absolute;
}

.icon-web {
  background: linear-gradient(135deg, rgba(94, 234, 212, 0.35), rgba(59, 130, 246, 0.3));
}

.icon-web::before {
  content: "";
  width: 24px;
  height: 24px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 50%;
  position: absolute;
}

.icon-files {
  background: linear-gradient(135deg, rgba(56, 189, 248, 0.35), rgba(250, 204, 21, 0.35));
}

.icon-files::before {
  content: "";
  width: 26px;
  height: 18px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 4px;
  position: absolute;
  top: 18px;
}

.icon-settings {
  background: linear-gradient(135deg, rgba(129, 140, 248, 0.35), rgba(94, 234, 212, 0.25));
}

.icon-settings::before {
  content: "";
  width: 22px;
  height: 22px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 50%;
  position: absolute;
}

.icon-monitor {
  background: linear-gradient(135deg, rgba(148, 163, 184, 0.3), rgba(56, 189, 248, 0.4));
}

.icon-monitor::before {
  content: "";
  width: 26px;
  height: 16px;
  border: 2px solid rgba(226, 232, 240, 0.9);
  border-radius: 4px;
  position: absolute;
}

.icon-apps {
  background: linear-gradient(135deg, rgba(94, 234, 212, 0.35), rgba(167, 139, 250, 0.25));
}

.icon-apps::before {
  content: "";
  width: 20px;
  height: 20px;
  background: rgba(226, 232, 240, 0.9);
  mask: radial-gradient(circle at 4px 4px, transparent 3px, #000 4px) 0 0/10px 10px;
  -webkit-mask: radial-gradient(circle at 4px 4px, transparent 3px, #000 4px) 0 0/10px 10px;
  position: absolute;
}
</style>
