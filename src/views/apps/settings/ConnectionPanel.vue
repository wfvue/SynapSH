<!--
  ConnectionPanel.vue - 连接设置面板
  设置 SSH 连接默认配置
-->
<script setup lang="ts">
import { computed, ref } from "vue";
import { Switch } from "@/components/ui/switch";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

// 连接设置项
const timeout = ref(30);
const keepAlive = ref(60);
const compression = ref(false);
const sshVersion = ref("2");
const authMethod = ref("password");
const { text } = useInterfaceLanguage();

const authMethods = computed(() => [
  { value: "password", label: text("Password", "密码"), icon: "icon-[lucide--key]" },
  { value: "publickey", label: text("Key", "密钥"), icon: "icon-[lucide--file-key-2]" },
  {
    value: "both",
    label: text("Password + Key", "密码 + 密钥"),
    icon: "icon-[lucide--shield-check]",
  },
]);
</script>

<template>
  <div class="space-y-7 animate-in fade-in duration-300 pb-8">
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Authentication", "身份验证")
        }}</span>
      </div>

      <div class="flex gap-4">
        <div
          v-for="method in authMethods"
          :key="method.value"
          class="flex-1 flex flex-col items-center gap-2 p-3 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl cursor-pointer transition-all shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
          :class="
            authMethod === method.value
              ? 'ring-2 ring-accent/50 border-accent/20 bg-accent/5'
              : 'hover:border-black/10 dark:hover:border-white/10'
          "
          @click="authMethod = method.value"
        >
          <span
            :class="method.icon"
            class="size-6"
            :style="{
              color: authMethod === method.value ? '#007aff' : 'currentColor',
              opacity: authMethod === method.value ? 1 : 0.7,
            }"
          ></span>
          <span class="text-[13px] font-medium text-primary">{{ method.label }}</span>
        </div>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Timeout & Connection", "超时与连接")
        }}</span>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl overflow-hidden divide-y divide-black/5 dark:divide-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-medium text-primary">{{
              text("Connection timeout (seconds)", "连接超时（秒）")
            }}</span>
            <span class="text-[11px] text-tertiary">{{
              text("Maximum time to wait for a server response", "等待服务器响应的最长时间")
            }}</span>
          </div>
          <input
            type="number"
            v-model="timeout"
            min="5"
            max="120"
            class="w-[70px] bg-black/5 dark:bg-white/10 border-none rounded-[6px] px-2 py-1 text-[12px] text-center focus:ring-2 focus:ring-accent/50 outline-none text-primary"
          />
        </div>

        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-medium text-primary">{{
              text("Keep-alive interval (seconds)", "心跳间隔（秒）")
            }}</span>
            <span class="text-[11px] text-tertiary">{{
              text("Interval between keep-alive messages", "保持连接活跃的心跳发送间隔")
            }}</span>
          </div>
          <input
            type="number"
            v-model="keepAlive"
            min="0"
            max="300"
            class="w-[70px] bg-black/5 dark:bg-white/10 border-none rounded-[6px] px-2 py-1 text-[12px] text-center focus:ring-2 focus:ring-accent/50 outline-none text-primary"
          />
        </div>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Advanced Controls", "高级控制")
        }}</span>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl overflow-hidden divide-y divide-black/5 dark:divide-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <div class="flex flex-col gap-0.5">
            <span class="text-[13px] font-medium text-primary">{{
              text("Data compression", "数据压缩")
            }}</span>
            <span class="text-[11px] text-tertiary">{{
              text("Compress transferred data to reduce bandwidth", "压缩传输数据以节省带宽")
            }}</span>
          </div>
          <Switch v-model:checked="compression" />
        </div>

        <div
          class="flex items-center justify-between p-3.5 px-4 hover:bg-black/[0.02] dark:hover:bg-white/[0.02] transition-colors"
        >
          <span class="text-[13px] font-medium text-primary">{{
            text("SSH protocol version", "SSH 协议版本")
          }}</span>
          <div
            class="flex bg-black/5 dark:bg-white/10 rounded-[6px] p-0.5 border border-black/5 dark:border-white/5"
          >
            <label
              class="px-3 py-1 rounded-[4px] cursor-pointer transition-all text-[12px]"
              :class="
                sshVersion === '2'
                  ? 'bg-white dark:bg-white/20 shadow-sm font-semibold'
                  : 'hover:bg-black/5 dark:hover:bg-white/5'
              "
            >
              <input type="radio" v-model="sshVersion" value="2" class="hidden" />
              SSH 2
            </label>
            <label
              class="px-3 py-1 rounded-[4px] cursor-pointer transition-all text-[12px]"
              :class="
                sshVersion === '1'
                  ? 'bg-white dark:bg-white/20 shadow-sm font-semibold'
                  : 'hover:bg-black/5 dark:hover:bg-white/5'
              "
            >
              <input type="radio" v-model="sshVersion" value="1" class="hidden" />
              SSH 1
            </label>
          </div>
        </div>
      </div>
    </section>

    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Authentication Keys", "认证密钥")
        }}</span>
      </div>

      <div class="flex flex-col gap-2">
        <div
          class="flex items-center gap-3 p-3 bg-white dark:bg-white/5 rounded-xl border border-black/5 dark:border-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
        >
          <div class="size-8 rounded-full bg-amber-500/10 flex items-center justify-center">
            <span class="icon-[lucide--key] size-4 text-amber-500"></span>
          </div>
          <div class="flex-1 flex flex-col">
            <span class="text-[13px] font-medium text-primary">id_rsa</span>
            <span class="text-[11px] text-tertiary font-mono">~/.ssh/id_rsa</span>
          </div>
          <span
            class="px-2.5 py-0.5 text-[11px] font-medium bg-green-500/15 text-green-600 dark:text-green-400 rounded-full"
            >{{ text("Added", "已添加") }}</span
          >
        </div>

        <button
          class="flex items-center justify-center gap-1.5 p-3 bg-transparent border border-dashed border-black/10 dark:border-white/10 rounded-xl text-tertiary hover:bg-black/[0.02] dark:hover:bg-white/[0.02] hover:text-primary transition-all shadow-sm"
        >
          <span class="icon-[lucide--plus] size-4"></span>
          <span class="text-[13px] font-medium">{{ text("Add local key", "添加本地密钥") }}</span>
        </button>
      </div>
    </section>
  </div>
</template>
