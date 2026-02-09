<!--
  SettingsApp.vue - 系统设置应用主组件
  macOS 风格的设置面板，sidebar + content 布局
-->
<script setup lang="ts">
import { ref } from "vue";
import GeneralPanel from "./settings/GeneralPanel.vue";
import AppearancePanel from "./settings/AppearancePanel.vue";
import TerminalPanel from "./settings/TerminalPanel.vue";
import ConnectionPanel from "./settings/ConnectionPanel.vue";
import AboutPanel from "./settings/AboutPanel.vue";

defineProps<{
    sessionId: string;
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
</script>

<template>
    <div class="settings-app">
        <!-- 侧边栏导航 -->
        <aside class="settings-sidebar">
            <div class="sidebar-header">
                <span class="icon-[mdi--cog] header-icon"></span>
                <span class="header-title">设置</span>
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
        <main class="settings-content">
            <GeneralPanel v-if="activePanel === 'general'" />
            <AppearancePanel v-else-if="activePanel === 'appearance'" />
            <TerminalPanel v-else-if="activePanel === 'terminal'" />
            <ConnectionPanel v-else-if="activePanel === 'connection'" />
            <AboutPanel v-else-if="activePanel === 'about'" />
        </main>
    </div>
</template>

<style scoped>
.settings-app {
    height: 100%;
    display: grid;
    grid-template-columns: 180px 1fr;
    background: rgba(14, 18, 28, 0.6);
    border-radius: 0 0 16px 16px;
    overflow: hidden;
}

/* Sidebar */
.settings-sidebar {
    display: flex;
    flex-direction: column;
    background: rgba(0, 0, 0, 0.25);
    border-right: 1px solid rgba(255, 255, 255, 0.04);
}

.sidebar-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 16px 16px 12px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.header-icon {
    font-size: 20px;
    color: rgba(255, 255, 255, 0.5);
}

.header-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.7);
    text-transform: uppercase;
    letter-spacing: 0.04em;
}

.sidebar-nav {
    flex: 1;
    padding: 12px 8px;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 8px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.55);
    transition: all 0.2s ease;
}

.nav-item:hover {
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.8);
}

.nav-item.active {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
}

.nav-icon {
    font-size: 18px;
}

.nav-label {
    font-size: 0.85rem;
    font-weight: 500;
}

/* Content */
.settings-content {
    overflow-y: auto;
    background: rgba(255, 255, 255, 0.02);
}

/* Scrollbar */
.settings-content::-webkit-scrollbar {
    width: 6px;
}

.settings-content::-webkit-scrollbar-track {
    background: transparent;
}

.settings-content::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
}

.settings-content::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
}
</style>
