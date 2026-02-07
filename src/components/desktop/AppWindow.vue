<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";

const props = defineProps<{
    title: string;
    appId: string;
    active: boolean;
    zIndex: number;
    offset: number;
    statusText?: string;
    statusOnline?: boolean;
}>();

const emit = defineEmits<{
    close: [];
    focus: [];
}>();

// 窗口大小和位置状态
const windowWidth = ref(0);
const windowHeight = ref(0);
const windowX = ref(0);
const windowY = ref(0);
const isResizing = ref(false);
const resizeDirection = ref("");

// 初始化窗口尺寸
const defaultSizes: Record<string, { width: number; height: number }> = {
    terminal: { width: 1120, height: 720 },
    files: { width: 980, height: 680 },
    monitor: { width: 1000, height: 700 },
    settings: { width: 860, height: 560 },
    "app-center": { width: 860, height: 560 },
    browser: { width: 980, height: 640 },
};

onMounted(() => {
    const size = defaultSizes[props.appId] || { width: 860, height: 560 };
    windowWidth.value = Math.min(size.width, window.innerWidth * 0.92);
    windowHeight.value = Math.min(size.height, window.innerHeight * 0.8);
});

const windowStyle = computed(() => ({
    width: `${windowWidth.value}px`,
    height: `${windowHeight.value}px`,
    top: `calc(8vh + ${props.offset}px)`,
    left: `calc(50% + ${props.offset}px)`,
    zIndex: props.zIndex,
}));

// Resize 处理 - 使用 requestAnimationFrame 优化性能
function startResize(e: MouseEvent, direction: string) {
    e.preventDefault();
    e.stopPropagation();
    isResizing.value = true;
    resizeDirection.value = direction;

    const startX = e.clientX;
    const startY = e.clientY;
    const startWidth = windowWidth.value;
    const startHeight = windowHeight.value;

    let currentX = e.clientX;
    let currentY = e.clientY;
    let rafId: number | null = null;

    function updateSize() {
        rafId = null;
        if (!isResizing.value) return;

        const deltaX = currentX - startX;
        const deltaY = currentY - startY;

        if (resizeDirection.value.includes("e")) {
            windowWidth.value = Math.max(400, startWidth + deltaX);
        }
        if (resizeDirection.value.includes("s")) {
            windowHeight.value = Math.max(300, startHeight + deltaY);
        }
        if (resizeDirection.value.includes("w")) {
            windowWidth.value = Math.max(400, startWidth - deltaX);
        }
        if (resizeDirection.value.includes("n")) {
            windowHeight.value = Math.max(300, startHeight - deltaY);
        }
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isResizing.value) return;
        
        currentX = e.clientX;
        currentY = e.clientY;

        if (!rafId) {
            rafId = requestAnimationFrame(updateSize);
        }
    }

    function handleMouseUp() {
        isResizing.value = false;
        resizeDirection.value = "";
        if (rafId) {
            cancelAnimationFrame(rafId);
        }
        document.removeEventListener("mousemove", handleMouseMove);
        document.removeEventListener("mouseup", handleMouseUp);
    }

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
}
</script>

<template>
    <div class="app-window" :class="[`app-window--${appId}`, { active, resizing: isResizing }]" :style="windowStyle"
        :data-resize-dir="isResizing ? resizeDirection : ''"
        @mousedown="emit('focus')">
        <header class="app-titlebar">
            <div class="window-controls">
                <button class="control control--close" @click.stop="emit('close')"></button>
                <button class="control control--min"></button>
                <button class="control control--max"></button>
            </div>
            <div class="app-title">{{ title }}</div>
            <div class="title-actions">
                <span v-if="statusText" class="status-pill" :class="{ online: statusOnline }">
                    {{ statusText }}
                </span>
            </div>
        </header>

        <div class="app-body">
            <slot />
        </div>

        <!-- Resize handles -->
        <div class="resize-handle resize-n" @mousedown="startResize($event, 'n')"></div>
        <div class="resize-handle resize-s" @mousedown="startResize($event, 's')"></div>
        <div class="resize-handle resize-e" @mousedown="startResize($event, 'e')"></div>
        <div class="resize-handle resize-w" @mousedown="startResize($event, 'w')"></div>
        <div class="resize-handle resize-ne" @mousedown="startResize($event, 'ne')"></div>
        <div class="resize-handle resize-nw" @mousedown="startResize($event, 'nw')"></div>
        <div class="resize-handle resize-se" @mousedown="startResize($event, 'se')"></div>
        <div class="resize-handle resize-sw" @mousedown="startResize($event, 'sw')"></div>
    </div>
</template>

<style scoped>
.app-window {
    pointer-events: auto;
    position: absolute;
    transform: translateX(-50%);
    border-radius: 18px;
    background: rgba(14, 18, 28, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: var(--shadow-strong);
    backdrop-filter: blur(20px);
    overflow: visible;
    transition: box-shadow 0.2s ease, transform 0.2s ease;
    min-width: 400px;
    min-height: 300px;
}

.app-window.active {
    box-shadow: 0 28px 80px rgba(0, 0, 0, 0.5);
    transform: translateX(-50%) translateY(-2px);
}

.app-window.resizing {
    transition: none;
    user-select: none;
}

/* Resizing 时统一使用对应方向的 cursor */
.app-window.resizing[data-resize-dir="n"] {
    cursor: n-resize !important;
}

.app-window.resizing[data-resize-dir="s"] {
    cursor: s-resize !important;
}

.app-window.resizing[data-resize-dir="e"] {
    cursor: e-resize !important;
}

.app-window.resizing[data-resize-dir="w"] {
    cursor: w-resize !important;
}

.app-window.resizing[data-resize-dir="ne"] {
    cursor: ne-resize !important;
}

.app-window.resizing[data-resize-dir="nw"] {
    cursor: nw-resize !important;
}

.app-window.resizing[data-resize-dir="se"] {
    cursor: se-resize !important;
}

.app-window.resizing[data-resize-dir="sw"] {
    cursor: sw-resize !important;
}

.app-titlebar {
    display: grid;
    grid-template-columns: 120px 1fr 160px;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(18, 22, 32, 0.8);
    cursor: default;
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
    cursor: pointer;
    transition: transform 0.15s ease;
}

.control:hover {
    transform: scale(1.1);
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
    overflow: auto;
    border-radius: 0 0 18px 18px;
}

/* Resize handles */
.resize-handle {
    position: absolute;
    z-index: 10000;
    pointer-events: auto;
}

/* 四边 */
.resize-n {
    left: 12px;
    right: 12px;
    top: 0;
    height: 8px;
    cursor: n-resize !important;
}

.resize-s {
    left: 12px;
    right: 12px;
    bottom: 0;
    height: 8px;
    cursor: s-resize !important;
}

.resize-e {
    top: 12px;
    bottom: 12px;
    right: 0;
    width: 8px;
    cursor: e-resize !important;
}

.resize-w {
    top: 12px;
    bottom: 12px;
    left: 0;
    width: 8px;
    cursor: w-resize !important;
}

/* 四角 */
.resize-ne {
    top: 0;
    right: 0;
    width: 12px;
    height: 12px;
    cursor: ne-resize !important;
}

.resize-nw {
    top: 0;
    left: 0;
    width: 12px;
    height: 12px;
    cursor: nw-resize !important;
}

.resize-se {
    bottom: 0;
    right: 0;
    width: 12px;
    height: 12px;
    cursor: se-resize !important;
}

.resize-sw {
    bottom: 0;
    left: 0;
    width: 12px;
    height: 12px;
    cursor: sw-resize !important;
}
</style>
