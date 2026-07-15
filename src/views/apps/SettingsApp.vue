<!--
  SettingsApp.vue - 系统设置应用主组件
  极致还原 macOS 系统设置布局
-->
<script setup lang="ts">
import { ref, computed } from "vue";
import GeneralPanel from "./settings/GeneralPanel.vue";
import AppearancePanel from "./settings/AppearancePanel.vue";
import TerminalPanel from "./settings/TerminalPanel.vue";
import ConnectionPanel from "./settings/ConnectionPanel.vue";
import AboutPanel from "./settings/AboutPanel.vue";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

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
const { text } = useInterfaceLanguage();

const panels = computed(() => [
  {
    id: "general" as const,
    label: text("General", "通用"),
    icon: "icon-[lucide--settings-2]",
    color: "from-gray-400 to-gray-500",
    keywords: ["general", "startup", "language", "通用"],
  },
  {
    id: "appearance" as const,
    label: text("Appearance", "外观"),
    icon: "icon-[lucide--palette]",
    color: "from-blue-400 to-blue-500",
    keywords: ["appearance", "theme", "color", "外观"],
  },
  {
    id: "terminal" as const,
    label: text("Terminal", "终端"),
    icon: "icon-[lucide--terminal]",
    color: "from-slate-700 to-slate-800",
    keywords: ["terminal", "font", "终端"],
  },
  {
    id: "connection" as const,
    label: text("Connections", "连接控制"),
    icon: "icon-[lucide--link-2]",
    color: "from-green-400 to-green-500",
    keywords: ["connection", "ssh", "连接"],
  },
  {
    id: "about" as const,
    label: text("About", "关于"),
    icon: "icon-[lucide--info]",
    color: "from-amber-400 to-amber-500",
    keywords: ["about", "version", "关于"],
  },
]);

const currentPanel = computed(() => panels.value.find((p) => p.id === activePanel.value));

const getPanelDescription = (id: PanelId) => {
  const desc: Record<PanelId, string> = {
    general: text(
      "Manage SynapSH startup behavior, interface language, and defaults.",
      "管理 SynapSH 的基础运行模式、界面语言以及启动项配置。",
    ),
    appearance: text(
      "Personalize themes, wallpapers, accent colors, and Dock behavior.",
      "通过调整主题、壁纸与强调色，个性化你的终端界面。",
    ),
    terminal: text(
      "Configure terminal fonts, color schemes, and cursor behavior.",
      "精细化配置终端字体、颜色方案以及光标行为。",
    ),
    connection: text(
      "Manage secure connections, authentication, and session policies.",
      "维护远程服务器的安全连接、代理设置与会话策略。",
    ),
    about: text(
      "View the current SynapSH version, links, and project information.",
      "查看 SynapSH 的当前版本、更新记录以及相关信息。",
    ),
  };
  return desc[id] || "";
};

const filteredPanels = computed(() => {
  if (!searchQuery.value) return panels.value;
  const query = searchQuery.value.toLowerCase();
  return panels.value.filter(
    (p) => p.label.toLowerCase().includes(query) || p.keywords.some((k) => k.includes(query)),
  );
});
</script>

<template>
  <div
    class="flex h-full bg-[#f6f6f6] dark:bg-[#1e1e1e] text-foreground select-none overflow-hidden"
  >
    <!-- macOS 侧边栏 -->
    <aside
      class="w-[260px] flex flex-col bg-[#e9e9e9]/80 dark:bg-[#2d2d2d]/80 border-r border-black/5 dark:border-white/5 backdrop-blur-3xl p-3 shrink-0"
    >
      <!-- 搜索栏 -->
      <div class="px-2 pt-2 pb-4 drag-region" @mousedown="startDrag">
        <div class="relative">
          <span
            class="icon-[lucide--search] absolute left-2.5 top-1/2 -translate-y-1/2 text-tertiary/60 size-3.5 z-10"
          ></span>
          <input
            type="text"
            :placeholder="text('Search settings', '搜索设置')"
            v-model="searchQuery"
            class="w-full h-7 bg-black/5 dark:bg-white/5 border-none rounded-full pl-8 pr-3 text-[11px] outline-none focus:ring-0 placeholder:text-tertiary/40"
          />
        </div>
      </div>

      <!-- 用户概览 -->
      <div class="px-2 mb-4">
        <div
          class="flex items-center gap-3 p-2 rounded-xl hover:bg-black/5 dark:hover:bg-white/5 transition-colors cursor-pointer"
        >
          <div
            class="size-10 rounded-full bg-gradient-to-br from-gray-300 to-gray-500 border border-white/20 shadow-sm flex items-center justify-center overflow-hidden shrink-0"
          >
            <span class="icon-[lucide--user] text-white size-6"></span>
          </div>
          <div class="flex flex-col min-w-0">
            <span class="text-[13px] font-semibold truncate">Admin</span>
            <span class="text-[10px] text-tertiary">{{
              text("System Administrator", "系统管理员")
            }}</span>
          </div>
        </div>
      </div>

      <!-- 侧边栏菜单 -->
      <nav class="flex-1 overflow-y-auto space-y-0.5 px-0.5 custom-scrollbar">
        <template v-if="filteredPanels.length > 0">
          <div
            v-for="panel in filteredPanels"
            :key="panel.id"
            class="flex items-center gap-2.5 px-2.5 py-1.5 rounded-lg cursor-pointer text-[12px] transition-all duration-75 group"
            :class="[
              activePanel === panel.id
                ? 'bg-[#007aff] text-white'
                : 'text-primary hover:bg-black/5 dark:hover:bg-white/5',
            ]"
            @click="activePanel = panel.id"
          >
            <!-- 图标容器 -->
            <div
              class="size-6 rounded-md flex items-center justify-center shrink-0 shadow-sm"
              :class="[
                panel.color,
                activePanel === panel.id ? 'opacity-100' : 'opacity-90 group-hover:opacity-100',
              ]"
            >
              <span :class="panel.icon" class="size-3.5 text-white"></span>
            </div>
            <span class="font-medium">{{ panel.label }}</span>
          </div>
        </template>
        <div v-else class="px-4 py-8 text-center text-tertiary text-[10px]">
          {{ text("No settings found", "未找到内容") }}
        </div>
      </nav>
    </aside>

    <!-- macOS 内容区域 -->
    <main class="flex-1 flex flex-col min-w-0 overflow-hidden bg-white/40 dark:bg-black/10">
      <!-- 居向概览 -->
      <header class="pt-10 pb-6 flex flex-col items-center shrink-0">
        <div
          class="size-16 rounded-[18px] flex items-center justify-center shadow-2xl mb-3 border border-white/10"
          :class="currentPanel?.color"
        >
          <span :class="currentPanel?.icon" class="size-9 text-white drop-shadow-md"></span>
        </div>
        <h1 class="text-[19px] font-bold tracking-tight text-primary">{{ currentPanel?.label }}</h1>
        <p class="text-[11px] text-tertiary mt-1.5 px-10 text-center max-w-[420px] leading-relaxed">
          {{ getPanelDescription(activePanel) }}
        </p>
      </header>

      <!-- 实际滚动内容 -->
      <div class="flex-1 overflow-y-auto w-full px-6 custom-scrollbar">
        <div class="max-w-[640px] mx-auto pb-12">
          <GeneralPanel v-if="activePanel === 'general'" />
          <AppearancePanel v-else-if="activePanel === 'appearance'" />
          <TerminalPanel v-else-if="activePanel === 'terminal'" />
          <ConnectionPanel v-else-if="activePanel === 'connection'" />
          <AboutPanel v-else-if="activePanel === 'about'" />
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.drag-region {
  -webkit-app-region: drag;
}

.custom-scrollbar::-webkit-scrollbar {
  width: 5px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 0, 0, 0.1);
  border-radius: 10px;
}
.dark .custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
}
</style>
