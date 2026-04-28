<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted, watch } from "vue";
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
  minimized?: boolean;
}>();

const emit = defineEmits<{
  close: [];
  focus: [];
  minimize: [];
  maximize: [];
  restore: [];
}>();

const MIN_WIDTH = 400;
const MIN_HEIGHT = 300;

const isMaximized = ref(false);
const isFullscreen = ref(false);
const isMinimized = ref(false);
const savedPosition = ref({ x: 0, y: 0, width: 0, height: 0 });
const buttonPressState = ref<"minimize" | "maximize" | "close" | null>(null);

const windowWidth = ref(0);
const windowHeight = ref(0);
const windowX = ref(0);
const windowY = ref(0);
const isResizing = ref(false);
const resizeDirection = ref("");
const isDragging = ref(false);

const defaultSizes: Record<string, { width: number; height: number }> = {
  terminal: { width: 1120, height: 720 },
  files: { width: 980, height: 680 },
  monitor: { width: 1000, height: 700 },
  settings: { width: 860, height: 560 },
  "app-center": { width: 860, height: 560 },
  browser: { width: 980, height: 640 },
};

const windowStyle = computed(() => {
  if (isFullscreen.value) {
    return {
      width: "100%",
      height: "100%",
      top: "0",
      left: "0",
      zIndex: props.zIndex,
    };
  }
  if (isMaximized.value) {
    return {
      width: "100%",
      height: "100%",
      top: "0",
      left: "0",
      zIndex: props.zIndex,
    };
  }
  return {
    width: `${windowWidth.value}px`,
    height: `${windowHeight.value}px`,
    top: `${windowY.value}px`,
    left: `${windowX.value}px`,
    zIndex: props.zIndex,
  };
});

const windowClass = computed(() => [
  "fixed rounded-lg border border-white/10 backdrop-blur-xl transition-all duration-200 bg-background/95",
  `app-window--${props.appId}`,
  {
    "shadow-[0_32px_64px_rgba(0,0,0,0.5),0_2px_12px_rgba(0,0,0,0.2)]":
      props.active && !isMaximized.value && !props.minimized,
    "shadow-[0_8px_32px_rgba(0,0,0,0.3)]": !props.active && !isMaximized.value && !props.minimized,
    "shadow-none border-0 rounded-none": isMaximized.value || isFullscreen.value,
    "transition-none select-none": (isResizing.value || isDragging.value) && !props.minimized,
    "border-none bg-transparent shadow-[0_10px_30px_rgba(0,0,0,0.3)]":
      props.customChrome && !props.minimized,
    "scale-[0.98]": buttonPressState.value === "minimize",
    "cursor-default": isMaximized.value || isFullscreen.value,
    "opacity-0 pointer-events-none scale-95 translate-y-4": props.minimized,
  },
]);

const resizeHandlesVisible = computed(
  () => !isMaximized.value && !isFullscreen.value && !props.customChrome,
);

const headerClass = computed(() => [
  "flex items-center justify-between pl-4 pr-0 h-[36px] bg-transparent select-none",
  { "cursor-grab active:cursor-grabbing": !isMaximized.value && !isFullscreen.value },
  { "rounded-none": isMaximized.value || isFullscreen.value },
  { "rounded-t-lg": !isMaximized.value && !isFullscreen.value },
]);

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
    "cursor-sw-resize",
  );
}

function saveNormalState() {
  savedPosition.value = {
    x: windowX.value,
    y: windowY.value,
    width: windowWidth.value,
    height: windowHeight.value,
  };
}

function handleMinimize() {
  buttonPressState.value = "minimize";
  isMinimized.value = true;
  emit("minimize");
  setTimeout(() => {
    isMinimized.value = false;
    buttonPressState.value = null;
  }, 150);
}

function handleMaximize() {
  buttonPressState.value = "maximize";
  if (isMaximized.value) {
    windowX.value = savedPosition.value.x;
    windowY.value = savedPosition.value.y;
    windowWidth.value = savedPosition.value.width;
    windowHeight.value = savedPosition.value.height;
    isMaximized.value = false;
    emit("restore");
  } else {
    saveNormalState();
    isMaximized.value = true;
    emit("maximize");
  }
  setTimeout(() => {
    buttonPressState.value = null;
  }, 150);
}

function handleClose() {
  buttonPressState.value = "close";
  emit("close");
  setTimeout(() => {
    buttonPressState.value = null;
  }, 150);
}

onMounted(() => {
  const size = defaultSizes[props.appId] || { width: 860, height: 560 };
  windowWidth.value = Math.min(size.width, window.innerWidth * 0.92);
  windowHeight.value = Math.min(size.height, window.innerHeight * 0.8);
  windowX.value = (window.innerWidth - windowWidth.value) / 2 + props.offset;
  windowY.value = window.innerHeight * 0.08 + props.offset;

  savedPosition.value = {
    x: windowX.value,
    y: windowY.value,
    width: windowWidth.value,
    height: windowHeight.value,
  };

  window.addEventListener("resize", handleWindowResize);
});

onUnmounted(() => {
  window.removeEventListener("resize", handleWindowResize);
});

function handleWindowResize() {
  if (!isMaximized.value && !isFullscreen.value) {
    const maxWidth = window.innerWidth - 40;
    const maxHeight = window.innerHeight - 40;
    if (windowWidth.value > maxWidth) windowWidth.value = maxWidth;
    if (windowHeight.value > maxHeight) windowHeight.value = maxHeight;
  }
}

watch(
  () => props.active,
  (active) => {
    if (active && isMinimized.value) {
      isMinimized.value = false;
    }
  },
);

function startDrag(e: MouseEvent) {
  if ((e.target as HTMLElement).closest("button")) return;
  if (isMaximized.value || isFullscreen.value) return;

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

    windowX.value = Math.max(0, startWindowX + deltaX);
    windowY.value = Math.max(0, startWindowY + deltaY);
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
    if (rafId) cancelAnimationFrame(rafId);
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  }

  document.addEventListener("mousemove", handleMouseMove);
  document.addEventListener("mouseup", handleMouseUp);
}

function startResize(e: MouseEvent, direction: string) {
  if (isMaximized.value || isFullscreen.value || props.customChrome) return;

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
      windowWidth.value = Math.max(
        MIN_WIDTH,
        Math.min(startWidth + deltaX, window.innerWidth - windowX.value - 20),
      );
    }
    if (resizeDirection.value.includes("s")) {
      windowHeight.value = Math.max(
        MIN_HEIGHT,
        Math.min(startHeight + deltaY, window.innerHeight - windowY.value - 20),
      );
    }
    if (resizeDirection.value.includes("w")) {
      const newWidth = Math.max(MIN_WIDTH, startWidth - deltaX);
      if (newWidth !== startWidth) {
        const intendedWidth = startWidth - deltaX;
        if (intendedWidth >= MIN_WIDTH && windowX.value + intendedWidth < window.innerWidth - 10) {
          windowWidth.value = intendedWidth;
          windowX.value = startWindowX + deltaX;
        } else {
          windowWidth.value = MIN_WIDTH;
          windowX.value = Math.max(0, startWindowX + (startWidth - MIN_WIDTH));
        }
      }
    }
    if (resizeDirection.value.includes("n")) {
      const intendedHeight = startHeight - deltaY;
      if (
        intendedHeight >= MIN_HEIGHT &&
        windowY.value + intendedHeight < window.innerHeight - 10
      ) {
        windowHeight.value = intendedHeight;
        windowY.value = startWindowY + deltaY;
      } else {
        windowHeight.value = MIN_HEIGHT;
        windowY.value = Math.max(0, startWindowY + (startHeight - MIN_HEIGHT));
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
    if (rafId) cancelAnimationFrame(rafId);
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", handleMouseUp);
  }

  document.addEventListener("mousemove", handleMouseMove);
  document.addEventListener("mouseup", handleMouseUp);
}
</script>

<template>
  <div
    class="fixed"
    :class="windowClass"
    :style="windowStyle"
    :data-resize-dir="isResizing ? resizeDirection : ''"
    @mousedown="emit('focus')"
  >
    <header v-if="!customChrome" :class="headerClass" @mousedown="startDrag">
      <!-- Windows 风格标题 (居左) -->
      <div class="flex items-center gap-2">
        <div class="text-[12px] text-foreground/80 tracking-wide font-medium">
          {{ title }}
        </div>
        <!-- 在线状态指示 -->
        <div v-if="statusText" class="flex items-center gap-1.5 ml-2">
          <span class="text-[11px] text-muted-foreground">
            {{ statusText }}
          </span>
        </div>
      </div>

      <!-- 控制按钮 (紧贴右上角) -->
      <div class="flex items-center h-full">
        <TrafficLights
          :is-maximized="isMaximized"
          :is-fullscreen="isFullscreen"
          @minimize="handleMinimize"
          @maximize="handleMaximize"
          @close="handleClose"
        />
      </div>
    </header>

    <div
      class="overflow-auto flex flex-col"
      :class="[
        customChrome ? 'h-full' : 'h-[calc(100%-36px)]',
        { 'pointer-events-none': isResizing || isDragging },
        { 'rounded-none h-full': isMaximized || isFullscreen },
        { 'rounded-b-lg': !isMaximized && !isFullscreen },
      ]"
    >
      <slot
        :start-drag="startDrag"
        :close="handleClose"
        :minimize="handleMinimize"
        :maximize="handleMaximize"
        :is-maximized="isMaximized"
        :is-fullscreen="isFullscreen"
      />
    </div>

    <template v-if="resizeHandlesVisible">
      <div
        class="absolute z-[99999] pointer-events-auto left-4 right-4 -top-2 h-4 cursor-ns-resize"
        @mouseenter="setResizeCursor('n')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'n')"
      ></div>
      <div
        class="absolute z-[99999] pointer-events-auto left-4 right-4 -bottom-2 h-4 cursor-ns-resize"
        @mouseenter="setResizeCursor('s')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 's')"
      ></div>
      <div
        class="absolute z-[99999] pointer-events-auto top-4 bottom-4 -right-2 w-4 cursor-ew-resize"
        @mouseenter="setResizeCursor('e')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'e')"
      ></div>
      <div
        class="absolute z-[99999] pointer-events-auto top-4 bottom-4 -left-2 w-4 cursor-ew-resize"
        @mouseenter="setResizeCursor('w')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'w')"
      ></div>

      <div
        class="absolute z-[99999] pointer-events-auto -top-2 -right-2 w-8 h-8 cursor-nesw-resize group flex items-start justify-end p-1"
        @mouseenter="setResizeCursor('ne')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'ne')"
      >
        <span
          class="iconify mdi--resize-bottom-right rotate-180 text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"
        ></span>
      </div>
      <div
        class="absolute z-[99999] pointer-events-auto -top-2 -left-2 w-8 h-8 cursor-nwse-resize group flex items-start justify-start p-1"
        @mouseenter="setResizeCursor('nw')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'nw')"
      >
        <span
          class="iconify mdi--resize-bottom-right rotate-[270deg] text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"
        ></span>
      </div>
      <div
        class="absolute z-[99999] pointer-events-auto -bottom-2 -right-2 w-8 h-8 cursor-nwse-resize group flex items-end justify-end p-1"
        @mouseenter="setResizeCursor('se')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'se')"
      >
        <span
          class="iconify mdi--resize-bottom-right text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"
        ></span>
      </div>
      <div
        class="absolute z-[99999] pointer-events-auto -bottom-2 -left-2 w-8 h-8 cursor-nesw-resize group flex items-end justify-start p-1"
        @mouseenter="setResizeCursor('sw')"
        @mouseleave="handleMouseLeave"
        @mousedown="startResize($event, 'sw')"
      >
        <span
          class="iconify mdi--resize-bottom-right rotate-90 text-muted-foreground/30 opacity-0 group-hover:opacity-100 transition-opacity"
        ></span>
      </div>
    </template>
  </div>
</template>

<style scoped></style>

<style>
body.cursor-n-resize,
body.cursor-s-resize {
  cursor:
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M16 4l-6 8h4v8h-4l6 8 6-8h-4v-8h4z"/></svg>')
      16 16,
    ns-resize !important;
}

body.cursor-e-resize,
body.cursor-w-resize {
  cursor:
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M28 16l-8-6v4h-8v-4l-8 6 8 6v-4h8v4z"/></svg>')
      16 16,
    ew-resize !important;
}

body.cursor-ne-resize,
body.cursor-sw-resize {
  cursor:
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M22.5 9.5l-8.5 2.5 1.5 1.5-6 6 1.5 1.5 6-6 1.5 1.5 2.5-8.5z"/></svg>')
      16 16,
    nesw-resize !important;
}

body.cursor-nw-resize,
body.cursor-se-resize {
  cursor:
    url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 32 32"><path fill="black" stroke="white" stroke-width="1.5" d="M9.5 9.5l8.5 2.5-1.5 1.5 6 6-1.5 1.5-6-6-1.5 1.5-2.5-8.5z"/></svg>')
      16 16,
    nwse-resize !important;
}
</style>
