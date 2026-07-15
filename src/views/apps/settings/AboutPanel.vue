<!--
  AboutPanel.vue - 关于面板
  显示应用版本信息和系统信息
-->
<script setup lang="ts">
import { computed, ref } from "vue";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

const appVersion = ref("1.0.0");
const buildDate = "2026-02-07";
const { language, text } = useInterfaceLanguage();
const introductionLanguage = computed<"en" | "zh">({
  get: () => (language.value === "en-US" ? "en" : "zh"),
  set: (value) => {
    language.value = value === "en" ? "en-US" : "zh-CN";
  },
});

const introductions = {
  en: {
    title: "About SynapSH",
    content:
      "SynapSH is a modern SSH client built around a desktop-inspired workspace. It brings terminals, file management, system monitoring, and everyday server operations together in one focused environment.",
  },
  zh: {
    title: "关于 SynapSH",
    content:
      "SynapSH 是一款采用桌面化工作空间设计的现代 SSH 客户端。它将终端、文件管理、系统监控和日常服务器操作集中在一个专注、高效的环境中。",
  },
} as const;

const links = computed(() => [
  {
    label: text("GitHub Repository", "GitHub 仓库"),
    url: "https://github.com",
    icon: "icon-[lucide--github]",
  },
  { label: text("Send Feedback", "提交反馈"), url: "#", icon: "icon-[lucide--message-square]" },
  { label: text("Documentation", "文档"), url: "#", icon: "icon-[lucide--book-open]" },
]);
</script>

<template>
  <div class="space-y-7 animate-in fade-in duration-300 pb-8">
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Version Information", "版本信息")
        }}</span>
      </div>
      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl overflow-hidden divide-y divide-black/5 dark:divide-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <span class="text-[13px] font-medium text-primary">{{
            text("Current version", "当前版本")
          }}</span>
          <span
            class="px-2.5 py-0.5 bg-blue-500/10 text-blue-600 dark:text-blue-400 font-mono text-[11px] rounded-full"
            >v{{ appVersion }}</span
          >
        </div>
        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <span class="text-[13px] font-medium text-primary">{{
            text("Build date", "构建日期")
          }}</span>
          <span class="text-[12px] text-tertiary font-mono">{{ buildDate }}</span>
        </div>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2 flex min-h-8 items-center justify-between gap-3">
        <span class="text-[12px] font-semibold text-tertiary">
          {{ introductions[introductionLanguage].title }}
        </span>
        <div
          class="flex rounded-lg border border-black/5 bg-black/[0.03] p-0.5 dark:border-white/10 dark:bg-white/5"
          role="group"
          aria-label="Introduction language"
        >
          <button
            v-for="language in [
              { value: 'en', label: 'English' },
              { value: 'zh', label: '中文' },
            ] as const"
            :key="language.value"
            type="button"
            class="min-h-7 rounded-md px-2.5 text-[11px] font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]"
            :class="
              introductionLanguage === language.value
                ? 'bg-white text-primary shadow-sm dark:bg-white/10'
                : 'text-tertiary hover:text-primary'
            "
            :aria-pressed="introductionLanguage === language.value"
            @click="introductionLanguage = language.value"
          >
            {{ language.label }}
          </button>
        </div>
      </div>
      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-4 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <p class="text-[12px] leading-relaxed text-tertiary" aria-live="polite">
          {{ introductions[introductionLanguage].content }}
        </p>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Support & Links", "支持与链接")
        }}</span>
      </div>
      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl overflow-hidden divide-y divide-black/5 dark:divide-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <a
          v-for="link in links"
          :key="link.label"
          :href="link.url"
          class="flex items-center gap-3 p-3.5 px-4 bg-transparent hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors group cursor-pointer"
          target="_blank"
        >
          <span
            :class="link.icon"
            class="size-4 text-tertiary group-hover:text-primary transition-colors"
          ></span>
          <span class="flex-1 text-[13px] font-medium text-primary">{{ link.label }}</span>
          <span
            class="icon-[lucide--chevron-right] size-4 text-black/20 dark:text-white/20 group-hover:text-primary transition-colors"
          ></span>
        </a>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Core Technologies", "核心技术栈")
        }}</span>
      </div>
      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-4 flex flex-wrap gap-2 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 bg-black/5 dark:bg-white/10 rounded-md border border-black/5 dark:border-white/5"
        >
          <span class="icon-[lucide--code-2] size-3.5 text-[#42b883]"></span>
          <span class="text-[11px] font-medium text-primary">Vue 3</span>
        </div>
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 bg-black/5 dark:bg-white/10 rounded-md border border-black/5 dark:border-white/5"
        >
          <span class="icon-[lucide--monitor-smartphone] size-3.5 text-[#47848f]"></span>
          <span class="text-[11px] font-medium text-primary">Electron</span>
        </div>
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 bg-black/5 dark:bg-white/10 rounded-md border border-black/5 dark:border-white/5"
        >
          <span class="icon-[lucide--server] size-3.5 text-[#68a063]"></span>
          <span class="text-[11px] font-medium text-primary">Node.js</span>
        </div>
        <div
          class="flex items-center gap-1.5 px-2.5 py-1 bg-black/5 dark:bg-white/10 rounded-md border border-black/5 dark:border-white/5"
        >
          <span class="icon-[lucide--file-code-2] size-3.5 text-[#3178c6]"></span>
          <span class="text-[11px] font-medium text-primary">TypeScript</span>
        </div>
      </div>
    </section>

    <footer class="mt-8 pt-8 text-center text-[11px] text-tertiary/60">
      <p class="mb-1">Made with ❤️ for developers</p>
      <p>Copyright © 2026 SynapSH Team. All rights reserved.</p>
    </footer>
  </div>
</template>
