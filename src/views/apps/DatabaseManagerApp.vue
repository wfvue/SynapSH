<!-- 数据库管理应用 - 宝塔面板风格 -->
<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "../../components/ui/toast";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "../../components/ui/dialog";
import { Button } from "../../components/ui/button";
import { Input } from "../../components/ui/input";

// ==================== 类型定义 ====================

type DatabaseType = "mysql" | "sqlserver" | "mongodb" | "redis" | "postgresql" | "sqlite";
type InstallStatus = "installed" | "notInstalled" | "remote";

interface DatabaseInstance {
  id: string;
  name: string;
  username: string;
  password: string;
  backupCount: number;
  location: string;
  comment: string;
  createdAt: string;
}

interface DbTypeInfo {
  type: DatabaseType;
  name: string;
  icon: string;
  color: string;
  installed: boolean;
  hasRemote: boolean;
}

// ==================== Props ====================

const props = defineProps<{
  sessionId: string;
}>();

// ==================== 状态 ====================

const { toast } = useToast();

const activeTab = ref<DatabaseType>("mysql");
const dbTypes = ref<DbTypeInfo[]>([
  { type: "mysql", name: "MySQL", icon: "icon-[mdi--database]", color: "#00758f", installed: false, hasRemote: false },
  { type: "sqlserver", name: "SQLServer", icon: "icon-[mdi--microsoft-windows]", color: "#a91d22", installed: false, hasRemote: false },
  { type: "mongodb", name: "MongoDB", icon: "icon-[mdi--leaf]", color: "#47a248", installed: false, hasRemote: false },
  { type: "redis", name: "Redis", icon: "icon-[mdi--database-cog]", color: "#dc382d", installed: false, hasRemote: false },
  { type: "postgresql", name: "PgSQL", icon: "icon-[mdi--elephant]", color: "#336791", installed: false, hasRemote: false },
  { type: "sqlite", name: "SQLite", icon: "icon-[mdi--database-outline]", color: "#003b57", installed: false, hasRemote: false },
]);

const databases = ref<DatabaseInstance[]>([]);
const loading = ref(false);
const searchQuery = ref("");
const currentPage = ref(1);
const pageSize = ref(10);

// 模态框状态
const showInstallModal = ref(false);
const showAddModal = ref(false);
const showRemoteModal = ref(false);
const installLoading = ref(false);
const installLog = ref("");

// 表单数据
const formData = ref({
  name: "",
  username: "",
  password: "",
  comment: "",
  access: "localhost",
  charset: "utf8mb4",
  rootPassword: "",
});

const charsets = [
  { value: "utf8mb4", label: "utf8mb4" },
  { value: "utf8", label: "utf8" },
  { value: "latin1", label: "latin1" },
  { value: "gbk", label: "gbk" },
];

// ==================== 计算属性 ====================

const activeDbType = computed(() => dbTypes.value.find(d => d.type === activeTab.value)!);

const installStatus = computed((): InstallStatus => {
  if (activeDbType.value.installed) return "installed";
  if (activeDbType.value.hasRemote) return "remote";
  return "notInstalled";
});

const filteredDatabases = computed(() => {
  if (!searchQuery.value) return databases.value;
  const query = searchQuery.value.toLowerCase();
  return databases.value.filter(d => 
    d.name.toLowerCase().includes(query) || 
    d.comment.toLowerCase().includes(query)
  );
});

const paginatedDatabases = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  return filteredDatabases.value.slice(start, start + pageSize.value);
});

const totalPages = computed(() => Math.ceil(filteredDatabases.value.length / pageSize.value));

// ==================== 方法 ====================

// 全局安装状态（只检测一次）
const globalInstallStatus = ref<Record<DatabaseType, boolean>>({
  mysql: false,
  sqlserver: false,
  mongodb: false,
  redis: false,
  postgresql: false,
  sqlite: false,
});
const hasDetectedAll = ref(false);

async function detectAllDatabases() {
  if (hasDetectedAll.value) return;
  
  loading.value = true;
  try {
    const result = await invoke<any[]>("detect_databases", {
      params: { sessionId: props.sessionId },
    });
    
    // 更新所有数据库的安装状态
    for (const db of result) {
      const type = db.dbType as DatabaseType;
      const typeInfo = dbTypes.value.find(t => t.type === type);
      if (typeInfo) {
        typeInfo.installed = db.installed;
        globalInstallStatus.value[type] = db.installed;
      }
    }
    
    hasDetectedAll.value = true;
    
    // 加载当前选中的数据库列表
    if (activeDbType.value.installed) {
      await loadDatabases();
    }
  } catch (error) {
    console.error("检测数据库失败:", error);
  } finally {
    loading.value = false;
  }
}

// 重新检测单个数据库（点击"读取本地"按钮时）
async function redetectCurrentDb() {
  const currentType = activeTab.value;
  loading.value = true;
  try {
    // 这里可以调用一个单独的检测接口
    await loadDatabases();
  } catch (error) {
    console.error("重新检测失败:", error);
  } finally {
    loading.value = false;
  }
}

async function loadDatabases() {
  if (!activeDbType.value.installed) return;
  
  loading.value = true;
  try {
    const result = await invoke<DatabaseInstance[]>("get_database_schemas", {
      params: {
        sessionId: props.sessionId,
        dbType: activeTab.value,
      },
    });
    databases.value = result;
  } catch (error) {
    console.error("加载数据库列表失败:", error);
    toast({ title: "加载失败", description: String(error), variant: "destructive" });
    databases.value = [];
  } finally {
    loading.value = false;
  }
}

function switchTab(type: DatabaseType) {
  activeTab.value = type;
  currentPage.value = 1;
  searchQuery.value = "";
  databases.value = [];
  
  // 如果已检测过，直接加载数据库列表
  if (hasDetectedAll.value && activeDbType.value.installed) {
    loadDatabases();
  }
}

async function installEnvironment() {
  installLoading.value = true;
  installLog.value = "";
  
  try {
    const result = await invoke<string>("install_database", {
      params: {
        sessionId: props.sessionId,
        dbType: activeTab.value,
        options: {
          port: getDefaultPort(activeTab.value),
          rootPassword: formData.value.rootPassword || generatePassword(),
        },
      },
    });
    
    installLog.value = result;
    toast({ title: "安装成功", description: `${activeDbType.value.name} 环境安装完成` });
    
    activeDbType.value.installed = true;
    showInstallModal.value = false;
    await loadDatabases();
  } catch (error) {
    toast({ title: "安装失败", description: String(error), variant: "destructive" });
  } finally {
    installLoading.value = false;
  }
}

function getDefaultPort(type: DatabaseType): number {
  const ports: Record<DatabaseType, number> = {
    mysql: 3306,
    sqlserver: 1433,
    mongodb: 27017,
    redis: 6379,
    postgresql: 5432,
    sqlite: 0,
  };
  return ports[type];
}

function openInstallModal() {
  formData.value.rootPassword = generatePassword();
  installLog.value = "";
  showInstallModal.value = true;
}

function openAddModal() {
  formData.value = {
    name: "",
    username: "",
    password: generatePassword(),
    comment: "",
    access: "localhost",
    charset: "utf8mb4",
    rootPassword: "",
  };
  showAddModal.value = true;
}

async function createDatabase() {
  if (!formData.value.name) {
    toast({ title: "请输入数据库名称", variant: "destructive" });
    return;
  }

  loading.value = true;
  try {
    await invoke("create_database_schema", {
      params: {
        sessionId: props.sessionId,
        dbType: activeTab.value,
        name: formData.value.name,
        username: formData.value.username || formData.value.name,
        password: formData.value.password,
        comment: formData.value.comment,
        access: formData.value.access,
        charset: formData.value.charset,
      },
    });

    toast({ title: "创建成功", description: `数据库 ${formData.value.name} 已创建` });
    showAddModal.value = false;
    await loadDatabases();
  } catch (error) {
    toast({ title: "创建失败", description: String(error), variant: "destructive" });
  } finally {
    loading.value = false;
  }
}

async function deleteDatabase(db: DatabaseInstance) {
  if (!confirm(`确定要删除数据库 "${db.name}" 吗？此操作不可恢复！`)) return;

  loading.value = true;
  try {
    await invoke("delete_database", {
      params: {
        sessionId: props.sessionId,
        dbType: activeTab.value,
        dbId: db.id,
        username: db.username,
      },
    });

    toast({ title: "删除成功" });
    await loadDatabases();
  } catch (error) {
    toast({ title: "删除失败", description: String(error), variant: "destructive" });
  } finally {
    loading.value = false;
  }
}

function generatePassword(): string {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let password = "";
  for (let i = 0; i < 16; i++) {
    password += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return password;
}

function copyPassword() {
  navigator.clipboard.writeText(formData.value.password);
  toast({ title: "密码已复制到剪贴板" });
}

function addRemoteDatabase() {
  showRemoteModal.value = true;
}

function readLocalDatabase() {
  redetectCurrentDb();
}

// ==================== 生命周期 ====================

onMounted(() => {
  // 一次性检测所有数据库安装状态
  detectAllDatabases();
});
</script>

<template>
  <div class="h-full flex flex-col bg-[#f5f5f5] dark:bg-neutral-900">
    <!-- 数据库类型标签栏 -->
    <div class="bg-white dark:bg-neutral-800 border-b border-neutral-200 dark:border-white/10">
      <div class="flex items-center px-4">
        <div
          v-for="db in dbTypes"
          :key="db.type"
          class="tab-item"
          :class="{ active: activeTab === db.type }"
          @click="switchTab(db.type)"
        >
          <span :class="db.icon"></span>
          <span>{{ db.name }}</span>
          <span
            v-if="db.installed"
            class="ml-1.5 w-1.5 h-1.5 rounded-full bg-green-500"
          ></span>
        </div>
      </div>
    </div>

    <!-- 加载中状态 -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <span class="icon-[mdi--loading] text-4xl text-neutral-300 dark:text-neutral-600 animate-spin block mb-4"></span>
        <p class="text-neutral-500 dark:text-neutral-400">正在检测 {{ activeDbType.name }} 安装状态...</p>
      </div>
    </div>

    <!-- 未安装提示 -->
    <template v-else-if="installStatus === 'notInstalled'">
      <div class="flex-1 flex items-center justify-center p-8">
        <div class="bg-white dark:bg-neutral-800 rounded-xl border border-neutral-200 dark:border-white/10 p-8 max-w-lg w-full text-center">
          <div
            class="w-16 h-16 rounded-2xl mx-auto mb-4 flex items-center justify-center text-white text-2xl"
            :style="{ backgroundColor: activeDbType.color }"
          >
            <span :class="activeDbType.icon"></span>
          </div>
          <h3 class="text-lg font-semibold text-neutral-800 dark:text-neutral-100 mb-2">
            当前未安装 {{ activeDbType.name }} 环境/远程数据库
          </h3>
          <p class="text-sm text-neutral-500 dark:text-neutral-400 mb-6">
            您可以选择添加远程数据库连接，或者在当前服务器上安装 {{ activeDbType.name }}
          </p>
          <div class="flex items-center justify-center gap-3">
            <Button variant="outline" class="gap-2" @click="addRemoteDatabase">
              <span class="icon-[mdi--cloud-plus]"></span>
              添加远程数据库
            </Button>
            <Button variant="outline" class="gap-2" @click="readLocalDatabase">
              <span class="icon-[mdi--refresh]"></span>
              读取本地 {{ activeDbType.name }}
            </Button>
            <Button class="gap-2 bg-green-500 hover:bg-green-600 text-white" @click="openInstallModal">
              <span class="icon-[mdi--download]"></span>
              安装 {{ activeDbType.name }}
            </Button>
          </div>
        </div>
      </div>
    </template>

    <!-- 已安装 - 显示数据库列表 -->
    <template v-else-if="installStatus === 'installed'">
      <!-- 工具栏 -->
      <div class="bg-white dark:bg-neutral-800 border-b border-neutral-200 dark:border-white/10 p-4">
        <div class="flex items-center justify-between flex-wrap gap-4">
          <div class="flex items-center gap-2">
            <Button 
              v-if="activeTab !== 'redis' && activeTab !== 'sqlite'"
              class="gap-2 bg-green-500 hover:bg-green-600 text-white" 
              @click="openAddModal"
            >
              <span class="icon-[mdi--plus]"></span>
              添加数据库
            </Button>
            <Button variant="outline" class="gap-2">
              <span class="icon-[mdi--lock]"></span>
              管理员密码
            </Button>
            <Button variant="outline" class="gap-2" @click="addRemoteDatabase">
              <span class="icon-[mdi--cloud]"></span>
              远程数据库
            </Button>
            <Button variant="outline" class="gap-2" @click="loadDatabases">
              <span :class="['icon-[mdi--refresh]', loading && 'animate-spin']"></span>
              同步数据库
            </Button>
            <Button variant="ghost" class="text-green-500 hover:text-green-600 gap-1">
              <span class="icon-[mdi--help-circle]"></span>
              需求反馈
            </Button>
          </div>

          <div class="flex items-center gap-2">
            <select v-model="pageSize" class="h-9 px-3 rounded-md border border-neutral-300 dark:border-white/10 bg-white dark:bg-neutral-700 text-sm">
              <option :value="10">10条/页</option>
              <option :value="20">20条/页</option>
              <option :value="50">50条/页</option>
            </select>
            <div class="relative">
              <input
                v-model="searchQuery"
                type="text"
                placeholder="请输入数据库名称/备注"
                class="h-9 w-64 pl-3 pr-9 rounded-md border border-neutral-300 dark:border-white/10 bg-white dark:bg-neutral-700 text-sm focus:outline-none focus:ring-2 focus:ring-green-500 dark:text-neutral-200"
              />
              <span class="icon-[mdi--magnify] absolute right-3 top-1/2 -translate-y-1/2 text-neutral-400"></span>
            </div>
          </div>
        </div>
      </div>

      <!-- 数据库列表 -->
      <div class="flex-1 overflow-auto p-4">
        <div class="bg-white dark:bg-neutral-800 rounded-lg border border-neutral-200 dark:border-white/10 overflow-hidden">
          <table class="w-full">
            <thead class="bg-neutral-50 dark:bg-neutral-700/50">
              <tr>
                <th class="w-10 px-4 py-3">
                  <input type="checkbox" class="rounded border-neutral-300" />
                </th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">
                  数据库名
                  <span class="icon-[mdi--sort] text-neutral-400 cursor-pointer ml-1"></span>
                </th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">
                  用户名
                  <span class="icon-[mdi--sort] text-neutral-400 cursor-pointer ml-1"></span>
                </th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">密码</th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">备份</th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">数据库位置</th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">备注</th>
                <th class="px-4 py-3 text-left text-sm font-medium text-neutral-600 dark:text-neutral-300">操作</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-neutral-100 dark:divide-white/5">
              <tr
                v-for="db in paginatedDatabases"
                :key="db.id"
                class="hover:bg-neutral-50 dark:hover:bg-white/5 transition-colors"
              >
                <td class="px-4 py-3">
                  <input type="checkbox" class="rounded border-neutral-300" />
                </td>
                <td class="px-4 py-3 text-sm font-medium text-neutral-700 dark:text-neutral-200">{{ db.name }}</td>
                <td class="px-4 py-3 text-sm text-neutral-700 dark:text-neutral-200">{{ db.username }}</td>
                <td class="px-4 py-3 text-sm">
                  <div class="flex items-center gap-2">
                    <span class="text-neutral-500">**********</span>
                    <button class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300">
                      <span class="icon-[mdi--eye]"></span>
                    </button>
                    <button class="text-neutral-400 hover:text-neutral-600 dark:hover:text-neutral-300" @click="copyPassword">
                      <span class="icon-[mdi--content-copy]"></span>
                    </button>
                  </div>
                </td>
                <td class="px-4 py-3 text-sm">
                  <div class="flex items-center gap-2">
                    <button class="text-green-500 hover:text-green-600 text-xs">点击备份</button>
                    <span class="text-neutral-300">|</span>
                    <button class="text-green-500 hover:text-green-600 text-xs">导入</button>
                  </div>
                </td>
                <td class="px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400">{{ db.location }}</td>
                <td class="px-4 py-3 text-sm text-neutral-600 dark:text-neutral-400">{{ db.comment }}</td>
                <td class="px-4 py-3 text-sm">
                  <div class="flex items-center gap-2">
                    <button class="text-green-500 hover:text-green-600 text-xs">权限</button>
                    <button class="text-green-500 hover:text-green-600 text-xs">改密</button>
                    <button class="text-red-500 hover:text-red-600 text-xs" @click="deleteDatabase(db)">删除</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>

          <!-- 空状态 -->
          <div v-if="paginatedDatabases.length === 0" class="py-16 text-center">
            <span class="icon-[mdi--database-off] text-4xl text-neutral-300 dark:text-neutral-600"></span>
            <p class="mt-2 text-neutral-500 dark:text-neutral-400">暂无数据库，点击"添加数据库"创建</p>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div class="bg-white dark:bg-neutral-800 border-t border-neutral-200 dark:border-white/10 p-4">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <select class="h-8 px-2 rounded border border-neutral-300 dark:border-white/10 bg-white dark:bg-neutral-700 text-sm">
              <option>请选择批量操作</option>
              <option>批量删除</option>
            </select>
            <Button variant="outline" size="sm" class="bg-green-500 hover:bg-green-600 text-white border-none">
              批量操作
            </Button>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm text-neutral-500 dark:text-neutral-400">共 {{ filteredDatabases.length }} 条</span>
            <div class="flex items-center gap-1">
              <button
                class="w-8 h-8 flex items-center justify-center rounded border border-neutral-300 dark:border-white/10 hover:bg-neutral-100 dark:hover:bg-white/5 disabled:opacity-50"
                :disabled="currentPage === 1"
                @click="currentPage--"
              >
                <span class="icon-[mdi--chevron-left] text-sm"></span>
              </button>
              <span class="w-8 h-8 flex items-center justify-center rounded bg-green-500 text-white text-sm">{{ currentPage }}</span>
              <button
                class="w-8 h-8 flex items-center justify-center rounded border border-neutral-300 dark:border-white/10 hover:bg-neutral-100 dark:hover:bg-white/5 disabled:opacity-50"
                :disabled="currentPage >= totalPages"
                @click="currentPage++"
              >
                <span class="icon-[mdi--chevron-right] text-sm"></span>
              </button>
            </div>
            <span class="text-sm text-neutral-500 dark:text-neutral-400">{{ pageSize }}条/页</span>
            <div class="flex items-center gap-1">
              <span class="text-sm text-neutral-500 dark:text-neutral-400">前往</span>
              <input
                v-model.number="currentPage"
                type="number"
                min="1"
                :max="totalPages"
                class="w-12 h-8 px-1 text-center rounded border border-neutral-300 dark:border-white/10 bg-white dark:bg-neutral-700 text-sm"
              />
              <span class="text-sm text-neutral-500 dark:text-neutral-400">页</span>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- 安装环境模态框 -->
    <Dialog v-model:open="showInstallModal">
      <DialogContent class="max-w-md bg-white dark:bg-neutral-800">
        <DialogHeader>
          <DialogTitle>安装 {{ activeDbType.name }} 环境</DialogTitle>
          <DialogDescription>设置管理员密码开始安装</DialogDescription>
        </DialogHeader>
        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <label class="text-sm font-medium">管理员密码</label>
            <div class="flex gap-2">
              <Input v-model="formData.rootPassword" type="text" />
              <Button variant="outline" size="icon" @click="formData.rootPassword = generatePassword()">
                <span class="icon-[mdi--dice-5]"></span>
              </Button>
            </div>
          </div>
          <div v-if="installLog" class="bg-neutral-100 dark:bg-neutral-900 rounded p-3 max-h-40 overflow-auto">
            <pre class="text-xs whitespace-pre-wrap">{{ installLog }}</pre>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="showInstallModal = false">取消</Button>
          <Button class="bg-green-500 hover:bg-green-600 text-white" :disabled="installLoading" @click="installEnvironment">
            {{ installLoading ? '安装中...' : '开始安装' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 添加数据库模态框 -->
    <Dialog v-model:open="showAddModal">
      <DialogContent class="max-w-md bg-white dark:bg-neutral-800">
        <DialogHeader>
          <DialogTitle>添加数据库</DialogTitle>
          <DialogDescription>在 {{ activeDbType.name }} 中创建新数据库</DialogDescription>
        </DialogHeader>
        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <label class="text-sm font-medium">数据库名</label>
            <Input v-model="formData.name" placeholder="请输入数据库名称" />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium">用户名</label>
            <Input v-model="formData.username" :placeholder="formData.name || '请输入用户名'" />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium">密码</label>
            <div class="flex gap-2">
              <Input v-model="formData.password" type="text" />
              <Button variant="outline" size="icon" @click="formData.password = generatePassword()">
                <span class="icon-[mdi--dice-5]"></span>
              </Button>
              <Button variant="outline" size="icon" @click="copyPassword">
                <span class="icon-[mdi--content-copy]"></span>
              </Button>
            </div>
          </div>
          <div v-if="activeTab === 'mysql'" class="space-y-2">
            <label class="text-sm font-medium">字符集</label>
            <select v-model="formData.charset" class="w-full h-10 px-3 rounded-md border border-neutral-300 dark:border-white/10 bg-white dark:bg-neutral-700">
              <option v-for="cs in charsets" :key="cs.value" :value="cs.value">{{ cs.label }}</option>
            </select>
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium">备注</label>
            <Input v-model="formData.comment" placeholder="请输入备注" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="showAddModal = false">取消</Button>
          <Button class="bg-green-500 hover:bg-green-600 text-white" :disabled="loading" @click="createDatabase">
            {{ loading ? '创建中...' : '提交' }}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>

    <!-- 添加远程数据库模态框 -->
    <Dialog v-model:open="showRemoteModal">
      <DialogContent class="max-w-md bg-white dark:bg-neutral-800">
        <DialogHeader>
          <DialogTitle>添加远程 {{ activeDbType.name }}</DialogTitle>
          <DialogDescription>连接到远程数据库服务器</DialogDescription>
        </DialogHeader>
        <div class="space-y-4 py-4">
          <div class="space-y-2">
            <label class="text-sm font-medium">主机地址</label>
            <Input placeholder="例如: 192.168.1.100" />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium">端口</label>
            <Input :value="String(getDefaultPort(activeTab))" />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium">用户名</label>
            <Input placeholder="root" />
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium">密码</label>
            <Input type="password" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" @click="showRemoteModal = false">取消</Button>
          <Button class="bg-green-500 hover:bg-green-600 text-white">连接</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </div>
</template>

<style scoped>
.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 12px 20px;
  font-size: 14px;
  color: #737373;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab-item:hover {
  color: #22c55e;
}

.tab-item.active {
  color: #22c55e;
  border-bottom-color: #22c55e;
}

.dark .tab-item {
  color: #a3a3a3;
}

.dark .tab-item:hover {
  color: #4ade80;
}

.dark .tab-item.active {
  color: #4ade80;
  border-bottom-color: #4ade80;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.animate-spin {
  animation: spin 1s linear infinite;
}
</style>
