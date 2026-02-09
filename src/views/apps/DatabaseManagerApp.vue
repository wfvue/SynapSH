<!-- 数据库管理应用 - 远程服务器数据库的安装、配置和管理 -->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../../components/ui/toast";

// ==================== 类型定义 ====================

type DatabaseType = "mysql" | "postgresql" | "redis" | "mongodb" | "mariadb";

type DatabaseStatus = "running" | "stopped" | "error" | "notInstalled" | "installing" | "unknown";

interface DatabaseInfo {
  dbType: DatabaseType;
  installed: boolean;
  version: string | null;
  status: DatabaseStatus;
  port: number | null;
  installPath: string | null;
}

interface InstallOptions {
  version?: string;
  port?: number;
  rootPassword?: string;
  dataPath?: string;
}

// ==================== Props ====================

const props = defineProps<{
  sessionId: string;
}>();

// ==================== 状态 ====================

const { toast } = useToast();

const databases = ref<DatabaseInfo[]>([]);
const loading = ref(false);
const scanning = ref(false);
const selectedDb = ref<DatabaseInfo | null>(null);
const configContent = ref("");
const showInstallModal = ref(false);
const showConfigModal = ref(false);
const installingDb = ref<DatabaseType | null>(null);
const installOptions = ref<InstallOptions>({
  port: undefined,
  rootPassword: "",
});
const installLog = ref("");

// ==================== 常量 ====================

const dbTypeConfig: Record<DatabaseType, { name: string; icon: string; color: string; defaultPort: number; description: string }> = {
  mysql: {
    name: "MySQL",
    icon: "icon-[mdi--database]",
    color: "#00758f",
    defaultPort: 3306,
    description: "最流行的开源关系型数据库",
  },
  postgresql: {
    name: "PostgreSQL",
    icon: "icon-[mdi--database]",
    color: "#336791",
    defaultPort: 5432,
    description: "功能强大的开源对象关系型数据库",
  },
  redis: {
    name: "Redis",
    icon: "icon-[mdi--database]",
    color: "#dc382d",
    defaultPort: 6379,
    description: "高性能键值对存储数据库",
  },
  mongodb: {
    name: "MongoDB",
    icon: "icon-[mdi--database]",
    color: "#47a248",
    defaultPort: 27017,
    description: "面向文档的 NoSQL 数据库",
  },
  mariadb: {
    name: "MariaDB",
    icon: "icon-[mdi--database]",
    color: "#003545",
    defaultPort: 3306,
    description: "MySQL 的兼容替代品",
  },
};

const statusConfig: Record<DatabaseStatus, { label: string; color: string; icon: string }> = {
  running: { label: "运行中", color: "#22c55e", icon: "icon-[mdi--check-circle]" },
  stopped: { label: "已停止", color: "#f59e0b", icon: "icon-[mdi--pause-circle]" },
  error: { label: "错误", color: "#ef4444", icon: "icon-[mdi--alert-circle]" },
  notInstalled: { label: "未安装", color: "#6b7280", icon: "icon-[mdi--download]" },
  installing: { label: "安装中", color: "#3b82f6", icon: "icon-[mdi--loading]" },
  unknown: { label: "未知", color: "#9ca3af", icon: "icon-[mdi--help-circle]" },
};

// ==================== 计算属性 ====================

const installedDbs = computed(() => databases.value.filter((db) => db.installed));
const notInstalledDbs = computed(() => databases.value.filter((db) => !db.installed));

// ==================== 方法 ====================

async function scanDatabases() {
  scanning.value = true;
  try {
    const result = await invoke<DatabaseInfo[]>("detect_databases", {
      params: { sessionId: props.sessionId },
    });
    databases.value = result;
    toast({
      title: "扫描完成",
      description: `检测到 ${result.filter((db) => db.installed).length} 个已安装的数据库`,
    });
  } catch (error) {
    console.error("扫描数据库失败:", error);
    toast({
      title: "扫描失败",
      description: String(error),
      variant: "destructive",
    });
  } finally {
    scanning.value = false;
  }
}

function openInstallModal(dbType: DatabaseType) {
  installingDb.value = dbType;
  installOptions.value = {
    port: dbTypeConfig[dbType].defaultPort,
    rootPassword: "",
  };
  installLog.value = "";
  showInstallModal.value = true;
}

async function installDatabase() {
  if (!installingDb.value) return;

  loading.value = true;
  installLog.value = "开始安装...\n";

  try {
    const result = await invoke<string>("install_database", {
      params: {
        sessionId: props.sessionId,
        dbType: installingDb.value,
        options: installOptions.value,
      },
    });

    installLog.value += result + "\n";
    installLog.value += "安装完成!\n";

    toast({
      title: "安装成功",
      description: `${dbTypeConfig[installingDb.value].name} 安装完成`,
    });

    // 刷新列表
    await scanDatabases();
    showInstallModal.value = false;
  } catch (error) {
    installLog.value += `错误: ${error}\n`;
    toast({
      title: "安装失败",
      description: String(error),
      variant: "destructive",
    });
  } finally {
    loading.value = false;
  }
}

async function manageService(db: DatabaseInfo, action: "start" | "stop" | "restart" | "enable") {
  const serviceName = getServiceName(db.dbType);
  
  try {
    await invoke<string>("manage_database_service", {
      params: {
        sessionId: props.sessionId,
        serviceName,
        action,
      },
    });

    toast({
      title: "操作成功",
      description: `${dbTypeConfig[db.dbType].name} ${action} 完成`,
    });

    // 刷新状态
    await scanDatabases();
  } catch (error) {
    toast({
      title: "操作失败",
      description: String(error),
      variant: "destructive",
    });
  }
}

function getServiceName(dbType: DatabaseType): string {
  const serviceMap: Record<DatabaseType, string> = {
    mysql: "mysql",
    postgresql: "postgresql",
    redis: "redis-server",
    mongodb: "mongod",
    mariadb: "mariadb",
  };
  return serviceMap[dbType];
}

async function openConfigModal(db: DatabaseInfo) {
  selectedDb.value = db;
  configContent.value = "加载中...";
  showConfigModal.value = true;

  try {
    const config = await invoke<string>("get_database_config", {
      params: {
        sessionId: props.sessionId,
        dbType: db.dbType,
      },
    });
    configContent.value = config || "(配置文件为空或不存在)";
  } catch (error) {
    configContent.value = `加载失败: ${error}`;
  }
}

async function saveConfig() {
  if (!selectedDb.value) return;

  loading.value = true;
  try {
    await invoke<string>("update_database_config", {
      params: {
        sessionId: props.sessionId,
        dbType: selectedDb.value.dbType,
        configContent: configContent.value,
      },
    });

    toast({
      title: "保存成功",
      description: "配置文件已更新，建议重启服务生效",
    });

    showConfigModal.value = false;
  } catch (error) {
    toast({
      title: "保存失败",
      description: String(error),
      variant: "destructive",
    });
  } finally {
    loading.value = false;
  }
}

// ==================== 生命周期 ====================

onMounted(() => {
  scanDatabases();
});
</script>

<template>
  <div class="database-manager">
    <!-- 工具栏 -->
    <div class="toolbar">
      <h2 class="title">
        <span class="icon-[mdi--database] text-xl"></span>
        数据库管理
      </h2>
      <button
        class="btn-primary"
        :disabled="scanning"
        @click="scanDatabases"
      >
        <span
          :class="['icon-[mdi--refresh]', scanning && 'animate-spin']"
        ></span>
        {{ scanning ? "扫描中..." : "重新扫描" }}
      </button>
    </div>

    <!-- 已安装数据库 -->
    <div class="section">
      <h3 class="section-title">
        <span class="icon-[mdi--check-circle] text-green-500"></span>
        已安装的数据库
        <span class="count">({{ installedDbs.length }})</span>
      </h3>

      <div v-if="installedDbs.length === 0" class="empty-state">
        <span class="icon-[mdi--database-off] text-4xl text-neutral-600"></span>
        <p>未检测到已安装的数据库</p>
        <p class="text-sm text-neutral-500">点击下方卡片安装数据库</p>
      </div>

      <div class="db-grid">
        <div
          v-for="db in installedDbs"
          :key="db.dbType"
          class="db-card"
          :class="{ running: db.status === 'running' }"
        >
          <div class="db-header">
            <div
              class="db-icon"
              :style="{ backgroundColor: dbTypeConfig[db.dbType].color }"
            >
              <span :class="dbTypeConfig[db.dbType].icon"></span>
            </div>
            <div class="db-info">
              <h4 class="db-name">{{ dbTypeConfig[db.dbType].name }}</h4>
              <span class="db-version" v-if="db.version">v{{ db.version }}</span>
            </div>
            <div
              class="status-badge"
              :style="{ backgroundColor: statusConfig[db.status].color + '20', color: statusConfig[db.status].color }"
            >
              <span :class="statusConfig[db.status].icon"></span>
              {{ statusConfig[db.status].label }}
            </div>
          </div>

          <div class="db-details">
            <div class="detail-item">
              <span class="icon-[mdi--ethernet]"></span>
              <span>端口: {{ db.port || "-" }}</span>
            </div>
            <div class="detail-item" v-if="db.installPath">
              <span class="icon-[mdi--folder]"></span>
              <span class="truncate">{{ db.installPath }}</span>
            </div>
          </div>

          <div class="db-actions">
            <button
              v-if="db.status === 'stopped'"
              class="btn-action start"
              @click="manageService(db, 'start')"
            >
              <span class="icon-[mdi--play]"></span>
              启动
            </button>
            <button
              v-if="db.status === 'running'"
              class="btn-action stop"
              @click="manageService(db, 'stop')"
            >
              <span class="icon-[mdi--stop]"></span>
              停止
            </button>
            <button
              class="btn-action restart"
              @click="manageService(db, 'restart')"
            >
              <span class="icon-[mdi--restart]"></span>
              重启
            </button>
            <button class="btn-action config" @click="openConfigModal(db)">
              <span class="icon-[mdi--cog]"></span>
              配置
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 可安装数据库 -->
    <div class="section">
      <h3 class="section-title">
        <span class="icon-[mdi--download] text-blue-500"></span>
        可安装的数据库
      </h3>

      <div class="db-grid">
        <div
          v-for="(config, dbType) in dbTypeConfig"
          :key="dbType"
          class="db-card installable"
          :class="{ installed: databases.find((d) => d.dbType === dbType)?.installed }"
        >
          <div class="db-header">
            <div class="db-icon" :style="{ backgroundColor: config.color }">
              <span :class="config.icon"></span>
            </div>
            <div class="db-info">
              <h4 class="db-name">{{ config.name }}</h4>
              <p class="db-desc">{{ config.description }}</p>
            </div>
          </div>

          <div class="db-details">
            <div class="detail-item">
              <span class="icon-[mdi--ethernet]"></span>
              <span>默认端口: {{ config.defaultPort }}</span>
            </div>
          </div>

          <button
            class="btn-install"
            :disabled="databases.find((d) => d.dbType === dbType)?.installed"
            @click="openInstallModal(dbType)"
          >
            <span
              :class="
                databases.find((d) => d.dbType === dbType)?.installed
                  ? 'icon-[mdi--check]'
                  : 'icon-[mdi--download]'
              "
            ></span>
            {{
              databases.find((d) => d.dbType === dbType)?.installed
                ? "已安装"
                : "一键安装"
            }}
          </button>
        </div>
      </div>
    </div>

    <!-- 安装模态框 -->
    <Teleport to="body">
      <div v-if="showInstallModal" class="modal-overlay" @click.self="showInstallModal = false">
        <div class="modal">
          <div class="modal-header">
            <h3>
              <span class="icon-[mdi--download]"></span>
              安装 {{ installingDb ? dbTypeConfig[installingDb].name : "" }}
            </h3>
            <button class="btn-close" @click="showInstallModal = false">
              <span class="icon-[mdi--close]"></span>
            </button>
          </div>

          <div class="modal-body">
            <div class="form-group">
              <label>端口</label>
              <input
                v-model.number="installOptions.port"
                type="number"
                placeholder="默认端口"
              />
            </div>

            <div class="form-group" v-if="installingDb && installingDb !== 'redis' && installingDb !== 'mongodb'">
              <label>Root 密码</label>
              <input
                v-model="installOptions.rootPassword"
                type="password"
                placeholder="设置 root 密码"
              />
            </div>

            <div v-if="installLog" class="install-log">
              <pre>{{ installLog }}</pre>
            </div>
          </div>

          <div class="modal-footer">
            <button class="btn-secondary" @click="showInstallModal = false">
              取消
            </button>
            <button
              class="btn-primary"
              :disabled="loading"
              @click="installDatabase"
            >
              <span
                :class="['icon-[mdi--loading]', loading && 'animate-spin']"
              ></span>
              {{ loading ? "安装中..." : "开始安装" }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- 配置模态框 -->
    <Teleport to="body">
      <div v-if="showConfigModal" class="modal-overlay" @click.self="showConfigModal = false">
        <div class="modal config-modal">
          <div class="modal-header">
            <h3>
              <span class="icon-[mdi--cog]"></span>
              {{ selectedDb ? dbTypeConfig[selectedDb.dbType].name : "" }} 配置
            </h3>
            <button class="btn-close" @click="showConfigModal = false">
              <span class="icon-[mdi--close]"></span>
            </button>
          </div>

          <div class="modal-body">
            <textarea
              v-model="configContent"
              class="config-editor"
              spellcheck="false"
            ></textarea>
          </div>

          <div class="modal-footer">
            <button class="btn-secondary" @click="showConfigModal = false">
              取消
            </button>
            <button
              class="btn-primary"
              :disabled="loading"
              @click="saveConfig"
            >
              <span
                :class="['icon-[mdi--loading]', loading && 'animate-spin']"
              ></span>
              {{ loading ? "保存中..." : "保存配置" }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.database-manager {
  height: 100%;
  padding: 20px;
  overflow-y: auto;
  background: var(--background, #0a0a0a);
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.title {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 1.5rem;
  font-weight: 600;
  color: #e5e5e5;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary:hover:not(:disabled) {
  background: #2563eb;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.1);
  color: #e5e5e5;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-secondary:hover {
  background: rgba(255, 255, 255, 0.15);
}

.section {
  margin-bottom: 32px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 1.1rem;
  font-weight: 500;
  color: #e5e5e5;
  margin-bottom: 16px;
}

.count {
  color: #737373;
  font-size: 0.9rem;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px;
  color: #737373;
  gap: 12px;
}

.db-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 16px;
}

.db-card {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 16px;
  transition: all 0.2s;
}

.db-card:hover {
  border-color: rgba(255, 255, 255, 0.2);
  background: rgba(255, 255, 255, 0.08);
}

.db-card.running {
  border-color: rgba(34, 197, 94, 0.3);
}

.db-card.installable {
  opacity: 0.8;
}

.db-card.installable:hover {
  opacity: 1;
}

.db-card.installed {
  opacity: 0.5;
  pointer-events: none;
}

.db-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.db-icon {
  width: 48px;
  height: 48px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 24px;
}

.db-info {
  flex: 1;
  min-width: 0;
}

.db-name {
  font-size: 1.1rem;
  font-weight: 600;
  color: #e5e5e5;
  margin: 0;
}

.db-version {
  font-size: 0.85rem;
  color: #737373;
}

.db-desc {
  font-size: 0.85rem;
  color: #a3a3a3;
  margin: 4px 0 0;
  line-height: 1.4;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 0.8rem;
  font-weight: 500;
}

.db-details {
  margin-bottom: 12px;
}

.detail-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: #a3a3a3;
  margin-bottom: 4px;
}

.detail-item .truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.db-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-action {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  font-size: 0.85rem;
  cursor: pointer;
  transition: all 0.2s;
  background: rgba(255, 255, 255, 0.1);
  color: #e5e5e5;
}

.btn-action:hover {
  background: rgba(255, 255, 255, 0.15);
}

.btn-action.start {
  background: rgba(34, 197, 94, 0.2);
  color: #22c55e;
}

.btn-action.start:hover {
  background: rgba(34, 197, 94, 0.3);
}

.btn-action.stop {
  background: rgba(239, 68, 68, 0.2);
  color: #ef4444;
}

.btn-action.stop:hover {
  background: rgba(239, 68, 68, 0.3);
}

.btn-action.restart {
  background: rgba(245, 158, 11, 0.2);
  color: #f59e0b;
}

.btn-action.restart:hover {
  background: rgba(245, 158, 11, 0.3);
}

.btn-install {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 10px;
  background: #3b82f6;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  margin-top: 12px;
}

.btn-install:hover:not(:disabled) {
  background: #2563eb;
}

.btn-install:disabled {
  background: #22c55e;
  cursor: default;
}

/* 模态框 */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal {
  background: #171717;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  width: 100%;
  max-width: 500px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.config-modal {
  max-width: 800px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.modal-header h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 1.1rem;
  font-weight: 600;
  color: #e5e5e5;
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  color: #737373;
  font-size: 20px;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s;
}

.btn-close:hover {
  color: #e5e5e5;
  background: rgba(255, 255, 255, 0.1);
}

.modal-body {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 0.9rem;
  color: #a3a3a3;
  margin-bottom: 6px;
}

.form-group input {
  width: 100%;
  padding: 10px 12px;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #e5e5e5;
  font-size: 14px;
  outline: none;
  transition: all 0.2s;
}

.form-group input:focus {
  border-color: #3b82f6;
  background: rgba(0, 0, 0, 0.5);
}

.install-log {
  background: rgba(0, 0, 0, 0.5);
  border-radius: 8px;
  padding: 12px;
  margin-top: 16px;
  max-height: 200px;
  overflow-y: auto;
}

.install-log pre {
  margin: 0;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.8rem;
  color: #a3a3a3;
  white-space: pre-wrap;
  word-break: break-all;
}

.config-editor {
  width: 100%;
  min-height: 400px;
  padding: 12px;
  background: rgba(0, 0, 0, 0.5);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  color: #e5e5e5;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 0.85rem;
  line-height: 1.6;
  resize: vertical;
  outline: none;
}

.config-editor:focus {
  border-color: #3b82f6;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid rgba(255, 255, 255, 0.1);
}

/* 动画 */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
