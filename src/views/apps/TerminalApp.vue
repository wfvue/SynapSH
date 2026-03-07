<!--
  TerminalApp.vue - 终端应用组件
  基于 xterm.js 的 SSH 终端，支持 WebGL 渲染
-->
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

// base64 解码为 Uint8Array，正确处理 UTF-8 多字节字符（如中文）
function base64Decode(base64: string): Uint8Array | null {
  try {
    const binaryStr = atob(base64);
    const bytes = new Uint8Array(binaryStr.length);
    for (let i = 0; i < binaryStr.length; i++) {
      bytes[i] = binaryStr.charCodeAt(i);
    }
    return bytes;
  } catch (e) {
    console.error("Base64 decode error:", e);
    return null;
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
  <div class="flex-1 flex flex-col overflow-hidden bg-[#1e1e1e]">
    <div ref="terminalRef" class="flex-1 w-full h-full"></div>
  </div>
</template>

<style scoped>
@reference "../../style.css";

:deep(.xterm) {
  @apply h-full;
}

:deep(.xterm-viewport) {
  @apply !bg-[#1e1e1e];
}

:deep(.xterm-viewport::-webkit-scrollbar) {
  @apply w-2;
}

:deep(.xterm-viewport::-webkit-scrollbar-track) {
  @apply bg-transparent;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb) {
  @apply bg-white/15 rounded;
}

:deep(.xterm-viewport::-webkit-scrollbar-thumb:hover) {
  @apply bg-white/25;
}
</style>
