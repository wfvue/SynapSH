<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
let unlistenFn: UnlistenFn | null = null;

// base64 解码
function base64Decode(base64: string): string {
  try {
    return atob(base64);
  } catch (e) {
    console.error("Base64 decode error:", e);
    return "";
  }
}

onMounted(async () => {
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

  // 监听 SSH 数据事件
  const eventName = `ssh-data-${props.sessionId}`;
  console.log("Listening for event:", eventName);

  unlistenFn = await listen<string>(eventName, (event) => {
    console.log("Received SSH data, length:", event.payload.length);
    const decoded = base64Decode(event.payload);
    if (decoded && terminal.value) {
      terminal.value.write(decoded);
    }
  });

  // 延迟发送回车，刷新 shell 提示符（shell 欢迎信息可能在监听器启动前发送）
  setTimeout(() => {
    if (props.sessionId && props.sessionId !== "default-session") {
      invoke("write_to_pty", {
        sessionId: props.sessionId,
        data: "\n",
      }).catch(console.error);
    }
  }, 500);

  // 监听窗口大小变化
  const resizeObserver = new ResizeObserver(() => {
    fitAddon.value?.fit();
  });
  resizeObserver.observe(terminalRef.value);

  // 清理函数
  onUnmounted(() => {
    resizeObserver.disconnect();
    if (unlistenFn) {
      unlistenFn();
    }
    terminal.value?.dispose();
  });
});

// 监听 sessionId 变化
watch(
  () => props.sessionId,
  async (newId, oldId) => {
    if (oldId && newId !== oldId) {
      // 取消旧的事件监听
      if (unlistenFn) {
        unlistenFn();
        unlistenFn = null;
      }
      // 清空终端
      terminal.value?.clear();

      // 监听新的事件
      const eventName = `ssh-data-${newId}`;
      console.log("Switching to new event:", eventName);
      unlistenFn = await listen<string>(eventName, (event) => {
        console.log("Received SSH data, length:", event.payload.length);
        const decoded = base64Decode(event.payload);
        if (decoded && terminal.value) {
          terminal.value.write(decoded);
        }
      });
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

:deep(.xterm-viewport::-webkit-scrollbar) {
  width: 8px;
}

:deep(.xterm-viewport::-webkit-scrollbar-track) {
  background: transparent;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb) {
  background-color: rgba(255, 255, 255, 0.15);
  border-radius: 4px;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb:hover) {
  background-color: rgba(255, 255, 255, 0.25);
}
</style>
