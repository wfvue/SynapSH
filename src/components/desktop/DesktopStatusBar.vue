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
    mode.value = mode.value === 'dark' ? 'light' : 'dark';
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
    <header class="menu-bar">
        <div class="left-section">
            <div class="apple-logo">
                <span class="icon-[mdi--apple]"></span>
            </div>
            <span class="app-name font-bold">SynapSH</span>
            <nav class="menu-items">
                <span>文件</span>
                <span>编辑</span>
                <span>视图</span>
                <span>窗口</span>
                <span>帮助</span>
            </nav>
        </div>

        <div class="right-section">
            <div class="status-item" title="SSH 连接状态">
                <span class="icon-[mdi--connection] text-lg"
                    :class="{ 'text-green-500': isConnected, 'text-red-500': !isConnected }"></span>
            </div>

            <div class="status-item clickable" @click="toggleTheme" title="切换主题">
                <span v-if="mode === 'dark'" class="icon-[mdi--weather-night] text-lg"></span>
                <span v-else class="icon-[mdi--weather-sunny] text-lg"></span>
            </div>

            <div class="status-item">
                <span class="icon-[mdi--wifi] text-lg"></span>
            </div>

            <div class="status-item">
                <span class="icon-[mdi--battery-70] text-lg"></span>
            </div>

            <div class="clock-section">
                <span>{{ dateText }}</span>
                <span>{{ timeText }}</span>
            </div>
        </div>
    </header>
</template>

<style scoped>
.menu-bar {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 32px;
    background: rgba(255, 255, 255, 0.3);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 16px;
    z-index: 1000;
    font-size: 13px;
    font-weight: 500;
    color: var(--foreground);
    border-bottom: 1px solid rgba(0, 0, 0, 0.05);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.02);
    user-select: none;
}

.dark .menu-bar {
    background: rgba(0, 0, 0, 0.3);
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    color: #ffffff;
}

.left-section,
.right-section {
    display: flex;
    align-items: center;
    gap: 16px;
}

.apple-logo {
    display: flex;
    align-items: center;
    font-size: 16px;
    opacity: 0.9;
}

.menu-items {
    display: flex;
    gap: 16px;
    opacity: 0.9;
}

.status-item {
    display: flex;
    align-items: center;
    opacity: 0.9;
}

.clickable {
    cursor: pointer;
}

.clickable:hover {
    opacity: 1;
}

.clock-section {
    display: flex;
    gap: 8px;
}
</style>
