<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebglAddon } from "@xterm/addon-webgl";
import "@xterm/xterm/css/xterm.css";

const props = defineProps<{
  sessionId: string;
}>();

const terminalRef = ref<HTMLElement>();
const terminal = ref<XTerm | null>(null);
const fitAddon = ref<FitAddon | null>(null);

onMounted(() => {
  if (!terminalRef.value) return;

  // 初始化 xterm.js
  terminal.value = new XTerm({
    fontFamily: 'Consolas, "Courier New", monospace',
    fontSize: 14,
    cursorBlink: true,
    cursorStyle: "block",
    theme: {
      background: "#1e1e1e",
      foreground: "#cccccc",
      cursor: "#cccccc",
      selectionBackground: "#264f78",
      black: "#000000",
      red: "#cd3131",
      green: "#0dbc79",
      yellow: "#e5e510",
      blue: "#2472c8",
      magenta: "#bc3fbc",
      cyan: "#11a8cd",
      white: "#e5e5e5",
      brightBlack: "#666666",
      brightRed: "#f14c4c",
      brightGreen: "#23d18b",
      brightYellow: "#f5f543",
      brightBlue: "#3b8eea",
      brightMagenta: "#d670d6",
      brightCyan: "#29b8db",
      brightWhite: "#e5e5e5",
    },
    scrollback: 10000,
    allowProposedApi: true,
  });

  // 加载插件
  fitAddon.value = new FitAddon();
  terminal.value.loadAddon(fitAddon.value);

  // 尝试加载 WebGL 渲染器
  try {
    const webglAddon = new WebglAddon();
    terminal.value.loadAddon(webglAddon);
  } catch (e) {
    console.warn("WebGL addon failed to load, falling back to canvas", e);
  }

  // 挂载到 DOM
  terminal.value.open(terminalRef.value);
  fitAddon.value.fit();

  // 处理输入
  terminal.value.onData((data) => {
    if (props.sessionId) {
      invoke("write_to_pty", {
        sessionId: props.sessionId,
        data,
      }).catch(console.error);
    }
  });

  // 处理终端大小变化
  terminal.value.onResize(({ cols, rows }) => {
    if (props.sessionId) {
      invoke("resize_pty", {
        sessionId: props.sessionId,
        cols,
        rows,
      }).catch(console.error);
    }
  });

  // 监听窗口大小变化
  const resizeObserver = new ResizeObserver(() => {
    fitAddon.value?.fit();
  });
  resizeObserver.observe(terminalRef.value);

  // 清理函数
  onUnmounted(() => {
    resizeObserver.disconnect();
    terminal.value?.dispose();
  });
});

// 监听 sessionId 变化
watch(
  () => props.sessionId,
  (newId, oldId) => {
    if (oldId && newId !== oldId) {
      // 会话变化时清空终端
      terminal.value?.clear();
    }
  }
);
</script>

<template>
  <div class="terminal-container">
    <div ref="terminalRef" class="terminal"></div>
  </div>
</template>

<style scoped>
.terminal-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: #1e1e1e;
  padding: 8px;
}

.terminal {
  flex: 1;
  width: 100%;
  height: 100%;
}

:deep(.xterm) {
  height: 100%;
}

:deep(.xterm-viewport) {
  background-color: #1e1e1e !important;
}
</style>
