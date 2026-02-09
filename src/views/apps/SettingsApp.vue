<!--
  SettingsApp.vue - 系统设置应用主组件
  macOS 风格的设置面板，sidebar + content 布局
-->
<script setup lang="ts">
import { ref, computed } from "vue";
import GeneralPanel from "./settings/GeneralPanel.vue";
import AppearancePanel from "./settings/AppearancePanel.vue";
import TerminalPanel from "./settings/TerminalPanel.vue";
import ConnectionPanel from "./settings/ConnectionPanel.vue";
import AboutPanel from "./settings/AboutPanel.vue";
import TrafficLights from "@/components/desktop/TrafficLights.vue";
import { Input } from "@/components/ui/input";

defineProps<{
    sessionId: string;
    startDrag?: (e: MouseEvent) => void;
    close?: () => void;
    minimize?: () => void;
    maximize?: () => void;
}>();

type PanelId = "general" | "appearance" | "terminal" | "connection" | "about";

const activePanel = ref<PanelId>("general");
const searchQuery = ref("");

const panels: { id: PanelId; label: string; icon: string; keywords: string[] }[] = [
    { id: "general", label: "通用", icon: "icon-[mdi--cog]", keywords: ["general", "startup", "language", "shell", "启动", "语言", "通用"] },
    { id: "appearance", label: "外观", icon: "icon-[mdi--palette]", keywords: ["appearance", "theme", "color", "font", "background", "外观", "主题", "颜色", "字体", "背景"] },
    { id: "terminal", label: "终端", icon: "icon-[mdi--console]", keywords: ["terminal", "font", "cursor", "scrollback", "终端", "字体", "光标", "回滚"] },
    { id: "connection", label: "连接", icon: "icon-[mdi--connection]", keywords: ["connection", "ssh", "auth", "compression", "timeout", "连接", "认证", "压缩", "超时"] },
    { id: "about", label: "关于", icon: "icon-[mdi--information]", keywords: ["about", "version", "synapsh", "关于", "版本"] },
];

const filteredPanels = computed(() => {
    if (!searchQuery.value) return panels;
    const query = searchQuery.value.toLowerCase();
    return panels.filter(p =>
        p.label.toLowerCase().includes(query) ||
        p.keywords.some(k => k.toLowerCase().includes(query))
    );
});

const currentPanelLabel = computed(() => {
    return panels.find((p) => p.id === activePanel.value)?.label || "设置";
});
</script>

<template>
    <div
        class="grid grid-cols-[240px_1fr] h-full bg-background text-foreground rounded-b-2xl overflow-hidden border border-border sm:grid-cols-[200px_1fr] md:grid-cols-[240px_1fr]">
        <!-- 侧边栏 -->
        <aside class="flex flex-col bg-muted/30 border-r border-border backdrop-blur-xl pt-3">
            <div class="px-4 pb-2 flex flex-col gap-3 drag-region" @mousedown="startDrag">
                <!-- Mac Traffic Lights -->
                <TrafficLights
                    @close="close?.()"
                    @minimize="minimize?.()"
                    @maximize="maximize?.()"
                />

                <div class="relative px-2 pb-2">
                    <span
                        class="icon-[mdi--magnify] absolute left-5 top-1/2 -translate-y-1/2 text-muted-foreground text-base pointer-events-none z-10"></span>
                    <Input type="text" placeholder="搜索设置" v-model="searchQuery"
                        class="h-8 bg-secondary/50 hover:bg-secondary/80 focus:bg-background border border-transparent focus:border-primary/30 focus-visible:ring-0 focus-visible:ring-offset-0 pl-9 pr-3 text-sm shadow-none rounded-md placeholder:text-muted-foreground/50 transition-all" />
                </div>
            </div>

            <nav class="flex-1 overflow-y-auto px-2 py-1 space-y-0.5">
                <template v-if="filteredPanels.length > 0">
                    <div v-for="panel in filteredPanels" :key="panel.id"
                        class="flex items-center gap-3 px-3 py-1.5 rounded-md cursor-pointer transition-colors text-sm group"
                        :class="[
                            activePanel === panel.id
                                ? 'bg-primary text-primary-foreground font-medium shadow-sm'
                                : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
                        ]" @click="activePanel = panel.id">
                        <span :class="panel.icon" class="text-lg opacity-80 group-hover:opacity-100"></span>
                        <span>{{ panel.label }}</span>
                    </div>
                </template>
                <div v-else class="px-4 py-8 text-center text-muted-foreground text-xs">
                    未找到相关设置
                </div>
            </nav>
        </aside>

        <!-- 内容区域 -->
        <main class="flex flex-col h-full overflow-hidden bg-background/50">
            <div class="flex-1 overflow-y-auto w-full">
                <GeneralPanel v-if="activePanel === 'general'" />
                <AppearancePanel v-else-if="activePanel === 'appearance'" />
                <TerminalPanel v-else-if="activePanel === 'terminal'" />
                <ConnectionPanel v-else-if="activePanel === 'connection'" />
                <AboutPanel v-else-if="activePanel === 'about'" />
            </div>
        </main>
    </div>
</template>

<style scoped>
/* 
  Use standard CSS for drag-region as Tailwind doesn't support 
  -webkit-app-region directly via standard utilities without plugins 
*/
.drag-region {
    -webkit-app-region: drag;
}

button {
    -webkit-app-region: no-drag;
}

input {
    -webkit-app-region: no-drag;
}
</style>
