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
    <div class="flex flex-col min-h-full p-8 max-w-5xl mx-auto text-foreground animate-in fade-in duration-500">
        <div class="flex flex-col items-center text-center py-8">
            <div
                class="w-20 h-20 bg-gradient-to-br from-blue-500 to-purple-500 rounded-2xl flex items-center justify-center text-4xl text-white mb-4 shadow-xl shadow-blue-500/30">
                <span class="icon-[mdi--console-network]"></span>
            </div>
            <h1
                class="text-3xl font-bold mb-1 bg-gradient-to-br from-foreground to-muted-foreground/70 bg-clip-text text-transparent">
                SynapSH</h1>
            <p class="text-sm text-muted-foreground mb-3">光析 - 智能 SSH 工作站</p>
            <div class="flex items-center gap-2 text-xs text-muted-foreground/80">
                <span class="px-2.5 py-1 bg-blue-500/15 text-blue-400 rounded-full">v{{ appVersion }}</span>
                <span class="text-muted-foreground/50">•</span>
                <span>{{ buildDate }}</span>
            </div>
        </div>

        <section class="mb-8">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">关于</h3>
            <div class="p-4 bg-secondary/20 rounded-xl text-sm leading-relaxed text-muted-foreground">
                <p>
                    SynapSH 是一款现代化的 SSH 客户端，提供类似桌面操作系统的交互体验。
                    集成终端、文件管理、系统监控等功能于一体。
                </p>
            </div>
        </section>

        <section class="mb-8">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">链接</h3>
            <div class="flex flex-col gap-1">
                <a v-for="link in links" :key="link.label" :href="link.url"
                    class="flex items-center gap-3 p-3 bg-secondary/20 rounded-xl text-foreground/90 hover:bg-secondary/40 transition-colors"
                    target="_blank">
                    <span :class="link.icon" class="text-xl text-muted-foreground"></span>
                    <span class="flex-1 text-sm">{{ link.label }}</span>
                    <span class="icon-[mdi--chevron-right] text-muted-foreground/50 text-xl"></span>
                </a>
            </div>
        </section>

        <section class="mb-8">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">技术栈</h3>
            <div class="flex flex-wrap gap-2">
                <div class="flex items-center gap-2 px-3 py-2 bg-secondary/20 rounded-lg text-sm">
                    <span class="icon-[mdi--vuejs] text-xl text-[#42b883]"></span>
                    <span>Vue 3</span>
                </div>
                <div class="flex items-center gap-2 px-3 py-2 bg-secondary/20 rounded-lg text-sm">
                    <span class="icon-[mdi--language-rust] text-xl text-[#dea584]"></span>
                    <span>Rust</span>
                </div>
                <div class="flex items-center gap-2 px-3 py-2 bg-secondary/20 rounded-lg text-sm">
                    <span class="icon-[mdi--application-brackets] text-xl text-[#ffc131]"></span>
                    <span>Tauri</span>
                </div>
                <div class="flex items-center gap-2 px-3 py-2 bg-secondary/20 rounded-lg text-sm">
                    <span class="icon-[mdi--language-typescript] text-xl text-[#3178c6]"></span>
                    <span>TypeScript</span>
                </div>
            </div>
        </section>

        <footer class="mt-auto pt-8 text-center text-xs text-muted-foreground/60">
            <p class="mb-1">Made with ❤️ for developers</p>
            <p class="text-muted-foreground/40">© 2026 SynapSH Team. All rights reserved.</p>
        </footer>
    </div>
</template>
