<!--
  ConnectionPanel.vue - SSH 连接配置面板
  用于配置 SSH 连接参数并建立连接
-->
<script setup lang="ts">
import { ref, computed } from "vue";
import { api } from "@/lib/api";

const props = defineProps<{
  isConnected: boolean;
}>();

const emit = defineEmits<{
  connected: [sessionId: string];
  disconnected: [];
}>();

const host = ref("");
const port = ref(22);
const username = ref("");
const password = ref("");
const authType = ref<"password" | "key">("password");
const privateKey = ref("");
const isConnecting = ref(false);
const error = ref("");

const canConnect = computed(() => {
  return host.value && username.value && (authType.value === "password" ? password.value : privateKey.value);
});

async function connect() {
  console.log("=== 连接按钮被点击 ===");
  console.log("canConnect:", canConnect.value);
  console.log("host:", host.value);
  console.log("port:", port.value);
  console.log("username:", username.value);
  
  if (!canConnect.value) {
    console.log("条件不满足，无法连接");
    return;
  }
  
  isConnecting.value = true;
  error.value = "";
  
  try {
    const sessionId = `session_${Date.now()}`;
    console.log("生成的 sessionId:", sessionId);
    
    const params = {
      host: host.value,
      port: port.value,
      username: username.value,
      password: authType.value === "password" ? password.value : undefined,
      privateKey: authType.value === "key" ? privateKey.value : undefined,
    };
    console.log("调用 connectSSH，参数:", params);
    
    await api.connectSSH(sessionId, params);
    
    console.log("连接成功！");
    emit("connected", sessionId);
  } catch (e) {
    console.error("连接失败:", e);
    error.value = String(e);
  } finally {
    isConnecting.value = false;
  }
}

async function disconnect() {
  try {
    emit("disconnected");
  } catch (e) {
    console.error("Disconnect error:", e);
  }
}

function selectKeyFile() {
  privateKey.value = "/path/to/key";
}
</script>

<template>
  <div class="flex flex-col h-full">
    <div class="px-4 py-4 border-b border-border">
      <h2 class="text-base font-semibold text-foreground">连接配置</h2>
    </div>

    <div class="p-4 flex flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground font-medium">主机地址</label>
        <input v-model="host" type="text" placeholder="例如: 192.168.1.1" :disabled="isConnected"
          class="px-3 py-2 bg-muted border border-border rounded-md text-foreground text-sm outline-none focus:border-primary transition-colors disabled:opacity-60 disabled:cursor-not-allowed" />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground font-medium">端口</label>
        <input v-model.number="port" type="number" :disabled="isConnected"
          class="px-3 py-2 bg-muted border border-border rounded-md text-foreground text-sm outline-none focus:border-primary transition-colors disabled:opacity-60 disabled:cursor-not-allowed" />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground font-medium">用户名</label>
        <input v-model="username" type="text" placeholder="root" :disabled="isConnected"
          class="px-3 py-2 bg-muted border border-border rounded-md text-foreground text-sm outline-none focus:border-primary transition-colors disabled:opacity-60 disabled:cursor-not-allowed" />
      </div>

      <div class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground font-medium">认证方式</label>
        <div class="flex gap-2">
          <button :class="authType === 'password' ? 'bg-primary border-primary text-primary-foreground' : 'bg-muted border-border text-muted-foreground'"
            @click="authType = 'password'" :disabled="isConnected"
            class="flex-1 px-3 py-2 border rounded-md text-sm cursor-pointer transition-all disabled:opacity-60 disabled:cursor-not-allowed">
            密码
          </button>
          <button :class="authType === 'key' ? 'bg-primary border-primary text-primary-foreground' : 'bg-muted border-border text-muted-foreground'"
            @click="authType = 'key'" :disabled="isConnected"
            class="flex-1 px-3 py-2 border rounded-md text-sm cursor-pointer transition-all disabled:opacity-60 disabled:cursor-not-allowed">
            密钥
          </button>
        </div>
      </div>

      <div v-if="authType === 'password'" class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground font-medium">密码</label>
        <input v-model="password" type="password" placeholder="输入密码" :disabled="isConnected"
          class="px-3 py-2 bg-muted border border-border rounded-md text-foreground text-sm outline-none focus:border-primary transition-colors disabled:opacity-60 disabled:cursor-not-allowed" />
      </div>

      <div v-else class="flex flex-col gap-1.5">
        <label class="text-sm text-muted-foreground font-medium">私钥路径</label>
        <div class="flex gap-2">
          <input v-model="privateKey" type="text" placeholder="选择私钥文件" readonly :disabled="isConnected"
            class="flex-1 px-3 py-2 bg-muted border border-border rounded-md text-foreground text-sm disabled:opacity-60 disabled:cursor-not-allowed" />
          <button @click="selectKeyFile" :disabled="isConnected"
            class="px-3 py-2 bg-muted border border-border rounded-md text-foreground text-sm cursor-pointer hover:border-primary transition-all disabled:opacity-60 disabled:cursor-not-allowed">
            浏览
          </button>
        </div>
      </div>

      <div v-if="error" class="px-3 py-2.5 bg-destructive/10 border border-destructive rounded-md text-destructive text-sm">
        {{ error }}
      </div>

      <div class="mt-auto pt-4 border-t border-border">
        <button v-if="!isConnected" :disabled="!canConnect || isConnecting" @click="connect"
          class="w-full px-4 py-2.5 bg-primary text-primary-foreground rounded-md text-sm font-medium cursor-pointer hover:bg-primary/90 transition-colors disabled:opacity-60 disabled:cursor-not-allowed">
          {{ isConnecting ? '连接中...' : '连接' }}
        </button>
        <button v-else @click="disconnect"
          class="w-full px-4 py-2.5 bg-destructive text-destructive-foreground rounded-md text-sm font-medium cursor-pointer hover:bg-destructive/90 transition-colors">
          断开连接
        </button>
      </div>
    </div>
  </div>
</template>
