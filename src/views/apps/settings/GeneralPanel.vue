<!--
  GeneralPanel.vue - 通用设置面板
  设置启动行为、默认应用等通用配置
-->
<script setup lang="ts">
import { ref } from "vue";

// 通用设置项
const autoConnect = ref(true);
const showWelcome = ref(true);
const language = ref("zh-CN");
const defaultShell = ref("bash");

const languages = [
    { value: "zh-CN", label: "简体中文" },
    { value: "en-US", label: "English" },
];

const shells = [
    { value: "bash", label: "Bash" },
    { value: "zsh", label: "Zsh" },
    { value: "sh", label: "Sh" },
];
</script>

<template>
    <div class="settings-panel">
        <h2 class="panel-title">通用</h2>
        <p class="panel-subtitle">配置基本行为和偏好设置</p>

        <section class="settings-section">
            <h3 class="section-title">启动</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">启动时自动连接</span>
                    <span class="setting-desc">启动应用后自动连接上次使用的服务器</span>
                </div>
                <label class="toggle">
                    <input type="checkbox" v-model="autoConnect" />
                    <span class="toggle-slider"></span>
                </label>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">显示欢迎界面</span>
                    <span class="setting-desc">启动时显示欢迎和快速入门指南</span>
                </div>
                <label class="toggle">
                    <input type="checkbox" v-model="showWelcome" />
                    <span class="toggle-slider"></span>
                </label>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">语言和区域</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">界面语言</span>
                </div>
                <select v-model="language" class="setting-select">
                    <option v-for="lang in languages" :key="lang.value" :value="lang.value">
                        {{ lang.label }}
                    </option>
                </select>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">默认设置</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">默认 Shell</span>
                    <span class="setting-desc">新终端会话使用的默认 Shell</span>
                </div>
                <select v-model="defaultShell" class="setting-select">
                    <option v-for="shell in shells" :key="shell.value" :value="shell.value">
                        {{ shell.label }}
                    </option>
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
    transition: background 0.2s;
}

.setting-item:hover {
    background: rgba(255, 255, 255, 0.06);
}

.setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.setting-label {
    font-size: 0.9rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.9);
}

.setting-desc {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
}

/* Toggle Switch */
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
    transition: all 0.2s;
}

.setting-select:hover {
    background: rgba(255, 255, 255, 0.12);
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
