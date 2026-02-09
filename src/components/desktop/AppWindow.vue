<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from "vue";
import TrafficLights from "./TrafficLights.vue";

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

// Constants
const MIN_WIDTH = 400;
const MIN_HEIGHT = 300;

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
    const startWindowX = windowX.value;
    const startWindowY = windowY.value;

    let currentX = e.clientX;
    let currentY = e.clientY;
    let rafId: number | null = null;

    function updateSize() {
        rafId = null;
        if (!isResizing.value) return;

        const deltaX = currentX - startX;
        const deltaY = currentY - startY;

        if (resizeDirection.value.includes("e")) {
            windowWidth.value = Math.max(MIN_WIDTH, startWidth + deltaX);
        }
        if (resizeDirection.value.includes("s")) {
            windowHeight.value = Math.max(MIN_HEIGHT, startHeight + deltaY);
        }
        if (resizeDirection.value.includes("w")) {
            const newWidth = Math.max(MIN_WIDTH, startWidth - deltaX);
            // 只有当宽度确实变化时才更新 X 坐标
            if (newWidth !== startWidth) {
                const intendedWidth = startWidth - deltaX;
                if (intendedWidth >= MIN_WIDTH) {
                    windowWidth.value = intendedWidth;
                    windowX.value = startWindowX + deltaX;
                } else {
                    windowWidth.value = MIN_WIDTH;
                    windowX.value = startWindowX + (startWidth - MIN_WIDTH);
                }
            }
        }
        if (resizeDirection.value.includes("n")) {
            const intendedHeight = startHeight - deltaY;
            if (intendedHeight >= MIN_HEIGHT) {
                windowHeight.value = intendedHeight;
                windowY.value = startWindowY + deltaY;
            } else {
                windowHeight.value = MIN_HEIGHT;
                windowY.value = startWindowY + (startHeight - MIN_HEIGHT);
            }
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
    <div class="fixed rounded-[18px] border border-border/40 shadow-2xl backdrop-blur-2xl transition-shadow bg-background/90"
        :class="[`app-window--${appId}`, { 'shadow-[0_28px_80px_rgba(0,0,0,0.5)]': active, 'transition-none select-none': isResizing || isDragging, 'border-none bg-transparent shadow-[0_10px_30px_rgba(0,0,0,0.3)]': customChrome }]"
        :style="windowStyle" :data-resize-dir="isResizing ? resizeDirection : ''" @mousedown="emit('focus')">

        <header v-if="!customChrome"
            class="grid grid-cols-[120px_1fr_160px] items-center px-4 py-3 border-b border-white/5 bg-[#121620]/80 rounded-t-[18px] select-none cursor-grab active:cursor-grabbing"
            @mousedown="startDrag">
            <TrafficLights
                @close="emit('close')"
                @minimize="emit('minimize')"
                @maximize="emit('maximize')"
            />
            <div class="text-center text-[0.9rem] tracking-[0.08em] text-muted-foreground uppercase">{{ title }}</div>
            <div class="flex justify-end">
                <span v-if="statusText"
                    class="text-[0.72rem] px-2.5 py-1 rounded-full bg-white/10 text-muted-foreground"
                    :class="{ 'bg-[rgba(94,234,212,0.18)] text-[#bff4ea]': statusOnline }">
                    {{ statusText }}
                </span>
            </div>
        </header>

        <div class="overflow-auto rounded-b-[18px] flex flex-col" :class="[
            customChrome ? 'h-full rounded-[18px]' : 'h-[calc(100%-48px)]',
            { 'pointer-events-none': isResizing || isDragging }
        ]">
            <slot :start-drag="startDrag" :close="() => emit('close')" :minimize="() => emit('minimize')"
                :maximize="() => emit('maximize')" />
        </div>


        <!-- Resize handles -->
        <!-- North -->
        <div class="absolute z-[99999] pointer-events-auto left-4 right-4 -top-2 h-4 cursor-ns-resize"
            @mouseenter="setResizeCursor('n')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'n')">
        </div>
        <!-- South -->
        <div class="absolute z-[99999] pointer-events-auto left-4 right-4 -bottom-2 h-4 cursor-ns-resize"
            @mouseenter="setResizeCursor('s')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 's')">
        </div>
        <!-- East -->
        <div class="absolute z-[99999] pointer-events-auto top-4 bottom-4 -right-2 w-4 cursor-ew-resize"
            @mouseenter="setResizeCursor('e')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'e')">
        </div>
        <!-- West -->
        <div class="absolute z-[99999] pointer-events-auto top-4 bottom-4 -left-2 w-4 cursor-ew-resize"
            @mouseenter="setResizeCursor('w')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'w')">
        </div>

        <!-- Corner Handles & Visuals -->
        <!-- NE -->
        <div class="absolute z-[99999] pointer-events-auto -top-2 -right-2 w-8 h-8 cursor-nesw-resize group flex items-start justify-end p-1"
            @mouseenter="setResizeCursor('ne')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'ne')">
            <span
                class="iconify mdi--resize-bottom-right rotate-180 text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"></span>
        </div>
        <!-- NW -->
        <div class="absolute z-[99999] pointer-events-auto -top-2 -left-2 w-8 h-8 cursor-nwse-resize group flex items-start justify-start p-1"
            @mouseenter="setResizeCursor('nw')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'nw')">
            <span
                class="iconify mdi--resize-bottom-right rotate-[270deg] text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"></span>
        </div>
        <!-- SE -->
        <div class="absolute z-[99999] pointer-events-auto -bottom-2 -right-2 w-8 h-8 cursor-nwse-resize group flex items-end justify-end p-1"
            @mouseenter="setResizeCursor('se')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'se')">
            <span
                class="iconify mdi--resize-bottom-right text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"></span>
        </div>
        <!-- SW -->
        <div class="absolute z-[99999] pointer-events-auto -bottom-2 -left-2 w-8 h-8 cursor-nesw-resize group flex items-end justify-start p-1"
            @mouseenter="setResizeCursor('sw')" @mouseleave="handleMouseLeave" @mousedown="startResize($event, 'sw')">
            <span
                class="iconify mdi--resize-bottom-right rotate-90 text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"></span>
        </div>
    </div>
</template>

<style scoped>
/* Remove scoped styles as they are replaced by Tailwind */
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
