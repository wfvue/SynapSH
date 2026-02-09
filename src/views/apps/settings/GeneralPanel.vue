<!--
  GeneralPanel.vue - 通用设置面板
  设置启动行为、默认应用等通用配置
-->
<script setup lang="ts">
import { ref } from "vue";
import { Switch } from "@/components/ui/switch";

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
    <div class="p-8 max-w-5xl mx-auto text-foreground animate-in fade-in duration-500">
        <div class="mb-8">
            <h2 class="text-3xl font-bold tracking-tight mb-2">通用</h2>
            <p class="text-muted-foreground">配置基本行为和偏好设置</p>
        </div>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">启动</h3>

            <div class="bg-secondary/20 border border-border rounded-xl overflow-hidden">
                <div
                    class="flex items-center justify-between p-4 border-b border-border hover:bg-foreground/5 transition-colors">
                    <div class="flex flex-col gap-1">
                        <span class="text-sm font-medium">启动时自动连接</span>
                        <span class="text-xs text-muted-foreground">启动应用后自动连接上次使用的服务器</span>
                    </div>
                    <Switch v-model:checked="autoConnect" />
                </div>

                <div class="flex items-center justify-between p-4 hover:bg-foreground/5 transition-colors">
                    <div class="flex flex-col gap-1">
                        <span class="text-sm font-medium">显示欢迎界面</span>
                        <span class="text-xs text-muted-foreground">启动时显示欢迎和快速入门指南</span>
                    </div>
                    <Switch v-model:checked="showWelcome" />
                </div>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">语言和区域</h3>

            <div
                class="bg-secondary/20 border border-border rounded-xl overflow-hidden p-4 flex items-center justify-between hover:bg-foreground/5 transition-colors">
                <span class="text-sm font-medium">界面语言</span>
                <select v-model="language"
                    class="bg-background border border-border rounded-md px-3 py-1.5 text-sm focus:ring-2 focus:ring-primary/50 outline-none text-foreground min-w-[140px]">
                    <option v-for="lang in languages" :key="lang.value" :value="lang.value"
                        class="bg-gray-900 text-white">
                        {{ lang.label }}
                    </option>
                </select>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">默认设置</h3>

            <div
                class="bg-secondary/20 border border-border rounded-xl overflow-hidden p-4 flex items-center justify-between hover:bg-foreground/5 transition-colors">
                <div class="flex flex-col gap-1">
                    <span class="text-sm font-medium">默认 Shell</span>
                    <span class="text-xs text-muted-foreground">新终端会话使用的默认 Shell</span>
                </div>
                <select v-model="defaultShell"
                    class="bg-background border border-border rounded-md px-3 py-1.5 text-sm focus:ring-2 focus:ring-primary/50 outline-none text-foreground min-w-[140px]">
                    <option v-for="shell in shells" :key="shell.value" :value="shell.value"
                        class="bg-gray-900 text-white">
                        {{ shell.label }}
                    </option>
                </select>
            </div>
        </section>
    </div>
</template>
