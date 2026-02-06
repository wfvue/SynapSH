<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

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
      password: authType.value === "password" ? password.value : null,
      privateKey: authType.value === "key" ? privateKey.value : null,
    };
    console.log("调用 connect_ssh，参数:", params);
    
    await invoke("connect_ssh", {
      sessionId,
      params,
    });
    
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
  <div class="connection-panel">
    <div class="panel-header">
      <h2>连接配置</h2>
    </div>
    
    <div class="panel-content">
      <div class="form-group">
        <label>主机地址</label>
        <input 
          v-model="host" 
          type="text" 
          placeholder="例如: 192.168.1.1"
          :disabled="isConnected"
        />
      </div>
      
      <div class="form-group">
        <label>端口</label>
        <input 
          v-model.number="port" 
          type="number" 
          :disabled="isConnected"
        />
      </div>
      
      <div class="form-group">
        <label>用户名</label>
        <input 
          v-model="username" 
          type="text" 
          placeholder="root"
          :disabled="isConnected"
        />
      </div>
      
      <div class="form-group">
        <label>认证方式</label>
        <div class="auth-type">
          <button 
            :class="{ active: authType === 'password' }"
            @click="authType = 'password'"
            :disabled="isConnected"
          >
            密码
          </button>
          <button 
            :class="{ active: authType === 'key' }"
            @click="authType = 'key'"
            :disabled="isConnected"
          >
            密钥
          </button>
        </div>
      </div>
      
      <div v-if="authType === 'password'" class="form-group">
        <label>密码</label>
        <input 
          v-model="password" 
          type="password" 
          placeholder="输入密码"
          :disabled="isConnected"
        />
      </div>
      
      <div v-else class="form-group">
        <label>私钥路径</label>
        <div class="key-input">
          <input 
            v-model="privateKey" 
            type="text" 
            placeholder="选择私钥文件"
            readonly
            :disabled="isConnected"
          />
          <button 
            @click="selectKeyFile"
            :disabled="isConnected"
          >
            浏览
          </button>
        </div>
      </div>
      
      <div v-if="error" class="error-message">
        {{ error }}
      </div>
      
      <div class="actions">
        <button 
          v-if="!isConnected"
          class="btn-primary"
          :disabled="!canConnect || isConnecting"
          @click="connect"
        >
          {{ isConnecting ? '连接中...' : '连接' }}
        </button>
        <button 
          v-else
          class="btn-danger"
          @click="disconnect"
        >
          断开连接
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.connection-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.panel-header {
  padding: 16px;
  border-bottom: 1px solid var(--border);
}

.panel-header h2 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.panel-content {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 0.85rem;
  color: var(--text-secondary);
  font-weight: 500;
}

.form-group input {
  padding: 8px 12px;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 0.9rem;
  transition: border-color 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: var(--accent);
}

.form-group input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.auth-type {
  display: flex;
  gap: 8px;
}

.auth-type button {
  flex: 1;
  padding: 8px 12px;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
}

.auth-type button.active {
  background-color: var(--accent);
  border-color: var(--accent);
  color: white;
}

.auth-type button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.key-input {
  display: flex;
  gap: 8px;
}

.key-input input {
  flex: 1;
}

.key-input button {
  padding: 8px 12px;
  background-color: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 4px;
  color: var(--text-primary);
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
}

.key-input button:hover:not(:disabled) {
  border-color: var(--accent);
}

.error-message {
  padding: 10px 12px;
  background-color: rgba(244, 135, 113, 0.1);
  border: 1px solid var(--error);
  border-radius: 4px;
  color: var(--error);
  font-size: 0.85rem;
}

.actions {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid var(--border);
}

.actions button {
  width: 100%;
  padding: 10px 16px;
  border: none;
  border-radius: 4px;
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background-color: var(--accent);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background-color: #005a9e;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-danger {
  background-color: var(--error);
  color: white;
}

.btn-danger:hover {
  background-color: #d66b5a;
}
</style>
