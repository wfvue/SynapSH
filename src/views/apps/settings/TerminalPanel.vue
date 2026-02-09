<!--
  TerminalPanel.vue - 终端设置面板
  设置终端字体、配色方案、光标样式等
-->
<script setup lang="ts">
import { ref } from "vue";

// 终端设置项
const fontSize = ref(14);
const fontFamily = ref("JetBrains Mono");
const cursorStyle = ref<"block" | "underline" | "bar">("block");
const cursorBlink = ref(true);
const colorScheme = ref("dracula");
const scrollback = ref(1000);

const fonts = [
    { value: "JetBrains Mono", label: "JetBrains Mono" },
    { value: "Fira Code", label: "Fira Code" },
    { value: "Source Code Pro", label: "Source Code Pro" },
    { value: "Monaco", label: "Monaco" },
    { value: "Consolas", label: "Consolas" },
];

const colorSchemes = [
    { value: "dracula", label: "Dracula", bg: "#282a36", fg: "#f8f8f2", accent: "#bd93f9" },
    { value: "one-dark", label: "One Dark", bg: "#282c34", fg: "#abb2bf", accent: "#61afef" },
    { value: "monokai", label: "Monokai", bg: "#272822", fg: "#f8f8f2", accent: "#a6e22e" },
    { value: "nord", label: "Nord", bg: "#2e3440", fg: "#d8dee9", accent: "#88c0d0" },
    { value: "solarized", label: "Solarized", bg: "#002b36", fg: "#839496", accent: "#268bd2" },
];
</script>

<template>
    <div class="settings-panel">
        <h2 class="panel-title">终端</h2>
        <p class="panel-subtitle">自定义终端外观和行为</p>

        <section class="settings-section">
            <h3 class="section-title">字体</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">字体</span>
                </div>
                <select v-model="fontFamily" class="setting-select">
                    <option v-for="font in fonts" :key="font.value" :value="font.value">
                        {{ font.label }}
                    </option>
                </select>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">字体大小</span>
                </div>
                <div class="number-input">
                    <button @click="fontSize = Math.max(10, fontSize - 1)" class="num-btn">−</button>
                    <span class="num-value">{{ fontSize }}px</span>
                    <button @click="fontSize = Math.min(24, fontSize + 1)" class="num-btn">+</button>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">光标</h3>

            <div class="cursor-options">
                <div class="cursor-option" :class="{ active: cursorStyle === 'block' }" @click="cursorStyle = 'block'">
                    <div class="cursor-preview block-cursor"></div>
                    <span>方块</span>
                </div>
                <div class="cursor-option" :class="{ active: cursorStyle === 'underline' }"
                    @click="cursorStyle = 'underline'">
                    <div class="cursor-preview underline-cursor"></div>
                    <span>下划线</span>
                </div>
                <div class="cursor-option" :class="{ active: cursorStyle === 'bar' }" @click="cursorStyle = 'bar'">
                    <div class="cursor-preview bar-cursor"></div>
                    <span>竖线</span>
                </div>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">光标闪烁</span>
                </div>
                <label class="toggle">
                    <input type="checkbox" v-model="cursorBlink" />
                    <span class="toggle-slider"></span>
                </label>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">配色方案</h3>

            <div class="color-schemes">
                <div v-for="scheme in colorSchemes" :key="scheme.value" class="scheme-item"
                    :class="{ active: colorScheme === scheme.value }" @click="colorScheme = scheme.value">
                    <div class="scheme-preview" :style="{ background: scheme.bg }">
                        <span :style="{ color: scheme.fg }">$ echo</span>
                        <span :style="{ color: scheme.accent }"> "Hello"</span>
                    </div>
                    <span class="scheme-name">{{ scheme.label }}</span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">高级</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">回滚行数</span>
                    <span class="setting-desc">终端保留的历史行数</span>
                </div>
                <select v-model="scrollback" class="setting-select">
                    <option :value="500">500 行</option>
                    <option :value="1000">1000 行</option>
                    <option :value="5000">5000 行</option>
                    <option :value="10000">10000 行</option>
                </select>
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

.setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    margin-bottom: 8px;
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

/* Number Input */
.number-input {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    padding: 4px;
}

.num-btn {
    width: 28px;
    height: 28px;
    border: none;
    background: rgba(255, 255, 255, 0.1);
    color: white;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    transition: background 0.2s;
}

.num-btn:hover {
    background: rgba(255, 255, 255, 0.2);
}

.num-value {
    min-width: 50px;
    text-align: center;
    font-size: 0.85rem;
}

/* Cursor Options */
.cursor-options {
    display: flex;
    gap: 12px;
    margin-bottom: 12px;
}

.cursor-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    background: rgba(255, 255, 255, 0.04);
    border: 2px solid transparent;
    border-radius: 10px;
    cursor: pointer;
    transition: all 0.2s;
}

.cursor-option:hover {
    background: rgba(255, 255, 255, 0.08);
}

.cursor-option.active {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
}

.cursor-preview {
    width: 40px;
    height: 24px;
    background: #1e1e2e;
    border-radius: 4px;
    position: relative;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    padding: 4px;
}

.block-cursor::after {
    content: "";
    width: 10px;
    height: 16px;
    background: #f0f0f0;
}

.underline-cursor::after {
    content: "";
    width: 10px;
    height: 2px;
    background: #f0f0f0;
}

.bar-cursor::after {
    content: "";
    width: 2px;
    height: 16px;
    background: #f0f0f0;
}

.cursor-option span {
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.7);
}

/* Color Schemes */
.color-schemes {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
}

.scheme-item {
    display: flex;
    flex-direction: column;
    gap: 6px;
    cursor: pointer;
    padding: 8px;
    border: 2px solid transparent;
    border-radius: 10px;
    transition: all 0.2s;
}

.scheme-item:hover {
    background: rgba(255, 255, 255, 0.04);
}

.scheme-item.active {
    border-color: #3b82f6;
}

.scheme-preview {
    padding: 10px 12px;
    border-radius: 6px;
    font-family: "JetBrains Mono", monospace;
    font-size: 0.75rem;
}

.scheme-name {
    font-size: 0.8rem;
    text-align: center;
    color: rgba(255, 255, 255, 0.7);
}

/* Toggle */
.toggle {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
    cursor: pointer;
}

.toggle input {
    opacity: 0;
    width: 0;
    height: 0;
}

.toggle-slider {
    position: absolute;
    inset: 0;
    background: rgba(255, 255, 255, 0.15);
    border-radius: 24px;
    transition: all 0.3s;
}

.toggle-slider::before {
    content: "";
    position: absolute;
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background: white;
    border-radius: 50%;
    transition: all 0.3s;
}

.toggle input:checked+.toggle-slider {
    background: #22c55e;
}

.toggle input:checked+.toggle-slider::before {
    transform: translateX(20px);
}

/* Select */
.setting-select {
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: rgba(255, 255, 255, 0.9);
    font-size: 0.85rem;
    cursor: pointer;
    min-width: 140px;
}

.setting-select:focus {
    outline: none;
    border-color: rgba(59, 130, 246, 0.5);
}

.setting-select option {
    background: #1e1e2e;
    color: white;
}
</style>
