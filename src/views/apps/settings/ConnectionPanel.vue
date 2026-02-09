<!--
  ConnectionPanel.vue - 连接设置面板
  设置 SSH 连接默认配置
-->
<script setup lang="ts">
import { ref } from "vue";

// 连接设置项
const timeout = ref(30);
const keepAlive = ref(60);
const compression = ref(false);
const sshVersion = ref("2");
const authMethod = ref("password");

const authMethods = [
    { value: "password", label: "密码", icon: "icon-[mdi--key]" },
    { value: "publickey", label: "密钥", icon: "icon-[mdi--key-chain]" },
    { value: "both", label: "密码 + 密钥", icon: "icon-[mdi--shield-key]" },
];
</script>

<template>
    <div class="settings-panel">
        <h2 class="panel-title">连接</h2>
        <p class="panel-subtitle">SSH 连接相关设置</p>

        <section class="settings-section">
            <h3 class="section-title">身份验证</h3>

            <div class="auth-methods">
                <div v-for="method in authMethods" :key="method.value" class="auth-option"
                    :class="{ active: authMethod === method.value }" @click="authMethod = method.value">
                    <span :class="method.icon" class="auth-icon"></span>
                    <span class="auth-label">{{ method.label }}</span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">超时设置</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">连接超时</span>
                    <span class="setting-desc">等待服务器响应的最长时间</span>
                </div>
                <div class="input-with-unit">
                    <input type="number" v-model="timeout" min="5" max="120" class="number-field" />
                    <span class="unit">秒</span>
                </div>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">心跳间隔</span>
                    <span class="setting-desc">保持连接活跃的心跳发送间隔</span>
                </div>
                <div class="input-with-unit">
                    <input type="number" v-model="keepAlive" min="0" max="300" class="number-field" />
                    <span class="unit">秒</span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">高级选项</h3>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">数据压缩</span>
                    <span class="setting-desc">压缩传输数据以节省带宽</span>
                </div>
                <label class="toggle">
                    <input type="checkbox" v-model="compression" />
                    <span class="toggle-slider"></span>
                </label>
            </div>

            <div class="setting-item">
                <div class="setting-info">
                    <span class="setting-label">SSH 协议版本</span>
                </div>
                <div class="radio-group">
                    <label class="radio-item" :class="{ active: sshVersion === '2' }">
                        <input type="radio" v-model="sshVersion" value="2" />
                        <span>SSH 2</span>
                    </label>
                    <label class="radio-item" :class="{ active: sshVersion === '1' }">
                        <input type="radio" v-model="sshVersion" value="1" />
                        <span>SSH 1</span>
                    </label>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">密钥管理</h3>

            <div class="keys-list">
                <div class="key-item">
                    <span class="icon-[mdi--key-variant] key-icon"></span>
                    <div class="key-info">
                        <span class="key-name">id_rsa</span>
                        <span class="key-path">~/.ssh/id_rsa</span>
                    </div>
                    <span class="key-status active">已添加</span>
                </div>
                <button class="add-key-btn">
                    <span class="icon-[mdi--plus]"></span>
                    添加密钥
                </button>
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

/* Auth Methods */
.auth-methods {
    display: flex;
    gap: 10px;
}

.auth-option {
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

.auth-option:hover {
    background: rgba(255, 255, 255, 0.08);
}

.auth-option.active {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
}

.auth-icon {
    font-size: 24px;
    color: rgba(255, 255, 255, 0.7);
}

.auth-option.active .auth-icon {
    color: #60a5fa;
}

.auth-label {
    font-size: 0.85rem;
    font-weight: 500;
}

/* Input with unit */
.input-with-unit {
    display: flex;
    align-items: center;
    gap: 8px;
}

.number-field {
    width: 70px;
    padding: 8px 10px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: white;
    font-size: 0.85rem;
    text-align: center;
}

.number-field:focus {
    outline: none;
    border-color: rgba(59, 130, 246, 0.5);
}

.unit {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.5);
}

/* Radio Group */
.radio-group {
    display: flex;
    gap: 8px;
}

.radio-item {
    display: flex;
    align-items: center;
    padding: 8px 14px;
    background: rgba(255, 255, 255, 0.06);
    border: 2px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
    font-size: 0.85rem;
}

.radio-item input {
    display: none;
}

.radio-item:hover {
    background: rgba(255, 255, 255, 0.1);
}

.radio-item.active {
    border-color: #3b82f6;
    background: rgba(59, 130, 246, 0.1);
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

/* Keys List */
.keys-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.key-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
}

.key-icon {
    font-size: 20px;
    color: #f59e0b;
}

.key-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.key-name {
    font-size: 0.9rem;
    font-weight: 500;
}

.key-path {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
    font-family: monospace;
}

.key-status {
    font-size: 0.75rem;
    padding: 4px 10px;
    border-radius: 20px;
    background: rgba(34, 197, 94, 0.15);
    color: #22c55e;
}

.add-key-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px dashed rgba(255, 255, 255, 0.2);
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
}

.add-key-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.3);
    color: rgba(255, 255, 255, 0.9);
}
</style>
