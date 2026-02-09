<!--
  AppearancePanel.vue - 外观设置面板
  设置主题、壁纸、强调色等外观配置
-->
<script setup lang="ts">
import { ref } from "vue";

// 外观设置项
const theme = ref<"dark" | "light" | "auto">("dark");
const accentColor = ref("#3b82f6");
const wallpaper = ref(0);
const transparency = ref(80);

const accentColors = [
    { value: "#3b82f6", name: "蓝色" },
    { value: "#8b5cf6", name: "紫色" },
    { value: "#ec4899", name: "粉色" },
    { value: "#ef4444", name: "红色" },
    { value: "#f97316", name: "橙色" },
    { value: "#22c55e", name: "绿色" },
    { value: "#06b6d4", name: "青色" },
];

const wallpapers = [
    { id: 0, name: "默认渐变", preview: "linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)" },
    { id: 1, name: "深空蓝", preview: "linear-gradient(135deg, #0c1445 0%, #1a237e 100%)" },
    { id: 2, name: "暗夜紫", preview: "linear-gradient(135deg, #1a0533 0%, #4a1259 100%)" },
    { id: 3, name: "极光绿", preview: "linear-gradient(135deg, #0d2818 0%, #1b4332 100%)" },
];
</script>

<template>
    <div class="settings-panel">
        <h2 class="panel-title">外观</h2>
        <p class="panel-subtitle">个性化您的桌面外观</p>

        <section class="settings-section">
            <h3 class="section-title">主题</h3>

            <div class="theme-options">
                <div class="theme-option" :class="{ active: theme === 'light' }" @click="theme = 'light'">
                    <div class="theme-preview light-preview">
                        <span class="icon-[mdi--white-balance-sunny]"></span>
                    </div>
                    <span class="theme-label">浅色</span>
                </div>
                <div class="theme-option" :class="{ active: theme === 'dark' }" @click="theme = 'dark'">
                    <div class="theme-preview dark-preview">
                        <span class="icon-[mdi--moon-waning-crescent]"></span>
                    </div>
                    <span class="theme-label">深色</span>
                </div>
                <div class="theme-option" :class="{ active: theme === 'auto' }" @click="theme = 'auto'">
                    <div class="theme-preview auto-preview">
                        <span class="icon-[mdi--theme-light-dark]"></span>
                    </div>
                    <span class="theme-label">自动</span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">强调色</h3>

            <div class="accent-colors">
                <div v-for="color in accentColors" :key="color.value" class="accent-color"
                    :class="{ active: accentColor === color.value }" :style="{ background: color.value }"
                    :title="color.name" @click="accentColor = color.value">
                    <span v-if="accentColor === color.value" class="icon-[mdi--check]"></span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">壁纸</h3>

            <div class="wallpaper-grid">
                <div v-for="wp in wallpapers" :key="wp.id" class="wallpaper-item"
                    :class="{ active: wallpaper === wp.id }" :style="{ background: wp.preview }"
                    @click="wallpaper = wp.id">
                    <span class="wallpaper-name">{{ wp.name }}</span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">透明度</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">窗口透明度</span>
                    <span class="setting-desc">调整窗口背景的透明程度</span>
                </div>
                <div class="slider-container">
                    <input type="range" v-model="transparency" min="20" max="100" class="slider" />
                    <span class="slider-value">{{ transparency }}%</span>
                </div>
            </div>
        </section>
    </div>
</template>

<style scoped>
.settings-panel {
    padding: 24px;
    color: rgba(255, 255, 255, 0.9);
}

.panel-title {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0 0 4px 0;
    color: #fff;
}

.panel-subtitle {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.5);
    margin: 0 0 24px 0;
}

.settings-section {
    margin-bottom: 28px;
}

.section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: rgba(255, 255, 255, 0.4);
    margin: 0 0 12px 0;
}

/* Theme Options */
.theme-options {
    display: flex;
    gap: 12px;
}

.theme-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    background: rgba(255, 255, 255, 0.04);
    border: 2px solid transparent;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
}

.theme-option:hover {
    background: rgba(255, 255, 255, 0.08);
}

.theme-option.active {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
}

.theme-preview {
    width: 60px;
    height: 44px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    font-size: 24px;
}

.light-preview {
    background: linear-gradient(135deg, #f0f0f0 0%, #e0e0e0 100%);
    color: #f59e0b;
}

.dark-preview {
    background: linear-gradient(135deg, #1e1e2e 0%, #2d2d3d 100%);
    color: #a78bfa;
}

.auto-preview {
    background: linear-gradient(135deg, #f0f0f0 0%, #2d2d3d 100%);
    color: #fff;
}

.theme-label {
    font-size: 0.85rem;
    font-weight: 500;
}

/* Accent Colors */
.accent-colors {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.accent-color {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    cursor: pointer;
    display: grid;
    place-items: center;
    color: white;
    font-size: 16px;
    border: 3px solid transparent;
    transition: all 0.2s;
}

.accent-color:hover {
    transform: scale(1.1);
}

.accent-color.active {
    border-color: rgba(255, 255, 255, 0.5);
    box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.2);
}

/* Wallpaper Grid */
.wallpaper-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
}

.wallpaper-item {
    height: 80px;
    border-radius: 10px;
    cursor: pointer;
    display: flex;
    align-items: flex-end;
    padding: 8px 12px;
    border: 2px solid transparent;
    transition: all 0.2s;
}

.wallpaper-item:hover {
    transform: scale(1.02);
}

.wallpaper-item.active {
    border-color: #3b82f6;
}

.wallpaper-name {
    font-size: 0.75rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

/* Setting Item */
.setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
}

.setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.setting-label {
    font-size: 0.9rem;
    font-weight: 500;
}

.setting-desc {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
}

/* Slider */
.slider-container {
    display: flex;
    align-items: center;
    gap: 12px;
}

.slider {
    width: 120px;
    height: 4px;
    -webkit-appearance: none;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 4px;
    outline: none;
}

.slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 16px;
    height: 16px;
    background: #3b82f6;
    border-radius: 50%;
    cursor: pointer;
    transition: transform 0.2s;
}

.slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
}

.slider-value {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.6);
    min-width: 40px;
    text-align: right;
}
</style>
