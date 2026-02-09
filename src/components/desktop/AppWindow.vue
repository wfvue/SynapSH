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
    customChrome?: boolean;
}>();

const emit = defineEmits<{
    close: [];
    focus: [];
    minimize: [];
    maximize: [];
}>();

// 全局 Body Class Cursor 覆盖 (解决 WebView Cursor 问题)
function setResizeCursor(direction: string) {
    const cursorClass = `cursor-${direction}-resize`;
    document.body.classList.add(cursorClass);
}

function handleMouseLeave() {
    resetCursor();
}

function resetCursor() {
    document.body.classList.remove(
        "cursor-n-resize",
        "cursor-s-resize",
        "cursor-e-resize",
        "cursor-w-resize",
        "cursor-ne-resize",
        "cursor-nw-resize",
        "cursor-se-resize",
        "cursor-sw-resize"
    );
}

// 窗口大小和位置状态
const windowWidth = ref(0);
const windowHeight = ref(0);
const windowX = ref(0);
const windowY = ref(0);
const isResizing = ref(false);
const resizeDirection = ref("");
const isDragging = ref(false);

// 初始化窗口尺寸
const defaultSizes: Record<string, { width: number; height: number }> = {
    terminal: { width: 1120, height: 720 },
    files: { width: 980, height: 680 },
    monitor: { width: 1000, height: 700 },
    settings: { width: 860, height: 560 },
    "app-center": { width: 860, height: 560 },
    browser: { width: 980, height: 640 },
};

const windowStyle = computed(() => ({
    width: `${windowWidth.value}px`,
    height: `${windowHeight.value}px`,
    top: `${windowY.value}px`,
    left: `${windowX.value}px`,
    zIndex: props.zIndex,
}));

// 初始化窗口位置
onMounted(() => {
    const size = defaultSizes[props.appId] || { width: 860, height: 560 };
    windowWidth.value = Math.min(size.width, window.innerWidth * 0.92);
    windowHeight.value = Math.min(size.height, window.innerHeight * 0.8);

    // 计算居中位置
    windowX.value = (window.innerWidth - windowWidth.value) / 2 + props.offset;
    windowY.value = window.innerHeight * 0.08 + props.offset;
});

// 拖拽处理
function startDrag(e: MouseEvent) {
    // 如果点击的是按钮，不启动拖拽
    if ((e.target as HTMLElement).closest('button')) return;

    e.preventDefault();
    isDragging.value = true;

    const startX = e.clientX;
    const startY = e.clientY;
    const startWindowX = windowX.value;
    const startWindowY = windowY.value;

    let currentX = e.clientX;
    let currentY = e.clientY;
    let rafId: number | null = null;

    function updatePosition() {
        rafId = null;
        if (!isDragging.value) return;

        const deltaX = currentX - startX;
        const deltaY = currentY - startY;

        windowX.value = startWindowX + deltaX;
        windowY.value = Math.max(0, startWindowY + deltaY); // 防止拖出顶部
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isDragging.value) return;

        currentX = e.clientX;
        currentY = e.clientY;

        if (!rafId) {
            rafId = requestAnimationFrame(updatePosition);
        }
    }

    function handleMouseUp() {
        isDragging.value = false;
        if (rafId) {
            cancelAnimationFrame(rafId);
        }
        document.removeEventListener('mousemove', handleMouseMove);
        document.removeEventListener('mouseup', handleMouseUp);
    }

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
}

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
    <div class="app-window bg-background/90"
        :class="[`app-window--${appId}`, { active, resizing: isResizing, dragging: isDragging, 'custom-chrome': customChrome }]"
        :style="windowStyle" :data-resize-dir="isResizing ? resizeDirection : ''" @mousedown="emit('focus')">

        <header v-if="!customChrome" class="app-titlebar" @mousedown="startDrag">
            <div class="window-controls">
                <button class="control control--close" @click.stop="emit('close')"></button>
                <button class="control control--min" @click.stop="emit('minimize')"></button>
                <button class="control control--max" @click.stop="emit('maximize')"></button>
            </div>
            <div class="app-title">{{ title }}</div>
            <div class="title-actions">
                <span v-if="statusText" class="status-pill" :class="{ online: statusOnline }">
                    {{ statusText }}
                </span>
            </div>
        </header>

        <div class="app-body">
            <slot :start-drag="startDrag" :close="() => emit('close')" :minimize="() => emit('minimize')"
                :maximize="() => emit('maximize')" />
        </div>


        <!-- Resize handles -->
        <!-- Resize handles -->
        <div class="resize-handle resize-n" @mouseenter="setResizeCursor('n')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'n')"></div>
        <div class="resize-handle resize-s" @mouseenter="setResizeCursor('s')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 's')"></div>
        <div class="resize-handle resize-e" @mouseenter="setResizeCursor('e')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'e')"></div>
        <div class="resize-handle resize-w" @mouseenter="setResizeCursor('w')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'w')"></div>
        <div class="resize-handle resize-ne" @mouseenter="setResizeCursor('ne')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'ne')"></div>
        <div class="resize-handle resize-nw" @mouseenter="setResizeCursor('nw')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'nw')"></div>
        <div class="resize-handle resize-se" @mouseenter="setResizeCursor('se')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'se')"></div>
        <div class="resize-handle resize-sw" @mouseenter="setResizeCursor('sw')" @mouseleave="handleMouseLeave"
            @mousedown="startResize($event, 'sw')"></div>
    </div>
</template>

<style scoped>
.app-window {
    pointer-events: auto;
    position: absolute;
    border-radius: 18px;
    border: 1px solid var(--border);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(20px);
    overflow: visible;
    transition: box-shadow 0.2s ease, background 0.3s ease;
    min-width: 400px;
    min-height: 300px;
}

.app-window.active {
    box-shadow: 0 28px 80px rgba(0, 0, 0, 0.5);
}

.app-window.resizing {
    transition: none;
    user-select: none;
}

/* Resizing 时统一使用对应方向的 cursor */
.app-window.resizing[data-resize-dir="n"] {
    cursor: ns-resize !important;
}

.app-window.resizing[data-resize-dir="s"] {
    cursor: ns-resize !important;
}

.app-window.resizing[data-resize-dir="e"] {
    cursor: ew-resize !important;
}

.app-window.resizing[data-resize-dir="w"] {
    cursor: ew-resize !important;
}

.app-window.resizing[data-resize-dir="ne"] {
    cursor: nesw-resize !important;
}

.app-window.resizing[data-resize-dir="nw"] {
    cursor: nwse-resize !important;
}

.app-window.resizing[data-resize-dir="se"] {
    cursor: nwse-resize !important;
}

.app-window.resizing[data-resize-dir="sw"] {
    cursor: nesw-resize !important;
}

.app-titlebar {
    display: grid;
    grid-template-columns: 120px 1fr 160px;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(18, 22, 32, 0.8);
    border-radius: 18px 18px 0 0;
    cursor: grab;
    user-select: none;
}

.app-titlebar:active {
    cursor: grabbing;
}

.app-window.dragging {
    transition: none;
    user-select: none;
}

.app-window.dragging .app-titlebar {
    cursor: grabbing;
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
    color: var(--muted-foreground);
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
    color: var(--muted-foreground);
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

.app-window.custom-chrome {
    border: none;
    background: transparent;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
    /* Let content handle bg or keep it here */
}

.app-window.custom-chrome .app-body {
    height: 100%;
    border-radius: 18px;
}

/* Resize handles */
.resize-handle {
    position: absolute;
    z-index: 99999;
    pointer-events: auto;
    -webkit-app-region: no-drag;
    transition: background-color 0.15s ease;
    background: rgba(255, 255, 255, 1e-4);
    /* Fix for macOS WKWebView cursor issue */
}

/* 四边 - 增大交互区域 */
.resize-n {
    left: 16px;
    right: 16px;
    top: -4px;
    height: 12px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M16 4l-6 8h4v8h-4l6 8 6-8h-4v-8h4z"/></svg>') 16 16, ns-resize !important;
}

.resize-s {
    left: 16px;
    right: 16px;
    bottom: -4px;
    height: 12px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M16 4l-6 8h4v8h-4l6 8 6-8h-4v-8h4z"/></svg>') 16 16, ns-resize !important;
}

.resize-e {
    top: 16px;
    bottom: 16px;
    right: -4px;
    width: 12px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M28 16l-8-6v4h-8v-4l-8 6 8 6v-4h8v4z"/></svg>') 16 16, ew-resize !important;
}

.resize-w {
    top: 16px;
    bottom: 16px;
    left: -4px;
    width: 12px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M28 16l-8-6v4h-8v-4l-8 6 8 6v-4h8v4z"/></svg>') 16 16, ew-resize !important;
}

/* 四角 - 增大交互区域 */
.resize-ne {
    top: -4px;
    right: -4px;
    width: 16px;
    height: 16px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M22.5 9.5l-8.5 2.5 1.5 1.5-6 6 1.5 1.5 6-6 1.5 1.5 2.5-8.5z"/></svg>') 16 16, nesw-resize !important;
}

.resize-nw {
    top: -4px;
    left: -4px;
    width: 16px;
    height: 16px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M9.5 9.5l8.5 2.5-1.5 1.5 6 6-1.5 1.5-6-6-1.5 1.5-2.5-8.5z"/></svg>') 16 16, nwse-resize !important;
}

.resize-se {
    bottom: -4px;
    right: -4px;
    width: 16px;
    height: 16px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M9.5 9.5l8.5 2.5-1.5 1.5 6 6-1.5 1.5-6-6-1.5 1.5-2.5-8.5z"/></svg>') 16 16, nwse-resize !important;
}

.resize-sw {
    bottom: -4px;
    left: -4px;
    width: 16px;
    height: 16px;
    pointer-events: auto;
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M22.5 9.5l-8.5 2.5 1.5 1.5-6 6 1.5 1.5 6-6 1.5 1.5 2.5-8.5z"/></svg>') 16 16, nesw-resize !important;
}
</style>

<style>
/* Global cursor overrides */
/* Global cursor overrides using SVG Data URIs for reliability */
body.cursor-n-resize,
body.cursor-s-resize {
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M16 4l-6 8h4v8h-4l6 8 6-8h-4v-8h4z"/></svg>') 16 16, ns-resize !important;
}

body.cursor-e-resize,
body.cursor-w-resize {
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M28 16l-8-6v4h-8v-4l-8 6 8 6v-4h8v4z"/></svg>') 16 16, ew-resize !important;
}

body.cursor-ne-resize,
body.cursor-sw-resize {
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M22.5 9.5l-8.5 2.5 1.5 1.5-6 6 1.5 1.5 6-6 1.5 1.5 2.5-8.5z"/></svg>') 16 16, nesw-resize !important;
}

body.cursor-nw-resize,
body.cursor-se-resize {
    cursor: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M9.5 9.5l8.5 2.5-1.5 1.5 6 6-1.5 1.5-6-6-1.5 1.5-2.5-8.5z"/></svg>') 16 16, nwse-resize !important;
}
</style>
