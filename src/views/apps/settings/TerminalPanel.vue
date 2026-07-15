<!--
  TerminalPanel.vue - 终端设置面板
  设置终端字体、配色方案、光标样式等
-->
<script setup lang="ts">
import { ref } from "vue";
import { Switch } from "@/components/ui/switch";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

// 终端设置项
const fontSize = ref(14);
const fontFamily = ref("JetBrains Mono");
const cursorStyle = ref<"block" | "underline" | "bar">("block");
const cursorBlink = ref(true);
const colorScheme = ref("dracula");
const scrollback = ref(1000);
const { text } = useInterfaceLanguage();

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
  <div class="space-y-7 animate-in fade-in duration-300 pb-8">
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{ text("Font", "字体") }}</span>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl overflow-hidden divide-y divide-black/5 dark:divide-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <span class="text-[13px] font-medium text-primary">{{
            text("Font family", "字体族")
          }}</span>
          <div class="relative group">
            <select
              v-model="fontFamily"
              class="appearance-none bg-black/5 dark:bg-white/10 border-0 rounded-[6px] pl-3 pr-8 py-1 text-[12px] outline-none text-primary min-w-[140px] transition-all cursor-pointer"
            >
              <option
                v-for="font in fonts"
                :key="font.value"
                :value="font.value"
                class="bg-background text-primary"
              >
                {{ font.label }}
              </option>
            </select>
            <span
              class="icon-[lucide--chevron-down] absolute right-2.5 top-1/2 -translate-y-1/2 size-3.5 text-tertiary pointer-events-none"
            ></span>
          </div>
        </div>

        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <span class="text-[13px] font-medium text-primary">{{
            text("Font size", "字体大小")
          }}</span>
          <div
            class="flex items-center gap-1 bg-black/5 dark:bg-white/10 rounded-[6px] px-1 py-0.5"
          >
            <button
              @click="fontSize = Math.max(10, fontSize - 1)"
              class="size-5 flex items-center justify-center rounded hover:bg-black/10 dark:hover:bg-white/20 transition-colors"
            >
              <span class="icon-[lucide--minus] size-3"></span>
            </button>
            <span class="w-10 text-center text-[12px] font-mono">{{ fontSize }}px</span>
            <button
              @click="fontSize = Math.min(36, fontSize + 1)"
              class="size-5 flex items-center justify-center rounded hover:bg-black/10 dark:hover:bg-white/20 transition-colors"
            >
              <span class="icon-[lucide--plus] size-3"></span>
            </button>
          </div>
        </div>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{ text("Cursor", "光标") }}</span>
      </div>

      <div class="flex gap-4 mb-4">
        <div
          class="flex-1 flex flex-col items-center gap-2 p-3 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl cursor-pointer transition-all shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
          :class="{
            'ring-2 ring-accent/50 border-accent/20 bg-accent/5': cursorStyle === 'block',
            'hover:border-black/10 dark:hover:border-white/10': cursorStyle !== 'block',
          }"
          @click="cursorStyle = 'block'"
        >
          <div
            class="w-10 h-6 bg-[#1e1e2e] rounded shadow-sm relative flex items-end justify-center pb-1"
          >
            <div class="w-2.5 h-4 bg-gray-200"></div>
          </div>
          <span class="text-[11px] text-tertiary">{{ text("Block", "方块") }}</span>
        </div>
        <div
          class="flex-1 flex flex-col items-center gap-2 p-3 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl cursor-pointer transition-all shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
          :class="{
            'ring-2 ring-accent/50 border-accent/20 bg-accent/5': cursorStyle === 'underline',
            'hover:border-black/10 dark:hover:border-white/10': cursorStyle !== 'underline',
          }"
          @click="cursorStyle = 'underline'"
        >
          <div
            class="w-10 h-6 bg-[#1e1e2e] rounded shadow-sm relative flex items-end justify-center pb-1"
          >
            <div class="w-2.5 h-0.5 bg-gray-200"></div>
          </div>
          <span class="text-[11px] text-tertiary">{{ text("Underline", "下划线") }}</span>
        </div>
        <div
          class="flex-1 flex flex-col items-center gap-2 p-3 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl cursor-pointer transition-all shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
          :class="{
            'ring-2 ring-accent/50 border-accent/20 bg-accent/5': cursorStyle === 'bar',
            'hover:border-black/10 dark:hover:border-white/10': cursorStyle !== 'bar',
          }"
          @click="cursorStyle = 'bar'"
        >
          <div
            class="w-10 h-6 bg-[#1e1e2e] rounded shadow-sm relative flex items-end justify-center pb-1"
          >
            <div class="w-0.5 h-4 bg-gray-200"></div>
          </div>
          <span class="text-[11px] text-tertiary">{{ text("Bar", "竖线") }}</span>
        </div>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-3.5 px-4 flex items-center justify-between shadow-[0_1px_3px_rgba(0,0,0,0.02)] hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
      >
        <span class="text-[13px] font-medium text-primary">{{
          text("Blinking cursor", "光标闪烁")
        }}</span>
        <Switch v-model:checked="cursorBlink" />
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Color Scheme", "配色方案")
        }}</span>
      </div>

      <div class="grid grid-cols-2 lg:grid-cols-3 gap-3">
        <div
          v-for="scheme in colorSchemes"
          :key="scheme.value"
          class="flex flex-col gap-2 p-3 border rounded-xl cursor-pointer transition-all shadow-[0_1px_3px_rgba(0,0,0,0.02)] bg-white dark:bg-white/5"
          :class="
            colorScheme === scheme.value
              ? 'ring-2 ring-accent/50 border-accent/20 bg-accent/5'
              : 'border-black/5 dark:border-white/5 hover:border-black/10 dark:hover:border-white/10'
          "
          @click="colorScheme = scheme.value"
        >
          <div
            class="rounded-lg p-3 font-mono text-[11px] shadow-inner"
            :style="{ background: scheme.bg }"
          >
            <span :style="{ color: scheme.fg }">$ echo</span>
            <span :style="{ color: scheme.accent }"> "Hello"</span>
          </div>
          <span class="text-center text-[12px] font-medium text-primary">{{ scheme.label }}</span>
        </div>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Advanced Options", "高级选项")
        }}</span>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-3.5 px-4 flex items-center justify-between shadow-[0_1px_3px_rgba(0,0,0,0.02)] hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
      >
        <div class="flex flex-col gap-0.5">
          <span class="text-[13px] font-medium text-primary">{{
            text("Scrollback lines", "回滚行数")
          }}</span>
          <span class="text-[11px] text-tertiary">{{
            text("Number of terminal history lines to retain", "终端保留的历史行数")
          }}</span>
        </div>
        <div class="relative group">
          <select
            v-model="scrollback"
            class="appearance-none bg-black/5 dark:bg-white/10 border-0 rounded-[6px] pl-3 pr-8 py-1 text-[12px] outline-none text-primary min-w-[120px] transition-all cursor-pointer"
          >
            <option :value="500" class="bg-background">500 {{ text("lines", "行") }}</option>
            <option :value="1000" class="bg-background">1000 {{ text("lines", "行") }}</option>
            <option :value="5000" class="bg-background">5000 {{ text("lines", "行") }}</option>
            <option :value="10000" class="bg-background">10000 {{ text("lines", "行") }}</option>
          </select>
          <span
            class="icon-[lucide--chevron-down] absolute right-2.5 top-1/2 -translate-y-1/2 size-3.5 text-tertiary pointer-events-none"
          ></span>
        </div>
      </div>
    </section>
  </div>
</template>
