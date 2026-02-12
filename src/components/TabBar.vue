<!--
  TabBar.vue - 顶部标签栏
  原生标题栏覆盖模式下的标签栏，包含标签管理与系统外观快捷设置
-->
<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";

export interface Tab {
    id: string;
    title: string;
    icon?: string;
    closable?: boolean;
    view: "machine-manager" | "desktop";
}

const props = defineProps<{
    tabs: Tab[];
    activeTabId: string;
}>();

const emit = defineEmits<{
    (e: "switch-tab", id: string): void;
    (e: "close-tab", id: string): void;
    (e: "new-tab", id: string): void;
}>();

type ThemeMode = "light" | "dark" | "auto";

const mode = useColorMode({
    emitAuto: true,
    storageKey: "vueuse-color-mode",
    attribute: "class",
    modes: { dark: "dark", light: "light", auto: "auto" },
});

const themeMode = computed<ThemeMode>({
    get: () => (mode.value === "light" || mode.value === "dark" || mode.value === "auto" ? mode.value : "auto"),
    set: (value) => {
        mode.value = value;
    },
});

const accentColor = useLocalStorage("appearance-accent-color", "#3b82f6");
const isAppearanceOpen = ref(false);
const appearanceMenuRef = ref<HTMLElement | null>(null);
const platform = ref<"macos" | "windows" | "linux" | "unknown">("unknown");
const isMac = computed(() => platform.value === "macos");
const leadingInset = computed(() => (isMac.value ? "84px" : "8px"));
const trailingInset = computed(() => "8px");
const contentOffsetClass = computed(() => (isMac.value ? "-translate-y-px" : ""));

const accentColors = [
    { value: "#3b82f6", name: "蓝色" },
    { value: "#22c55e", name: "绿色" },
    { value: "#f59e0b", name: "橙色" },
    { value: "#ef4444", name: "红色" },
    { value: "#14b8a6", name: "青色" },
    { value: "#a855f7", name: "紫色" },
];

function applyAccentColor(color: string) {
    accentColor.value = color;
    document.documentElement.style.setProperty("--accent-color", color);
}

function detectPlatform() {
    const navigatorWithUserAgentData = navigator as Navigator & { userAgentData?: { platform?: string } };
    const ua = navigator.userAgent.toLowerCase();
    const platformText = (navigatorWithUserAgentData.userAgentData?.platform || navigator.platform || "").toLowerCase();
    const source = `${ua} ${platformText}`;

    if (source.includes("mac")) return "macos";
    if (source.includes("win")) return "windows";
    if (source.includes("linux")) return "linux";
    return "unknown";
}

function toggleAppearanceMenu() {
    isAppearanceOpen.value = !isAppearanceOpen.value;
}

function handleClickOutside(event: MouseEvent) {
    const target = event.target as Node | null;
    if (!target) return;

    if (appearanceMenuRef.value && !appearanceMenuRef.value.contains(target)) {
        isAppearanceOpen.value = false;
    }
}

onMounted(() => {
    platform.value = detectPlatform();
    document.documentElement.style.setProperty("--accent-color", accentColor.value);
    window.addEventListener("mousedown", handleClickOutside);
});

onUnmounted(() => {
    window.removeEventListener("mousedown", handleClickOutside);
});
</script>

<template>
    <div
        class="h-10 flex items-center select-none relative bg-sidebar/80 backdrop-blur-xl border-b border-border/50"
        :style="{ paddingLeft: leadingInset, paddingRight: trailingInset }"
    >
        <div
            class="w-8 h-8 flex items-center justify-center shrink-0 rounded-lg"
            :class="contentOffsetClass"
            data-tauri-drag-region
        >
            <div class="size-5 rounded-lg bg-gradient-to-br from-brand to-brand/70 grid place-items-center">
                <span class="icon-[mdi--lightning-bolt] text-brand-foreground text-xs leading-none"></span>
            </div>
        </div>

        <div
            class="flex-1 min-w-0 flex items-center overflow-x-auto [&::-webkit-scrollbar]:hidden gap-1 pl-1"
            :class="contentOffsetClass"
        >
            <div
                v-for="tab in tabs"
                :key="tab.id"
                class="group/tab relative flex items-center h-7 min-w-[100px] max-w-[160px] px-2.5 cursor-pointer text-sm transition-all duration-200 ease-out rounded-lg"
                :class="[
                    tab.id === activeTabId
                        ? 'bg-card text-foreground shadow-sm'
                        : 'text-muted-foreground hover:bg-accent/30 hover:text-foreground'
                ]"
                @click="emit('switch-tab', tab.id)"
            >
                <span class="text-sm mr-1.5 shrink-0" :class="tab.icon || 'icon-[mdi--terminal]'"></span>
                <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis text-xs font-medium">{{ tab.title }}</span>

                <button
                    v-if="tab.closable !== false"
                    class="flex items-center justify-center size-4 ml-1.5 rounded border-none bg-transparent text-muted-foreground transition-all duration-150"
                    :class="tab.id === activeTabId ? 'opacity-60 hover:opacity-100 hover:bg-muted hover:text-foreground' : 'opacity-0 group-hover/tab:opacity-60 hover:!opacity-100 hover:bg-accent hover:text-foreground'"
                    @click.stop="emit('close-tab', tab.id)"
                >
                    <span class="icon-[mdi--close] text-xs"></span>
                </button>
            </div>

            <button
                class="shrink-0 flex items-center justify-center size-6 rounded-md border-none bg-transparent text-muted-foreground cursor-pointer transition-all duration-150 hover:bg-accent hover:text-foreground"
                @click="emit('new-tab', '')"
                title="新建标签页"
            >
                <span class="icon-[mdi--plus] text-base"></span>
            </button>
        </div>

        <div class="shrink-0 flex items-center gap-1 px-2 no-drag" :class="contentOffsetClass">
            <div ref="appearanceMenuRef" class="relative">
                <button
                    class="flex items-center justify-center size-6 rounded-md border-none bg-transparent text-muted-foreground cursor-pointer transition-all duration-150 hover:bg-accent hover:text-foreground"
                    :class="isAppearanceOpen ? 'bg-accent text-foreground' : ''"
                    title="系统外观设置"
                    @mousedown.stop
                    @click.stop="toggleAppearanceMenu"
                >
                    <span class="icon-[mdi--palette-outline] text-base"></span>
                </button>

                <div
                    v-if="isAppearanceOpen"
                    class="fixed right-3 top-11 z-[9999] w-60 rounded-2xl border border-white/12 bg-[#111827]/95 p-3 shadow-2xl backdrop-blur-2xl no-drag"
                    @mousedown.stop
                    @click.stop
                >
                    <div class="text-[11px] uppercase tracking-[0.08em] text-neutral-400 mb-2">系统外观</div>

                    <div class="grid grid-cols-3 gap-1 mb-3">
                        <button
                            class="h-7 rounded-lg text-xs transition-colors border"
                            :class="themeMode === 'light'
                                ? 'bg-blue-500/20 border-blue-400/60 text-blue-100'
                                : 'bg-white/5 border-white/10 text-neutral-300 hover:bg-white/10'"
                            @click="themeMode = 'light'"
                        >
                            浅色
                        </button>
                        <button
                            class="h-7 rounded-lg text-xs transition-colors border"
                            :class="themeMode === 'dark'
                                ? 'bg-blue-500/20 border-blue-400/60 text-blue-100'
                                : 'bg-white/5 border-white/10 text-neutral-300 hover:bg-white/10'"
                            @click="themeMode = 'dark'"
                        >
                            深色
                        </button>
                        <button
                            class="h-7 rounded-lg text-xs transition-colors border"
                            :class="themeMode === 'auto'
                                ? 'bg-blue-500/20 border-blue-400/60 text-blue-100'
                                : 'bg-white/5 border-white/10 text-neutral-300 hover:bg-white/10'"
                            @click="themeMode = 'auto'"
                        >
                            自动
                        </button>
                    </div>

                    <div class="text-[11px] uppercase tracking-[0.08em] text-neutral-400 mb-2">强调色</div>
                    <div class="flex items-center gap-2">
                        <button
                            v-for="color in accentColors"
                            :key="color.value"
                            class="size-5 rounded-full border transition-transform hover:scale-110"
                            :class="accentColor === color.value ? 'border-white' : 'border-white/20'"
                            :style="{ backgroundColor: color.value }"
                            :title="color.name"
                            @click="applyAccentColor(color.value)"
                        >
                            <span v-if="accentColor === color.value" class="icon-[mdi--check] text-[10px] text-white"></span>
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <div class="w-2 h-8 shrink-0" data-tauri-drag-region></div>
    </div>
</template>

<style scoped>
.no-drag {
    -webkit-app-region: no-drag;
}
</style>
