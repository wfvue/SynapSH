<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  sessionId: string;
}>();

// 浏览器状态
const currentUrl = ref('');
const inputUrl = ref('');
const isLoading = ref(false);
const hasError = ref(false);
const showWelcome = ref(true);
const profileMode = ref<'session' | 'new'>('session');

const homeUrl = 'https://www.baidu.com';

function normalizeUrl(url: string) {
  if (!url) return '';
  if (url.startsWith('http://') || url.startsWith('https://')) return url;
  return `https://${url}`;
}

async function loadUrl(url: string) {
  if (!url) return;
  const formattedUrl = normalizeUrl(url);
  currentUrl.value = formattedUrl;
  inputUrl.value = formattedUrl;
  isLoading.value = true;
  hasError.value = false;
  showWelcome.value = false;

  try {
    await invoke('browser_open', {
      sessionId: props.sessionId,
      url: formattedUrl,
      options: { profileMode: profileMode.value },
    });
  } catch (error) {
    console.error('打开浏览器失败:', error);
    hasError.value = true;
  } finally {
    setTimeout(() => {
      isLoading.value = false;
    }, 400);
  }
}

function goHome() {
  loadUrl(homeUrl);
}

function handleUrlSubmit() {
  if (inputUrl.value) {
    loadUrl(inputUrl.value);
  }
}

async function reopenWindow() {
  const target = currentUrl.value || inputUrl.value || homeUrl;
  await loadUrl(target);
}

// 常用书签
const bookmarks = ref([
  { name: '百度', url: 'https://www.baidu.com' },
  { name: 'GitHub', url: 'https://github.com' },
  { name: 'Google', url: 'https://www.google.com' },
  { name: 'Bilibili', url: 'https://www.bilibili.com' },
]);
</script>

<template>
  <div class="browser-app">
    <!-- 工具栏 -->
    <div class="toolbar">
      <!-- 导航按钮 -->
      <div class="nav-buttons">
        <button 
          class="nav-btn" 
          disabled
          title="请在 Chrome 中操作后退"
        >
          <span class="i-mdi-arrow-left"></span>
        </button>
        <button 
          class="nav-btn" 
          disabled
          title="请在 Chrome 中操作前进"
        >
          <span class="i-mdi-arrow-right"></span>
        </button>
        <button 
          class="nav-btn" 
          disabled
          title="请在 Chrome 中操作刷新"
        >
          <span class="i-mdi-refresh"></span>
        </button>
        <button 
          class="nav-btn" 
          @click="goHome"
          title="主页"
        >
          <span class="i-mdi-home"></span>
        </button>
      </div>

      <!-- 地址栏 -->
      <form class="url-bar" @submit.prevent="handleUrlSubmit">
        <span class="i-mdi-earth url-icon"></span>
        <input 
          v-model="inputUrl" 
          type="text" 
          placeholder="输入网址或搜索内容..."
          class="url-input"
        />
        <button type="submit" class="go-btn" title="前往">
          <span class="i-mdi-arrow-right"></span>
        </button>
      </form>

      <!-- 外部打开按钮 -->
      <button 
        class="nav-btn" 
        @click="reopenWindow"
        title="使用 Chrome 打开"
      >
        <span class="i-mdi-open-in-new"></span>
      </button>

      <!-- Profile 选择 -->
      <div class="profile-toggle" title="选择 Chrome Profile 策略">
        <button
          class="profile-btn"
          :class="{ active: profileMode === 'session' }"
          @click="profileMode = 'session'"
        >
          复用
        </button>
        <button
          class="profile-btn"
          :class="{ active: profileMode === 'new' }"
          @click="profileMode = 'new'"
        >
          新建
        </button>
      </div>
    </div>

    <!-- 书签栏 -->
    <div class="bookmarks-bar">
      <button 
        v-for="bookmark in bookmarks" 
        :key="bookmark.url"
        class="bookmark-btn"
        @click="loadUrl(bookmark.url)"
      >
        <span class="i-mdi-star bookmark-icon"></span>
        {{ bookmark.name }}
      </button>
    </div>

    <!-- 加载指示器 -->
    <div v-if="isLoading" class="loading-bar">
      <div class="loading-progress"></div>
    </div>

    <!-- 浏览器内容区 -->
    <div class="browser-content">
      <!-- 欢迎页面 -->
      <div v-if="showWelcome" class="welcome-page">
        <div class="welcome-content">
          <div class="welcome-icon">
            <span class="i-mdi-compass" style="font-size: 64px;"></span>
          </div>
          <h1 class="welcome-title">欢迎使用浏览器</h1>
          <p class="welcome-subtitle">在上方地址栏输入网址开始浏览</p>
          
          <div class="quick-links">
            <h3>快速访问</h3>
            <div class="link-grid">
              <button class="quick-link" @click="loadUrl('https://www.baidu.com')">
                <span class="i-mdi-google"></span>
                百度
              </button>
              <button class="quick-link" @click="loadUrl('https://github.com')">
                <span class="i-mdi-github"></span>
                GitHub
              </button>
              <button class="quick-link" @click="loadUrl('https://www.bilibili.com')">
                <span class="i-mdi-youtube"></span>
                Bilibili
              </button>
            </div>
          </div>
          
          <div class="tips">
            <p>💡 提示：页面会通过 SSH 代理在独立窗口中打开，可使用右上角按钮重新打开。</p>
          </div>
        </div>
      </div>
      
      <!-- 独立窗口提示 -->
      <div v-else class="browser-remote">
        <div class="remote-card">
          <span class="i-mdi-google-chrome remote-icon"></span>
          <h2>Chrome 已准备</h2>
          <p>页面将通过 SSH 代理在 Chrome 中打开。</p>
          <p class="remote-url">{{ currentUrl || '尚未打开页面' }}</p>
          <div class="remote-meta">
            <span>Profile: {{ profileMode === 'new' ? '每次新建' : '复用' }}</span>
          </div>
          <button class="remote-btn" @click="reopenWindow">
            打开 Chrome
          </button>
        </div>
      </div>
    </div>

    <!-- 提示信息 -->
    <div class="status-bar">
      <span class="url-status">{{ currentUrl }}</span>
      <span class="i-mdi-information-outline info-icon" title="页面通过 SSH 代理在 Chrome 中加载"></span>
    </div>
  </div>
</template>

<style scoped>
.browser-app {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a1a;
}

/* 工具栏 */
.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: #2d2d2d;
  border-bottom: 1px solid #3d3d3d;
}

.nav-buttons {
  display: flex;
  gap: 4px;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #a0a0a0;
  cursor: pointer;
  transition: all 0.2s;
}

.nav-btn:hover:not(:disabled) {
  background: #3d3d3d;
  color: #e0e0e0;
}

.nav-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.nav-btn span {
  font-size: 18px;
}

.profile-toggle {
  display: inline-flex;
  gap: 4px;
  padding: 2px;
  background: #1e1e1e;
  border: 1px solid #3d3d3d;
  border-radius: 8px;
}

.profile-btn {
  border: none;
  background: transparent;
  color: #8b8b8b;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.profile-btn.active {
  background: #3d3d3d;
  color: #e0e0e0;
}

.profile-btn:hover {
  color: #e0e0e0;
}

/* 地址栏 */
.url-bar {
  flex: 1;
  display: flex;
  align-items: center;
  background: #1e1e1e;
  border: 1px solid #3d3d3d;
  border-radius: 8px;
  padding: 0 4px;
  height: 36px;
  transition: border-color 0.2s;
}

.url-bar:focus-within {
  border-color: #4a9eff;
}

.url-icon {
  color: #666;
  font-size: 18px;
  padding: 0 8px;
}

.url-input {
  flex: 1;
  border: none;
  background: transparent;
  color: #e0e0e0;
  font-size: 14px;
  outline: none;
  height: 100%;
}

.url-input::placeholder {
  color: #666;
}

.go-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: #4a9eff;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
}

.go-btn:hover {
  background: #3a8eef;
}

.go-btn span {
  font-size: 16px;
}

/* 书签栏 */
.bookmarks-bar {
  display: flex;
  gap: 4px;
  padding: 6px 12px;
  background: #252525;
  border-bottom: 1px solid #3d3d3d;
  overflow-x: auto;
}

.bookmark-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: #a0a0a0;
  font-size: 13px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s;
}

.bookmark-btn:hover {
  background: #3d3d3d;
  color: #e0e0e0;
}

.bookmark-icon {
  font-size: 14px;
}

/* 加载条 */
.loading-bar {
  position: relative;
  height: 2px;
  background: transparent;
}

.loading-progress {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: 100%;
  background: linear-gradient(90deg, #4a9eff 0%, #7cb3ff 50%, #4a9eff 100%);
  background-size: 200% 100%;
  animation: loading 1s linear infinite;
}

@keyframes loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* 浏览器内容区 */
.browser-content {
  flex: 1;
  overflow: hidden;
  background: #000;
}

.browser-remote {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: radial-gradient(circle at top, #1e293b 0%, #0b1120 60%, #000 100%);
}

.remote-card {
  width: min(520px, 90%);
  text-align: center;
  padding: 32px;
  border-radius: 16px;
  background: rgba(20, 24, 38, 0.85);
  border: 1px solid #2c344a;
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.45);
  color: #e0e0e0;
}

.remote-icon {
  font-size: 48px;
  color: #4a9eff;
  display: inline-block;
  margin-bottom: 16px;
}

.remote-card h2 {
  font-size: 22px;
  margin-bottom: 8px;
}

.remote-card p {
  margin: 6px 0;
  color: #a0a0a0;
}

.remote-meta {
  margin-top: 10px;
  font-size: 12px;
  color: #7cb3ff;
}

.remote-url {
  font-size: 12px;
  color: #7cb3ff;
  word-break: break-all;
  margin-top: 12px;
}

.remote-btn {
  margin-top: 18px;
  padding: 10px 18px;
  border: none;
  border-radius: 8px;
  background: #4a9eff;
  color: white;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.remote-btn:hover {
  background: #3a8eef;
}

/* 欢迎页面 */
.welcome-page {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
}

.welcome-content {
  text-align: center;
  color: #e0e0e0;
  padding: 40px;
}

.welcome-icon {
  margin-bottom: 24px;
  color: #4a9eff;
}

.welcome-title {
  font-size: 32px;
  font-weight: 600;
  margin-bottom: 12px;
  color: #e0e0e0;
}

.welcome-subtitle {
  font-size: 16px;
  color: #a0a0a0;
  margin-bottom: 40px;
}

.quick-links {
  margin-bottom: 40px;
}

.quick-links h3 {
  font-size: 18px;
  margin-bottom: 20px;
  color: #e0e0e0;
}

.link-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  max-width: 400px;
  margin: 0 auto;
}

.quick-link {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px;
  border: 1px solid #3d3d3d;
  border-radius: 12px;
  background: #252525;
  color: #e0e0e0;
  cursor: pointer;
  transition: all 0.2s;
}

.quick-link:hover {
  background: #3d3d3d;
  border-color: #4a9eff;
  transform: translateY(-2px);
}

.quick-link span {
  font-size: 32px;
}

.quick-link::after {
  content: attr(data-label);
  font-size: 14px;
  margin-top: 8px;
}

.tips {
  max-width: 500px;
  margin: 0 auto;
  padding: 16px;
  background: #252525;
  border-radius: 8px;
  border: 1px solid #3d3d3d;
}

.tips p {
  font-size: 14px;
  color: #a0a0a0;
  margin: 0;
  line-height: 1.6;
}

/* 状态栏 */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 12px;
  background: #2d2d2d;
  border-top: 1px solid #3d3d3d;
  font-size: 12px;
}

.url-status {
  color: #888;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-icon {
  color: #666;
  cursor: help;
  font-size: 16px;
}
</style>
