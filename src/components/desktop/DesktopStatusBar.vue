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
    <header
        class="absolute top-0 inset-x-0 h-8 bg-white/30 dark:bg-black/30 backdrop-blur-2xl flex justify-between items-center px-4 z-50 text-[13px] font-medium text-foreground border-b border-black/5 dark:border-white/5 shadow-sm select-none">
        <div class="flex items-center gap-4">
            <div class="flex items-center text-base opacity-90">
                <span class="icon-[mdi--apple]"></span>
            </div>
            <span class="font-bold">SynapSH</span>
            <nav class="flex gap-4 opacity-90">
                <span>文件</span>
                <span>编辑</span>
                <span>视图</span>
                <span>窗口</span>
                <span>帮助</span>
            </nav>
        </div>

        <div class="flex items-center gap-4">
            <div class="flex items-center opacity-90" title="SSH 连接状态">
                <span class="icon-[mdi--connection] text-lg"
                    :class="{ 'text-green-500': isConnected, 'text-red-500': !isConnected }"></span>
            </div>

            <div class="flex items-center opacity-90 cursor-pointer hover:opacity-100" @click="toggleTheme"
                title="切换主题">
                <span v-if="mode === 'dark'" class="icon-[mdi--weather-night] text-lg"></span>
                <span v-else class="icon-[mdi--weather-sunny] text-lg"></span>
            </div>

            <div class="flex items-center opacity-90">
                <span class="icon-[mdi--wifi] text-lg"></span>
            </div>

            <div class="flex items-center opacity-90">
                <span class="icon-[mdi--battery-70] text-lg"></span>
            </div>

            <div class="flex gap-2">
                <span>{{ dateText }}</span>
                <span>{{ timeText }}</span>
            </div>
        </div>
    </header>
</template>


/* Scoped styles replaced by Tailwind CSS */
