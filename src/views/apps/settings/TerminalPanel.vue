<!--
  TerminalPanel.vue - 终端设置面板
  设置终端字体、配色方案、光标样式等
-->
<script setup lang="ts">
import { ref } from "vue";
import { Switch } from "@/components/ui/switch";

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
    <div class="p-8 max-w-5xl mx-auto text-foreground animate-in fade-in duration-500">
        <div class="mb-8">
            <h2 class="text-3xl font-bold tracking-tight mb-2">终端</h2>
            <p class="text-muted-foreground">自定义终端外观和行为</p>
        </div>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">字体</h3>

            <div class="bg-secondary/20 border border-border rounded-xl overflow-hidden">
                <div
                    class="flex items-center justify-between p-4 border-b border-border hover:bg-foreground/5 transition-colors">
                    <span class="text-sm font-medium">字体族</span>
                    <select v-model="fontFamily"
                        class="bg-background border border-border rounded-md px-3 py-1.5 text-sm focus:ring-2 focus:ring-primary/50 outline-none text-foreground min-w-[140px]">
                        <option v-for="font in fonts" :key="font.value" :value="font.value"
                            class="bg-gray-900 text-white">
                            {{ font.label }}
                        </option>
                    </select>
                </div>

                <div class="flex items-center justify-between p-4 hover:bg-foreground/5 transition-colors">
                    <span class="text-sm font-medium">字体大小</span>
                    <div class="flex items-center gap-2 bg-background/50 rounded-lg p-1 border border-border/50">
                        <button @click="fontSize = Math.max(10, fontSize - 1)"
                            class="w-7 h-7 flex items-center justify-center rounded hover:bg-foreground/10 active:bg-foreground/20 transition-colors">−</button>
                        <span class="w-12 text-center text-sm font-mono">{{ fontSize }}px</span>
                        <button @click="fontSize = Math.min(24, fontSize + 1)"
                            class="w-7 h-7 flex items-center justify-center rounded hover:bg-foreground/10 active:bg-foreground/20 transition-colors">+</button>
                    </div>
                </div>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">光标</h3>

            <div class="flex gap-4 mb-4">
                <div class="flex-1 flex flex-col items-center gap-2 p-4 bg-secondary/20 border-2 border-transparent rounded-xl cursor-pointer transition-all hover:bg-secondary/40"
                    :class="{ 'border-primary bg-primary/5': cursorStyle === 'block' }" @click="cursorStyle = 'block'">
                    <div class="w-10 h-6 bg-[#1e1e2e] rounded relative flex items-end justify-center pb-1">
                        <div class="w-2.5 h-4 bg-gray-200"></div>
                    </div>
                    <span class="text-xs text-muted-foreground">方块</span>
                </div>
                <div class="flex-1 flex flex-col items-center gap-2 p-4 bg-secondary/20 border-2 border-transparent rounded-xl cursor-pointer transition-all hover:bg-secondary/40"
                    :class="{ 'border-primary bg-primary/5': cursorStyle === 'underline' }"
                    @click="cursorStyle = 'underline'">
                    <div class="w-10 h-6 bg-[#1e1e2e] rounded relative flex items-end justify-center pb-1">
                        <div class="w-2.5 h-0.5 bg-gray-200"></div>
                    </div>
                    <span class="text-xs text-muted-foreground">下划线</span>
                </div>
                <div class="flex-1 flex flex-col items-center gap-2 p-4 bg-secondary/20 border-2 border-transparent rounded-xl cursor-pointer transition-all hover:bg-secondary/40"
                    :class="{ 'border-primary bg-primary/5': cursorStyle === 'bar' }" @click="cursorStyle = 'bar'">
                    <div class="w-10 h-6 bg-[#1e1e2e] rounded relative flex items-end justify-center pb-1">
                        <div class="w-0.5 h-4 bg-gray-200"></div>
                    </div>
                    <span class="text-xs text-muted-foreground">竖线</span>
                </div>
            </div>

            <div
                class="bg-secondary/20 border border-border rounded-xl overflow-hidden p-4 flex items-center justify-between hover:bg-foreground/5 transition-colors">
                <span class="text-sm font-medium">光标闪烁</span>
                <Switch v-model:checked="cursorBlink" />
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">配色方案</h3>

            <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
                <div v-for="scheme in colorSchemes" :key="scheme.value"
                    class="flex flex-col gap-2 p-3 border-2 border-transparent rounded-xl cursor-pointer transition-all hover:bg-secondary/20"
                    :class="{ 'border-primary bg-primary/5': colorScheme === scheme.value }"
                    @click="colorScheme = scheme.value">
                    <div class="rounded-lg p-3 font-mono text-xs shadow-sm" :style="{ background: scheme.bg }">
                        <span :style="{ color: scheme.fg }">$ echo</span>
                        <span :style="{ color: scheme.accent }"> "Hello"</span>
                    </div>
                    <span class="text-center text-xs font-medium text-muted-foreground">{{ scheme.label }}</span>
                </div>
            </div>
        </section>

        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">高级</h3>

            <div
                class="bg-secondary/20 border border-border rounded-xl overflow-hidden p-4 flex items-center justify-between hover:bg-foreground/5 transition-colors">
                <div class="flex flex-col gap-1">
                    <span class="text-sm font-medium">回滚行数</span>
                    <span class="text-xs text-muted-foreground">终端保留的历史行数</span>
                </div>
                <select v-model="scrollback"
                    class="bg-background border border-border rounded-md px-3 py-1.5 text-sm focus:ring-2 focus:ring-primary/50 outline-none text-foreground min-w-[140px]">
                    <option :value="500" class="bg-gray-900 text-white">500 行</option>
                    <option :value="1000" class="bg-gray-900 text-white">1000 行</option>
                    <option :value="5000" class="bg-gray-900 text-white">5000 行</option>
                    <option :value="10000" class="bg-gray-900 text-white">10000 行</option>
                </select>
            </div>
        </section>
    </div>
</template>
