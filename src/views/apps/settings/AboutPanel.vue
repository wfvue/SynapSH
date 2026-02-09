<!--
  AboutPanel.vue - 关于面板
  显示应用版本信息和系统信息
-->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { getVersion } from "@tauri-apps/api/app";

const appVersion = ref("1.0.0");
const buildDate = "2026-02-07";

onMounted(async () => {
    try {
        appVersion.value = await getVersion();
    } catch (e) {
        console.warn("Failed to get app version:", e);
    }
});

const links = [
    { label: "GitHub 仓库", url: "https://github.com", icon: "icon-[mdi--github]" },
    { label: "提交反馈", url: "#", icon: "icon-[mdi--message-alert]" },
    { label: "文档", url: "#", icon: "icon-[mdi--book-open-page-variant]" },
];
</script>

<template>
    <div class="settings-panel about-panel">
        <div class="app-info">
            <div class="app-logo">
                <span class="icon-[mdi--console-network]"></span>
            </div>
            <h1 class="app-name">SynapSH</h1>
            <p class="app-tagline">光析 - 智能 SSH 工作站</p>
            <div class="version-info">
                <span class="version">v{{ appVersion }}</span>
                <span class="divider">•</span>
                <span class="build-date">{{ buildDate }}</span>
            </div>
        </div>

        <section class="settings-section">
            <h3 class="section-title">关于</h3>

            <div class="about-content">
                <p>
                    SynapSH 是一款现代化的 SSH 客户端，提供类似桌面操作系统的交互体验。
                    集成终端、文件管理、系统监控等功能于一体。
                </p>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">链接</h3>

            <div class="links-list">
                <a v-for="link in links" :key="link.label" :href="link.url" class="link-item" target="_blank">
                    <span :class="link.icon" class="link-icon"></span>
                    <span class="link-label">{{ link.label }}</span>
                    <span class="icon-[mdi--chevron-right] link-arrow"></span>
                </a>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">技术栈</h3>

            <div class="tech-stack">
                <div class="tech-item">
                    <span class="icon-[mdi--vuejs] tech-icon vue"></span>
                    <span>Vue 3</span>
                </div>
                <div class="tech-item">
                    <span class="icon-[mdi--language-rust] tech-icon rust"></span>
                    <span>Rust</span>
                </div>
                <div class="tech-item">
                    <span class="icon-[mdi--application-brackets] tech-icon tauri"></span>
                    <span>Tauri</span>
                </div>
                <div class="tech-item">
                    <span class="icon-[mdi--language-typescript] tech-icon ts"></span>
                    <span>TypeScript</span>
                </div>
            </div>
        </section>

        <footer class="about-footer">
            <p>Made with ❤️ for developers</p>
            <p class="copyright">© 2026 SynapSH Team. All rights reserved.</p>
        </footer>
    </div>
</template>

<style scoped>
.settings-panel {
    padding: 24px;
    color: rgba(255, 255, 255, 0.9);
}

.about-panel {
    display: flex;
    flex-direction: column;
    min-height: 100%;
}

.app-info {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 32px 0;
    text-align: center;
}

.app-logo {
    width: 80px;
    height: 80px;
    background: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
    border-radius: 20px;
    display: grid;
    place-items: center;
    font-size: 40px;
    color: white;
    margin-bottom: 16px;
    box-shadow: 0 8px 24px rgba(59, 130, 246, 0.3);
}

.app-name {
    font-size: 1.75rem;
    font-weight: 700;
    margin: 0 0 4px 0;
    background: linear-gradient(135deg, #fff 0%, #a0aec0 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

.app-tagline {
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.5);
    margin: 0 0 12px 0;
}

.version-info {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.4);
}

.version {
    padding: 4px 10px;
    background: rgba(59, 130, 246, 0.15);
    border-radius: 20px;
    color: #60a5fa;
}

.divider {
    color: rgba(255, 255, 255, 0.2);
}

.settings-section {
    margin-bottom: 24px;
}

.section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: rgba(255, 255, 255, 0.4);
    margin: 0 0 12px 0;
}

.about-content {
    padding: 16px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    font-size: 0.85rem;
    line-height: 1.6;
    color: rgba(255, 255, 255, 0.7);
}

.about-content p {
    margin: 0;
}

/* Links List */
.links-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.link-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.9);
    text-decoration: none;
    transition: all 0.2s;
}

.link-item:hover {
    background: rgba(255, 255, 255, 0.08);
}

.link-icon {
    font-size: 20px;
    color: rgba(255, 255, 255, 0.6);
}

.link-label {
    flex: 1;
    font-size: 0.9rem;
}

.link-arrow {
    font-size: 20px;
    color: rgba(255, 255, 255, 0.3);
}

/* Tech Stack */
.tech-stack {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
}

.tech-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.04);
    border-radius: 8px;
    font-size: 0.85rem;
}

.tech-icon {
    font-size: 20px;
}

.tech-icon.vue {
    color: #42b883;
}

.tech-icon.rust {
    color: #dea584;
}

.tech-icon.tauri {
    color: #ffc131;
}

.tech-icon.ts {
    color: #3178c6;
}

/* Footer */
.about-footer {
    margin-top: auto;
    padding-top: 24px;
    text-align: center;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.4);
}

.about-footer p {
    margin: 0 0 4px 0;
}

.copyright {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.3);
}
</style>
