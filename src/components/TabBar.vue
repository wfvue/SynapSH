<!--
  TabBar.vue - 顶部标签栏
  原生标题栏覆盖模式下的标签栏，包含标签管理与系统外观快捷设置
  支持全屏场景下悬停显示
-->
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

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
  isFullscreen?: boolean;
}>();

const emit = defineEmits<{
  (e: "switch-tab", id: string): void;
  (e: "close-tab", id: string): void;
  (e: "new-tab", id: string): void;
}>();

const isHovered = ref(false);
const { text } = useInterfaceLanguage();

type ThemeMode = "light" | "dark" | "auto";

const mode = useColorMode({
  emitAuto: true,
  storageKey: "vueuse-color-mode",
  attribute: "class",
  modes: { dark: "dark", light: "light", auto: "auto" },
});

const themeMode = computed<ThemeMode>({
  get: () =>
    mode.value === "light" || mode.value === "dark" || mode.value === "auto" ? mode.value : "auto",
  set: (value) => {
    mode.value = value;
  },
});

const accentColor = useLocalStorage("appearance-accent-color", "#0a84ff");
const isAppearanceOpen = ref(false);
const platform = ref<"macos" | "windows" | "linux" | "unknown">("unknown");
const isMac = computed(() => platform.value === "macos");
const leadingInset = computed(() => (isMac.value ? "84px" : "8px"));
const trailingInset = computed(() => "8px");
const contentOffsetClass = computed(() => (isMac.value ? "-translate-y-px" : ""));

const accentColors = [
  { value: "#0a84ff", nameEn: "Blue", nameZh: "蓝色" },
  { value: "#22c55e", nameEn: "Green", nameZh: "绿色" },
  { value: "#f59e0b", nameEn: "Orange", nameZh: "橙色" },
  { value: "#ef4444", nameEn: "Red", nameZh: "红色" },
  { value: "#14b8a6", nameEn: "Teal", nameZh: "青色" },
  { value: "#a855f7", nameEn: "Purple", nameZh: "紫色" },
];

function applyAccentColor(color: string) {
  accentColor.value = color;
  document.documentElement.style.setProperty("--accent-color", color);
}

function detectPlatform() {
  const navigatorWithUserAgentData = navigator as Navigator & {
    userAgentData?: { platform?: string };
  };
  const ua = navigator.userAgent.toLowerCase();
  const platformText = (
    navigatorWithUserAgentData.userAgentData?.platform ||
    navigator.platform ||
    ""
  ).toLowerCase();
  const source = `${ua} ${platformText}`;

  if (source.includes("mac")) return "macos";
  if (source.includes("win")) return "windows";
  if (source.includes("linux")) return "linux";
  return "unknown";
}

function toggleAppearanceMenu() {
  isAppearanceOpen.value = !isAppearanceOpen.value;
}

onMounted(() => {
  platform.value = detectPlatform();
  document.documentElement.style.setProperty("--accent-color", accentColor.value);
});
</script>

<template>
  <div
    class="flex items-center select-none bg-sidebar/80 backdrop-blur-xl transition-all duration-300 ease-out"
    :class="
      props.isFullscreen
        ? 'h-0 opacity-0 overflow-hidden fixed top-0 left-0 right-0 z-50 border-b-0 hover:h-10 hover:opacity-100 hover:border-b hover:border-border/50'
        : 'h-10 relative border-b border-border/50'
    "
    :style="{ paddingLeft: leadingInset, paddingRight: trailingInset }"
  >
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
            : 'text-muted-foreground hover:bg-accent/30 hover:text-foreground',
        ]"
        @click="emit('switch-tab', tab.id)"
      >
        <span class="text-sm mr-1.5 shrink-0" :class="tab.icon || 'icon-[mdi--terminal]'"></span>
        <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis text-xs font-medium">{{
          tab.title
        }}</span>

        <button
          v-if="tab.closable !== false"
          class="flex items-center justify-center size-4 ml-1.5 rounded border-none bg-transparent text-muted-foreground transition-all duration-150"
          :class="
            tab.id === activeTabId
              ? 'opacity-60 hover:opacity-100 hover:bg-muted hover:text-foreground'
              : 'opacity-0 group-hover/tab:opacity-60 hover:!opacity-100 hover:bg-accent hover:text-foreground'
          "
          @click.stop="emit('close-tab', tab.id)"
        >
          <span class="icon-[mdi--close] text-xs"></span>
        </button>
      </div>

      <button
        class="shrink-0 flex items-center justify-center size-6 rounded-md border-none bg-transparent text-muted-foreground cursor-pointer transition-all duration-150 hover:bg-accent hover:text-foreground"
        @click="emit('new-tab', '')"
        :title="text('New tab', '新建标签页')"
      >
        <span class="icon-[mdi--plus] text-base"></span>
      </button>
    </div>

    <div class="shrink-0 flex items-center gap-1 px-2 no-drag" :class="contentOffsetClass">
      <Dialog :open="isAppearanceOpen" @update:open="(open) => (isAppearanceOpen = open)">
        <button
          class="flex items-center justify-center size-6 rounded-md border-none bg-transparent text-muted-foreground cursor-pointer transition-all duration-150 hover:bg-accent hover:text-foreground"
          :class="isAppearanceOpen ? 'bg-accent text-foreground' : ''"
          :title="text('Appearance settings', '系统外观设置')"
          @click="toggleAppearanceMenu"
        >
          <span class="icon-[mdi--palette-outline] text-base"></span>
        </button>

        <DialogContent class="tab-appearance-dialog w-72 rounded-2xl border p-4">
          <DialogHeader class="mb-3">
            <DialogTitle class="tab-appearance-title text-base font-medium">{{
              text("Appearance", "外观设置")
            }}</DialogTitle>
          </DialogHeader>

          <div class="space-y-4">
            <div>
              <div class="tab-appearance-label text-xs mb-2">
                {{ text("Theme mode", "主题模式") }}
              </div>
              <div class="grid grid-cols-3 gap-2">
                <button
                  class="tab-appearance-option h-8 rounded-lg text-xs transition-colors border"
                  :class="themeMode === 'light' ? 'is-active' : ''"
                  @click="themeMode = 'light'"
                >
                  {{ text("Light", "浅色") }}
                </button>
                <button
                  class="tab-appearance-option h-8 rounded-lg text-xs transition-colors border"
                  :class="themeMode === 'dark' ? 'is-active' : ''"
                  @click="themeMode = 'dark'"
                >
                  {{ text("Dark", "深色") }}
                </button>
                <button
                  class="tab-appearance-option h-8 rounded-lg text-xs transition-colors border"
                  :class="themeMode === 'auto' ? 'is-active' : ''"
                  @click="themeMode = 'auto'"
                >
                  {{ text("Auto", "自动") }}
                </button>
              </div>
            </div>

            <div>
              <div class="tab-appearance-label text-xs mb-2">
                {{ text("Accent color", "强调色") }}
              </div>
              <div class="flex items-center gap-2">
                <button
                  v-for="color in accentColors"
                  :key="color.value"
                  class="tab-appearance-color size-6 rounded-full border transition-transform hover:scale-110"
                  :class="accentColor === color.value ? 'is-active' : ''"
                  :style="{ backgroundColor: color.value }"
                  :title="text(color.nameEn, color.nameZh)"
                  @click="applyAccentColor(color.value)"
                >
                  <span
                    v-if="accentColor === color.value"
                    class="icon-[mdi--check] text-[10px] text-white"
                  ></span>
                </button>
              </div>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>

    <div class="w-2 h-8 shrink-0" data-tauri-drag-region></div>
  </div>
</template>

<style scoped>
.no-drag {
  -webkit-app-region: no-drag;
}

.tab-appearance-dialog {
  border-color: var(--border-strong, rgba(255, 255, 255, 0.18));
  background: var(--bg-elevated, #1d2430);
  color: var(--text-primary, #f5f7fa);
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.55);
}

.tab-appearance-title {
  color: var(--text-primary, #f5f7fa);
}

.tab-appearance-label {
  color: var(--text-tertiary, #8e97a8);
}

.tab-appearance-option {
  border-color: var(--border-subtle, rgba(255, 255, 255, 0.1));
  color: var(--text-secondary, #c3c9d4);
  background: transparent;
}

.tab-appearance-option:hover {
  background: var(--bg-active, #263042);
  color: var(--text-primary, #f5f7fa);
}

.tab-appearance-option.is-active {
  border-color: var(--accent-color, #0a84ff);
  background: var(--bg-active, #263042);
  color: var(--text-primary, #f5f7fa);
}

.tab-appearance-color {
  border-color: var(--border-subtle, rgba(255, 255, 255, 0.1));
}

.tab-appearance-color.is-active {
  border-color: var(--text-primary, #f5f7fa);
  box-shadow:
    0 0 0 2px var(--bg-elevated, #1d2430),
    0 0 0 3px var(--accent-color, #0a84ff);
}
</style>
