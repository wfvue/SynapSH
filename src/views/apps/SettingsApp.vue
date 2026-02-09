<!--
  SettingsApp.vue - 系统设置应用主组件
  macOS 风格的设置面板，sidebar + content 布局
-->
<script setup lang="ts">
import { ref, computed } from "vue";
import GeneralPanel from "./settings/GeneralPanel.vue";
import AppearancePanel from "./settings/AppearancePanel.vue";
import TerminalPanel from "./settings/TerminalPanel.vue";
import ConnectionPanel from "./settings/ConnectionPanel.vue";
import AboutPanel from "./settings/AboutPanel.vue";

defineProps<{
    sessionId: string;
    startDrag?: (e: MouseEvent) => void;
    close?: () => void;
    minimize?: () => void;
    maximize?: () => void;
}>();

type PanelId = "general" | "appearance" | "terminal" | "connection" | "about";

const activePanel = ref<PanelId>("general");

const panels: { id: PanelId; label: string; icon: string }[] = [
    { id: "general", label: "通用", icon: "icon-[mdi--cog]" },
    { id: "appearance", label: "外观", icon: "icon-[mdi--palette]" },
    { id: "terminal", label: "终端", icon: "icon-[mdi--console]" },
    { id: "connection", label: "连接", icon: "icon-[mdi--connection]" },
    { id: "about", label: "关于", icon: "icon-[mdi--information]" },
];

const currentPanelLabel = computed(() => {
    return panels.find((p) => p.id === activePanel.value)?.label || "设置";
});
</script>

<template>
    <div class="settings-app">
        <!-- 侧边栏 -->
        <aside class="settings-sidebar">
            <div class="sidebar-header" @mousedown="startDrag">
                <!-- Mac Traffic Lights -->
                <div class="window-controls">
                    <button class="control control--close" @click.stop="close"></button>
                    <button class="control control--min" @click.stop="minimize"></button>
                    <button class="control control--max" @click.stop="maximize"></button>
                </div>

                <div class="search-container">
                    <span class="icon-[mdi--magnify] search-icon"></span>
                    <input type="text" placeholder="搜索" class="search-input" />
                </div>
            </div>

            <nav class="sidebar-nav">
                <div v-for="panel in panels" :key="panel.id" class="nav-item"
                    :class="{ active: activePanel === panel.id }" @click="activePanel = panel.id">
                    <span :class="panel.icon" class="nav-icon"></span>
                    <span class="nav-label">{{ panel.label }}</span>
                </div>
            </nav>
        </aside>

        <!-- 内容区域 -->
        <div class="settings-main">
            <main class="settings-content">
                <GeneralPanel v-if="activePanel === 'general'" />
                <AppearancePanel v-else-if="activePanel === 'appearance'" />
                <TerminalPanel v-else-if="activePanel === 'terminal'" />
                <ConnectionPanel v-else-if="activePanel === 'connection'" />
                <AboutPanel v-else-if="activePanel === 'about'" />
            </main>
        </div>
    </div>
</template>

<style scoped>
.settings-app {
    height: 100%;
    display: grid;
    grid-template-columns: 240px 1fr;
    background: rgba(14, 18, 28, 0.85);
    border-radius: 0 0 16px 16px;
    overflow: hidden;
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.08);
}

/* Sidebar */
.settings-sidebar {
    display: flex;
    flex-direction: column;
    background: rgba(0, 0, 0, 0.2);
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    padding-top: 10px;
}

.sidebar-header {
    padding: 16px 16px 10px;
    margin-bottom: 4px;
    border-bottom: none;
    display: flex;
    flex-direction: column;
    gap: 12px;
    -webkit-app-region: drag;
    /* Allow dragging natively if supported, but we use custom drag */
}

.window-controls {
    display: flex;
    gap: 8px;
    margin-bottom: 2px;
}

.control {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.25);
    cursor: pointer;
    transition: transform 0.1s ease, background-color 0.1s;
    position: relative;
    overflow: hidden;
}

.control::before {
    content: '';
    position: absolute;
    inset: 0;
    opacity: 0;
    transition: opacity 0.1s;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 8px;
    color: rgba(0, 0, 0, 0.5);
    font-weight: bold;
}

.control:hover {
    /* transform: scale(1.1); // No scale on macos hover usually, just symbol */
}

.settings-app:hover .control--close::before {
    content: '×';
    opacity: 1;
}

.settings-app:hover .control--min::before {
    content: '−';
    opacity: 1;
}

.settings-app:hover .control--max::before {
    content: '+';
    opacity: 1;
}

.control--close {
    background: #ff5f56;
    box-shadow: 0 0 0 0.5px #e0443e;
}

.control--min {
    background: #ffbd2e;
    box-shadow: 0 0 0 0.5px #dea123;
}

.control--max {
    background: #27c93f;
    box-shadow: 0 0 0 0.5px #1aab29;
}

.control:active {
    filter: brightness(0.8);
}

.search-container {
    position: relative;
    width: 100%;
}

.search-icon {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    color: rgba(255, 255, 255, 0.4);
    font-size: 16px;
}

.search-input {
    width: 100%;
    height: 28px;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    padding: 0 8px 0 30px;
    color: white;
    font-size: 13px;
    outline: none;
    transition: all 0.2s;
}

.search-input:focus {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.2);
}

.search-input::placeholder {
    color: rgba(255, 255, 255, 0.3);
}

.sidebar-nav {
    flex: 1;
    padding: 0 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    overflow-y: auto;
}

.nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 10px;
    border-radius: 6px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.7);
    transition: all 0.15s ease;
    height: 34px;
}

.nav-item:hover {
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.9);
}

.nav-item.active {
    background: var(--accent-color, #3b82f6);
    color: white;
}

.nav-icon {
    font-size: 18px;
    opacity: 0.9;
}

.nav-label {
    font-size: 13px;
    font-weight: 500;
}

/* Main Area */
.settings-main {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}

/* Content Header */
.content-header {
    height: 52px;
    display: flex;
    align-items: center;
    padding: 0 24px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(255, 255, 255, 0.02);
}

.nav-controls {
    display: flex;
    gap: 8px;
    margin-right: 16px;
}

.nav-btn {
    width: 24px;
    height: 24px;
    border-radius: 4px;
    /* MacOS rounded style */
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.6);
    display: grid;
    place-items: center;
    cursor: pointer;
    font-size: 20px;
}

.nav-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: white;
}

.nav-btn:disabled {
    opacity: 0.3;
    cursor: default;
}

.content-title {
    font-size: 15px;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
}

/* Content */
.settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
    /* Added padding to replace header spacing */
}

/* Scrollbar */
.settings-content::-webkit-scrollbar,
.sidebar-nav::-webkit-scrollbar {
    width: 6px;
}

.settings-content::-webkit-scrollbar-track,
.sidebar-nav::-webkit-scrollbar-track {
    background: transparent;
}

.settings-content::-webkit-scrollbar-thumb,
.sidebar-nav::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
}

.settings-content::-webkit-scrollbar-thumb:hover,
.sidebar-nav::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
}
</style>
