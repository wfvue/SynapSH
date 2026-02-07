<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const props = defineProps<{
    isConnected: boolean;
}>();

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
</script>

<template>
    <section class="status-bar">
        <div class="status-item">
            <span class="status-dot" :class="{ offline: !isConnected }"></span>
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
</template>

<style scoped>
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

.status-dot.offline {
    background: rgba(248, 113, 113, 0.8);
}

.status-muted {
    color: var(--text-secondary);
}
</style>
