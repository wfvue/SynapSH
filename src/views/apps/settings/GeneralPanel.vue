<!--
  GeneralPanel.vue - 通用设置面板
  设置启动行为、默认应用等通用配置
-->
<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { Switch } from "@/components/ui/switch";

// 通用设置项
const autoConnect = ref(true);
const showWelcome = ref(true);
const language = ref("zh-CN");
const defaultShell = ref("bash");

// 从数据库加载设置
onMounted(async () => {
  autoConnect.value = await window.electronAPI.getSetting("general:autoConnect", true);
  showWelcome.value = await window.electronAPI.getSetting("general:showWelcome", true);
  language.value = await window.electronAPI.getSetting("general:language", "zh-CN");
  defaultShell.value = await window.electronAPI.getSetting("general:defaultShell", "bash");
});

// 监听变动并保存
watch(autoConnect, (val) => window.electronAPI.setSetting("general:autoConnect", val));
watch(showWelcome, (val) => window.electronAPI.setSetting("general:showWelcome", val));
watch(language, (val) => window.electronAPI.setSetting("general:language", val));
watch(defaultShell, (val) => window.electronAPI.setSetting("general:defaultShell", val));

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
  <div class="space-y-7 animate-in fade-in duration-300 pb-8">
    <!-- 启动部分 -->
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">启动行为</span>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl overflow-hidden divide-y divide-black/5 dark:divide-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-medium text-primary">启动时自动连接</span>
            <span class="text-[11px] text-tertiary">同步应用开启后自动恢复上次的 SSH 会话</span>
          </div>
          <Switch v-model:checked="autoConnect" />
        </div>

        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-medium text-primary">显示欢迎界面</span>
            <span class="text-[11px] text-tertiary">启动时展示快速上手指南</span>
          </div>
          <Switch v-model:checked="showWelcome" />
        </div>
      </div>
    </section>

    <!-- 语言部分 -->
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">语言与区域</span>
      </div>
      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-3.5 px-4 flex items-center justify-between shadow-[0_1px_3px_rgba(0,0,0,0.02)] hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
      >
        <span class="text-[13px] font-medium text-primary">界面语言</span>
        <div class="relative group">
          <select
            v-model="language"
            class="appearance-none bg-black/5 dark:bg-white/10 border-0 rounded-[6px] pl-3 pr-8 py-1 text-[12px] outline-none text-primary min-w-[120px] transition-all cursor-pointer"
          >
            <option
              v-for="lang in languages"
              :key="lang.value"
              :value="lang.value"
              class="bg-background text-primary"
            >
              {{ lang.label }}
            </option>
          </select>
          <span
            class="icon-[lucide--chevron-down] absolute right-2.5 top-1/2 -translate-y-1/2 size-3.5 text-tertiary pointer-events-none"
          ></span>
        </div>
      </div>
    </section>

    <!-- 默认设置部分 -->
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">终端默认设置</span>
      </div>
      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-3.5 px-4 flex items-center justify-between shadow-[0_1px_3px_rgba(0,0,0,0.02)] hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
      >
        <div class="flex flex-col gap-0.5">
          <span class="text-[13px] font-medium text-primary">默认 Shell</span>
          <span class="text-[11px] text-tertiary">创建新终端连接时默认使用的 Shell 环境</span>
        </div>
        <div class="relative group">
          <select
            v-model="defaultShell"
            class="appearance-none bg-black/5 dark:bg-white/10 border-0 rounded-[6px] pl-3 pr-8 py-1 text-[12px] outline-none text-primary min-w-[120px] transition-all cursor-pointer"
          >
            <option
              v-for="shell in shells"
              :key="shell.value"
              :value="shell.value"
              class="bg-background text-primary"
            >
              {{ shell.label }}
            </option>
          </select>
          <span
            class="icon-[lucide--chevron-down] absolute right-2.5 top-1/2 -translate-y-1/2 size-3.5 text-tertiary pointer-events-none"
          ></span>
        </div>
      </div>
    </section>
  </div>
</template>
