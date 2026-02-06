<script setup lang="ts">
import { ref } from "vue";
import Terminal from "./components/Terminal.vue";
import ConnectionPanel from "./components/ConnectionPanel.vue";

const isConnected = ref(false);
const sessionId = ref("");

function onConnected(id: string) {
  sessionId.value = id;
  isConnected.value = true;
}

function onDisconnected() {
  isConnected.value = false;
  sessionId.value = "";
}
</script>

<template>
  <main class="app-container">
    <aside class="sidebar">
      <ConnectionPanel 
        :is-connected="isConnected"
        @connected="onConnected"
        @disconnected="onDisconnected"
      />
    </aside>
    <section class="terminal-area">
      <Terminal 
        v-if="isConnected" 
        :session-id="sessionId"
      />
      <div v-else class="welcome">
        <h1>光析</h1>
        <p>基于 Tauri 的现代化 SSH 客户端</p>
        <p class="hint">请在左侧配置并连接 SSH 服务器</p>
      </div>
    </section>
  </main>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

:root {
  --bg-primary: #1e1e1e;
  --bg-secondary: #252526;
  --bg-tertiary: #2d2d30;
  --text-primary: #cccccc;
  --text-secondary: #858585;
  --accent: #007acc;
  --border: #3e3e42;
  --success: #4ec9b0;
  --error: #f48771;
}

body {
  font-family: 'Segoe UI', 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  overflow: hidden;
}
</style>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  width: 100vw;
}

.sidebar {
  width: 300px;
  min-width: 300px;
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
}

.terminal-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.welcome {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}

.welcome h1 {
  font-size: 3rem;
  font-weight: 300;
  margin-bottom: 1rem;
  background: linear-gradient(135deg, #007acc 0%, #4ec9b0 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.welcome p {
  font-size: 1.2rem;
  margin-bottom: 0.5rem;
}

.welcome .hint {
  font-size: 0.9rem;
  color: var(--text-secondary);
  margin-top: 2rem;
}
</style>
