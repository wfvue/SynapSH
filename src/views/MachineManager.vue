<!-- MachineManager.vue - 机器管理页面，实现分组、批量操作、快捷键等功能 -->
<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from "vue";
import { api } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { Separator } from "@/components/ui/separator";
import { Checkbox } from "@/components/ui/checkbox";

const emit = defineEmits<{
  connect: [
    payload: {
      sessionId: string;
      machineId: string;
      machineName: string;
      host: string;
    },
  ];
}>();

interface Machine {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  password?: string;
  privateKeyPath?: string;
  authType: string;
  tags: string;
  os: string;
  createdAt: string;
  updatedAt: string;
}

interface MachineInput {
  name?: string;
  host: string;
  port?: number;
  username: string;
  password?: string;
  privateKeyPath?: string;
  authType: string;
  tags?: string[];
  os?: string;
}

type ConnectionStatus = "unknown" | "testing" | "online" | "offline";
type GroupMode = "none" | "os" | "tag";
type FilterMode = "all" | "online" | "offline";

const machines = ref<Machine[]>([]);
const machineStatus = ref<Record<string, ConnectionStatus>>({});
const isLoading = ref(false);
const error = ref("");
const searchQuery = ref("");

const showAddModal = ref(false);
const showEditModal = ref(false);
const showDeleteConfirm = ref(false);
const showBatchDeleteConfirm = ref(false);
const editingMachine = ref<Machine | null>(null);
const deletingMachine = ref<Machine | null>(null);

const newMachine = ref<MachineInput>({
  name: "",
  host: "",
  port: 22,
  username: "root",
  password: "",
  privateKeyPath: "",
  authType: "password",
  tags: [],
  os: "linux",
});

const isConnecting = ref<string | null>(null);

const selectedMachines = ref<Set<string>>(new Set());
const isSelectionMode = ref(false);
const groupMode = ref<GroupMode>("none");
const filterMode = ref<FilterMode>("all");

const showSkeleton = ref(true);

const allTags = computed(() => {
  const tagSet = new Set<string>();
  machines.value.forEach((m) => {
    parseTags(m.tags).forEach((t) => tagSet.add(t));
  });
  return Array.from(tagSet).sort();
});

const filteredMachines = computed(() => {
  let result = machines.value;

  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(
      (m) =>
        m.name.toLowerCase().includes(query) ||
        m.host.toLowerCase().includes(query) ||
        m.username.toLowerCase().includes(query) ||
        parseTags(m.tags).some((t) => t.toLowerCase().includes(query)),
    );
  }

  if (filterMode.value === "online") {
    result = result.filter((m) => machineStatus.value[m.id] === "online");
  } else if (filterMode.value === "offline") {
    result = result.filter(
      (m) => machineStatus.value[m.id] === "offline" || machineStatus.value[m.id] === "unknown",
    );
  }

  return result;
});

const groupedMachines = computed(() => {
  if (groupMode.value === "none") {
    return [{ key: "all", label: "全部机器", machines: filteredMachines.value }];
  }

  const groups: Map<string, { label: string; machines: Machine[] }> = new Map();

  filteredMachines.value.forEach((m) => {
    if (groupMode.value === "os") {
      const key = m.os || "unknown";
      const label = getOsLabel(key);
      if (!groups.has(key)) groups.set(key, { label, machines: [] });
      groups.get(key)!.machines.push(m);
    } else if (groupMode.value === "tag") {
      const tags = parseTags(m.tags);
      if (tags.length === 0) {
        if (!groups.has("untagged")) groups.set("untagged", { label: "未标记", machines: [] });
        groups.get("untagged")!.machines.push(m);
      } else {
        tags.forEach((tag) => {
          if (!groups.has(tag)) groups.set(tag, { label: tag, machines: [] });
          groups.get(tag)!.machines.push(m);
        });
      }
    }
  });

  return Array.from(groups.entries()).map(([key, value]) => ({
    key,
    label: value.label,
    machines: value.machines,
  }));
});

const selectedCount = computed(() => selectedMachines.value.size);
const selectedOnlineCount = computed(() => {
  return Array.from(selectedMachines.value).filter((id) => machineStatus.value[id] === "online")
    .length;
});

function getOsLabel(os: string): string {
  const labels: Record<string, string> = {
    linux: "Linux",
    windows: "Windows",
    macos: "macOS",
    unknown: "未知",
  };
  return labels[os] || os;
}

function getStatusColor(machineId: string): string {
  const status = machineStatus.value[machineId] || "unknown";
  switch (status) {
    case "online":
      return "bg-success";
    case "offline":
      return "bg-danger";
    case "testing":
      return "bg-warning animate-pulse";
    default:
      return "bg-tertiary";
  }
}

function getStatusTitle(machineId: string): string {
  const status = machineStatus.value[machineId] || "unknown";
  switch (status) {
    case "online":
      return "在线";
    case "offline":
      return "离线";
    case "testing":
      return "测试中...";
    default:
      return "未知状态";
  }
}

async function loadMachines() {
  isLoading.value = true;
  showSkeleton.value = true;
  error.value = "";
  try {
    machines.value = await api.listMachines();
    machineStatus.value = {};
    setTimeout(() => (showSkeleton.value = false), 300);
  } catch (e) {
    error.value = String(e);
    showSkeleton.value = false;
  } finally {
    isLoading.value = false;
  }
}

async function handleAddMachine() {
  try {
    const machine = await api.addMachine(JSON.parse(JSON.stringify(newMachine.value)));
    machines.value.unshift(machine);
    showAddModal.value = false;
    resetForm();
  } catch (e) {
    error.value = String(e);
  }
}

async function handleUpdateMachine() {
  if (!editingMachine.value) return;
  try {
    const input: MachineInput = {
      name: editingMachine.value.name,
      host: editingMachine.value.host,
      port: editingMachine.value.port,
      username: editingMachine.value.username,
      password: editingMachine.value.password,
      privateKeyPath: editingMachine.value.privateKeyPath,
      authType: editingMachine.value.authType,
      os: editingMachine.value.os,
    };
    const updated = await api.updateMachine(editingMachine.value.id, input);
    const idx = machines.value.findIndex((m) => m.id === updated.id);
    if (idx !== -1) {
      machines.value[idx] = updated;
    }
    showEditModal.value = false;
    editingMachine.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

async function handleDeleteMachine() {
  if (!deletingMachine.value) return;
  try {
    await api.deleteMachine(deletingMachine.value.id);
    machines.value = machines.value.filter((m) => m.id !== deletingMachine.value!.id);
    selectedMachines.value.delete(deletingMachine.value.id);
    showDeleteConfirm.value = false;
    deletingMachine.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

async function handleBatchDelete() {
  const ids = Array.from(selectedMachines.value);
  try {
    for (const id of ids) {
      await api.deleteMachine(id);
    }
    machines.value = machines.value.filter((m) => !ids.includes(m.id));
    selectedMachines.value.clear();
    isSelectionMode.value = false;
    showBatchDeleteConfirm.value = false;
  } catch (e) {
    error.value = String(e);
  }
}

async function testConnection(machine: Machine) {
  machineStatus.value[machine.id] = "testing";
  try {
    const result = await api.testConnection({
      host: machine.host,
      port: machine.port,
      username: machine.username,
      password: machine.password,
      privateKey: machine.privateKeyPath,
    });
    machineStatus.value[machine.id] = result ? "online" : "offline";
  } catch (e) {
    machineStatus.value[machine.id] = "offline";
  }
}

async function testAllConnections() {
  const batchSize = 5;
  const allMachines = filteredMachines.value;

  for (let i = 0; i < allMachines.length; i += batchSize) {
    const batch = allMachines.slice(i, i + batchSize);
    await Promise.all(batch.map((m) => testConnection(m)));
  }
}

async function testSelectedConnections() {
  const ids = Array.from(selectedMachines.value);
  const batchSize = 5;

  for (let i = 0; i < ids.length; i += batchSize) {
    const batch = ids.slice(i, i + batchSize);
    const machinesToTest = machines.value.filter((m) => batch.includes(m.id));
    await Promise.all(machinesToTest.map((m) => testConnection(m)));
  }
}

async function connectToMachine(machine: Machine) {
  if (isSelectionMode.value) {
    toggleSelection(machine.id);
    return;
  }

  isConnecting.value = machine.id;
  const sessionId = `session_${machine.id}_${Date.now()}`;

  const isKeyAuth = machine.authType === "key";
  const password = !isKeyAuth && machine.password ? machine.password : undefined;
  const privateKey = isKeyAuth && machine.privateKeyPath ? machine.privateKeyPath : undefined;

  try {
    await Promise.race([
      api.connectSSH(sessionId, {
        host: machine.host,
        port: machine.port,
        username: machine.username,
        password: password,
        privateKey: privateKey,
      }),
      new Promise((_, reject) => setTimeout(() => reject(new Error("连接超时 (15秒)")), 15000)),
    ]);
    emit("connect", {
      sessionId,
      machineId: machine.id,
      machineName: machine.name || machine.host,
      host: machine.host,
    });
  } catch (e) {
    console.error("连接失败:", e);
    error.value = String(e);
    machineStatus.value[machine.id] = "offline";
  } finally {
    isConnecting.value = null;
  }
}

function openEditModal(machine: Machine) {
  editingMachine.value = { ...machine };
  showEditModal.value = true;
}

function openDeleteConfirm(machine: Machine) {
  deletingMachine.value = machine;
  showDeleteConfirm.value = true;
}

function resetForm() {
  newMachine.value = {
    name: "",
    host: "",
    port: 22,
    username: "root",
    password: "",
    privateKeyPath: "",
    authType: "password",
    tags: [],
    os: "linux",
  };
}

function parseTags(tagsJson: string): string[] {
  try {
    return JSON.parse(tagsJson);
  } catch {
    return [];
  }
}

function formatLastUpdated(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return "刚刚";
  if (diffMins < 60) return `${diffMins} 分钟前`;
  if (diffHours < 24) return `${diffHours} 小时前`;
  if (diffDays < 7) return `${diffDays} 天前`;
  return date.toLocaleDateString("zh-CN");
}

function toggleSelection(machineId: string) {
  if (selectedMachines.value.has(machineId)) {
    selectedMachines.value.delete(machineId);
  } else {
    selectedMachines.value.add(machineId);
  }
}

function toggleSelectionMode() {
  isSelectionMode.value = !isSelectionMode.value;
  if (!isSelectionMode.value) {
    selectedMachines.value.clear();
  }
}

function selectAllInGroup(groupMachines: Machine[]) {
  groupMachines.forEach((m) => selectedMachines.value.add(m.id));
}

function clearSelection() {
  selectedMachines.value.clear();
}

function handleKeydown(e: KeyboardEvent) {
  if (e.metaKey || e.ctrlKey) {
    if (e.key === "n") {
      e.preventDefault();
      showAddModal.value = true;
    } else if (e.key === "f") {
      e.preventDefault();
      const searchInput = document.querySelector('input[placeholder*="搜索"]') as HTMLInputElement;
      if (searchInput) searchInput.focus();
    } else if (e.key === "a" && isSelectionMode.value) {
      e.preventDefault();
      filteredMachines.value.forEach((m) => selectedMachines.value.add(m.id));
    }
  }

  if (e.key === "Escape") {
    if (showAddModal.value) showAddModal.value = false;
    if (showEditModal.value) showEditModal.value = false;
    if (showDeleteConfirm.value) showDeleteConfirm.value = false;
    if (showBatchDeleteConfirm.value) showBatchDeleteConfirm.value = false;
    if (isSelectionMode.value) {
      isSelectionMode.value = false;
      selectedMachines.value.clear();
    }
  }
}

watch(showSkeleton, (val) => {
  if (!val) {
    setTimeout(() => testAllConnections(), 500);
  }
});

onMounted(() => {
  loadMachines();
  window.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleKeydown);
});
</script>

<template>
  <TooltipProvider>
    <div class="flex h-screen w-screen bg-canvas text-primary overflow-hidden">
      <aside class="mm-sidebar" data-tauri-drag-region>
        <div
          class="h-14 flex items-center px-5 gap-3 font-semibold select-none border-b border-subtle/50"
          style="-webkit-app-region: drag"
        >
          <div
            class="size-8 rounded-[10px] grid place-items-center bg-gradient-to-br from-[#0a84ff] to-[#0066cc] shadow-[0_2px_8px_rgba(10,132,255,0.35)]"
          >
            <span class="icon-[mdi--lightning-bolt] size-4.5 text-white"></span>
          </div>
          <span class="text-primary font-bold tracking-tight">SynapSH</span>
        </div>

        <nav class="flex-1 px-3 py-4 space-y-1">
          <a href="#" class="mm-nav-item mm-nav-item-active">
            <span class="icon-[lucide--server] size-4"></span>
            <span>机器</span>
            <span
              class="ml-auto text-[11px] font-semibold bg-white/20 dark:bg-white/10 px-2 py-0.5 rounded-full"
              >{{ machines.length }}</span
            >
          </a>
          <a href="#" class="mm-nav-item mm-nav-item-inactive">
            <span class="icon-[lucide--key-round] size-4"></span>
            <span>密钥管理</span>
          </a>
          <a href="#" class="mm-nav-item mm-nav-item-inactive">
            <span class="icon-[lucide--settings] size-4"></span>
            <span>设置</span>
          </a>
        </nav>

        <div class="px-4 py-3 border-t border-subtle/50">
          <div class="text-[11px] text-tertiary/70 flex items-center gap-1.5">
            <span class="size-1.5 rounded-full bg-success/60"></span>
            v1.0.0
          </div>
        </div>
      </aside>

      <main class="flex-1 flex flex-col bg-canvas overflow-hidden">
        <header class="mm-header" data-tauri-drag-region>
          <h1
            class="flex items-center gap-2.5 text-lg font-bold text-primary"
            style="-webkit-app-region: drag"
          >
            <span class="icon-[lucide--server] size-5 text-accent"></span>
            全部机器
            <Badge v-if="selectedCount > 0" variant="secondary" class="text-xs px-2">
              已选 {{ selectedCount }}
            </Badge>
          </h1>

          <div class="flex items-center gap-3">
            <div class="relative">
              <span
                class="icon-[lucide--search] size-4 absolute left-3 top-1/2 -translate-y-1/2 text-tertiary"
              ></span>
              <Input
                v-model="searchQuery"
                placeholder="搜索机器... (⌘F)"
                class="pl-9 w-48 h-9 app-input"
              />
            </div>

            <div
              class="flex items-center gap-1.5 bg-elevated rounded-lg px-2 py-1 border border-subtle"
            >
              <Tooltip>
                <TooltipTrigger as-child>
                  <Button
                    variant="ghost"
                    size="sm"
                    :class="[
                      'px-2 h-7 text-xs',
                      filterMode === 'all' && 'bg-accent/30 text-accent',
                    ]"
                    @click="filterMode = 'all'"
                  >
                    全部
                  </Button>
                </TooltipTrigger>
                <TooltipContent>显示全部机器</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger as-child>
                  <Button
                    variant="ghost"
                    size="sm"
                    :class="[
                      'px-2 h-7 text-xs',
                      filterMode === 'online' && 'bg-success/20 text-success',
                    ]"
                    @click="filterMode = 'online'"
                  >
                    在线
                  </Button>
                </TooltipTrigger>
                <TooltipContent>仅显示在线机器</TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger as-child>
                  <Button
                    variant="ghost"
                    size="sm"
                    :class="[
                      'px-2 h-7 text-xs',
                      filterMode === 'offline' && 'bg-danger/20 text-danger',
                    ]"
                    @click="filterMode = 'offline'"
                  >
                    离线
                  </Button>
                </TooltipTrigger>
                <TooltipContent>仅显示离线机器</TooltipContent>
              </Tooltip>
            </div>

            <Separator orientation="vertical" class="h-6" />

            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  :class="isSelectionMode && 'bg-accent/20 text-accent'"
                  @click="toggleSelectionMode"
                >
                  <span class="icon-[lucide--list-checks] size-4"></span>
                </Button>
              </TooltipTrigger>
              <TooltipContent>{{ isSelectionMode ? "退出选择模式" : "批量选择" }}</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="ghost"
                  size="icon-sm"
                  @click="testAllConnections"
                  :disabled="isLoading"
                >
                  <span
                    class="icon-[lucide--wifi] size-4"
                    :class="{ 'animate-pulse': isLoading }"
                  ></span>
                </Button>
              </TooltipTrigger>
              <TooltipContent>测试所有连接</TooltipContent>
            </Tooltip>

            <Tooltip>
              <TooltipTrigger as-child>
                <Button variant="ghost" size="icon-sm" @click="loadMachines" :disabled="isLoading">
                  <span
                    class="icon-[lucide--refresh-cw] size-4"
                    :class="{ 'animate-spin': isLoading }"
                  ></span>
                </Button>
              </TooltipTrigger>
              <TooltipContent>刷新列表</TooltipContent>
            </Tooltip>

            <Button size="sm" class="app-btn app-btn-primary" @click="showAddModal = true">
              <span class="icon-[lucide--plus] size-4"></span>
              添加机器
            </Button>
          </div>
        </header>

        <div
          v-if="isSelectionMode"
          class="mx-6 mt-3 p-3 bg-elevated border border-accent/30 rounded-xl flex justify-between items-center"
        >
          <div class="flex items-center gap-3">
            <span class="text-sm text-secondary">已选择 {{ selectedCount }} 台机器</span>
            <Button
              variant="ghost"
              size="sm"
              class="text-xs px-2"
              @click="filteredMachines.forEach((m) => selectedMachines.add(m.id))"
            >
              全选当前
            </Button>
            <Button variant="ghost" size="sm" class="text-xs px-2" @click="clearSelection">
              清除选择
            </Button>
          </div>
          <div class="flex gap-2">
            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="outline"
                  size="sm"
                  @click="testSelectedConnections"
                  :disabled="selectedCount === 0"
                >
                  <span class="icon-[lucide--wifi] size-4"></span>
                  测试连接
                </Button>
              </TooltipTrigger>
              <TooltipContent>测试选中机器的连接状态</TooltipContent>
            </Tooltip>
            <Tooltip>
              <TooltipTrigger as-child>
                <Button
                  variant="destructive"
                  size="sm"
                  @click="showBatchDeleteConfirm = true"
                  :disabled="selectedCount === 0"
                >
                  <span class="icon-[lucide--trash-2] size-4"></span>
                  批量删除
                </Button>
              </TooltipTrigger>
              <TooltipContent>删除选中的机器</TooltipContent>
            </Tooltip>
          </div>
        </div>

        <div class="px-6 py-3 flex items-center gap-3">
          <span class="text-xs text-tertiary">分组方式：</span>
          <div class="flex gap-1.5">
            <Button
              variant="ghost"
              size="sm"
              :class="['px-3 h-7 text-xs', groupMode === 'none' && 'bg-accent/30 text-accent']"
              @click="groupMode = 'none'"
            >
              无分组
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :class="['px-3 h-7 text-xs', groupMode === 'os' && 'bg-accent/30 text-accent']"
              @click="groupMode = 'os'"
            >
              按系统
            </Button>
            <Button
              variant="ghost"
              size="sm"
              :class="['px-3 h-7 text-xs', groupMode === 'tag' && 'bg-accent/30 text-accent']"
              @click="groupMode = 'tag'"
            >
              按标签
            </Button>
          </div>
        </div>

        <div
          v-if="error"
          class="mx-6 p-3 bg-danger/10 border border-danger/20 rounded-xl text-danger text-sm flex justify-between items-center"
        >
          <span>{{ error }}</span>
          <Button variant="ghost" size="icon-sm" @click="error = ''">
            <span class="icon-[lucide--x] size-4"></span>
          </Button>
        </div>

        <div v-if="showSkeleton && machines.length === 0" class="flex-1 overflow-y-auto p-6">
          <div class="grid grid-cols-[repeat(auto-fill,minmax(320px,1fr))] gap-4">
            <div v-for="i in 6" :key="i" class="app-card p-4 space-y-3">
              <div class="flex items-start gap-3">
                <div class="app-skeleton size-10 rounded-[10px]"></div>
                <div class="flex-1 space-y-2">
                  <div class="app-skeleton h-4 w-3/4"></div>
                  <div class="app-skeleton h-3 w-1/2"></div>
                </div>
              </div>
              <div class="flex gap-1.5">
                <div class="app-skeleton h-5 w-16"></div>
                <div class="app-skeleton h-5 w-20"></div>
              </div>
              <div class="app-divider"></div>
              <div class="app-skeleton h-7 w-24"></div>
            </div>
          </div>
        </div>

        <div
          v-else-if="isLoading && machines.length === 0"
          class="flex-1 flex flex-col items-center justify-center text-tertiary gap-3"
        >
          <span class="icon-[lucide--loader-2] size-8 animate-spin text-accent"></span>
          <p class="text-sm">加载中...</p>
        </div>

        <div
          v-else-if="machines.length === 0"
          class="flex-1 flex flex-col items-center justify-center text-tertiary"
        >
          <div
            class="size-20 rounded-2xl bg-active grid place-items-center mb-5 shadow-[0_10px_24px_rgba(0,0,0,0.30)]"
          >
            <span class="icon-[lucide--server-off] size-10 text-tertiary"></span>
          </div>
          <h3 class="text-base font-semibold text-primary mb-2">暂无机器</h3>
          <p class="text-sm mb-5 text-secondary">添加您的第一台服务器开始连接</p>
          <Button class="app-btn app-btn-primary" @click="showAddModal = true">
            <span class="icon-[lucide--plus] size-4"></span>
            添加机器
          </Button>
        </div>

        <div
          v-else-if="filteredMachines.length === 0"
          class="flex-1 flex flex-col items-center justify-center text-tertiary"
        >
          <div class="size-20 rounded-2xl bg-active grid place-items-center mb-5">
            <span class="icon-[lucide--search-x] size-10"></span>
          </div>
          <h3 class="text-base font-semibold text-primary mb-2">未找到匹配的机器</h3>
          <p class="text-sm text-secondary">尝试其他关键词或筛选条件</p>
        </div>

        <div v-else class="flex-1 overflow-y-auto p-6 space-y-6">
          <div v-for="group in groupedMachines" :key="group.key">
            <div v-if="groupMode !== 'none'" class="app-group-header mb-3">
              <span class="icon-[lucide--folder] size-3.5 text-tertiary"></span>
              <span>{{ group.label }}</span>
              <Badge variant="secondary" class="text-xs px-1.5">{{ group.machines.length }}</Badge>
              <Button
                v-if="isSelectionMode"
                variant="ghost"
                size="sm"
                class="ml-auto text-xs px-2 h-6"
                @click="selectAllInGroup(group.machines)"
              >
                全选
              </Button>
            </div>

            <div class="grid grid-cols-[repeat(auto-fill,minmax(340px,1fr))] gap-4">
              <div
                v-for="machine in group.machines"
                :key="machine.id"
                :class="[
                  'mm-card group',
                  isSelectionMode && selectedMachines.has(machine.id) && 'mm-card-selected',
                ]"
                @click="connectToMachine(machine)"
              >
                <!-- 卡片顶部：OS图标 + 信息 + 操作按钮 -->
                <div class="flex items-start gap-3.5">
                  <div v-if="isSelectionMode" class="flex items-center justify-center pt-0.5">
                    <Checkbox
                      :checked="selectedMachines.has(machine.id)"
                      class="app-checkbox"
                      @update:checked="toggleSelection(machine.id)"
                    />
                  </div>
                  <div v-else :class="['mm-os-icon', `mm-os-icon-${machine.os || 'linux'}`]">
                    <span
                      v-if="machine.os === 'windows'"
                      class="icon-[mdi--microsoft-windows] size-5 text-white"
                    ></span>
                    <span
                      v-else-if="machine.os === 'macos'"
                      class="icon-[mdi--apple] size-5 text-white"
                    ></span>
                    <span v-else class="icon-[mdi--linux] size-5 text-white"></span>
                  </div>

                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="font-semibold truncate text-primary text-[15px]">{{
                        machine.name || machine.host
                      }}</span>
                      <Tooltip>
                        <TooltipTrigger>
                          <span class="mm-status-dot" :class="getStatusColor(machine.id)"></span>
                        </TooltipTrigger>
                        <TooltipContent>{{ getStatusTitle(machine.id) }}</TooltipContent>
                      </Tooltip>
                    </div>
                    <p class="text-xs text-tertiary font-mono truncate mt-1">
                      {{ machine.username }}@{{ machine.host }}:{{ machine.port }}
                    </p>
                  </div>

                  <!-- 操作按钮：hover 可见 -->
                  <div
                    v-if="!isSelectionMode"
                    class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity duration-150"
                  >
                    <Tooltip>
                      <TooltipTrigger as-child>
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          class="size-7 rounded-md text-tertiary hover:text-accent hover:bg-accent/10"
                          @click.stop="testConnection(machine)"
                        >
                          <span class="icon-[lucide--wifi] size-3.5"></span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>测试连接</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger as-child>
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          class="size-7 rounded-md text-tertiary hover:text-primary hover:bg-active"
                          @click.stop="openEditModal(machine)"
                        >
                          <span class="icon-[lucide--pencil] size-3.5"></span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>编辑</TooltipContent>
                    </Tooltip>
                    <Tooltip>
                      <TooltipTrigger as-child>
                        <Button
                          variant="ghost"
                          size="icon-sm"
                          class="size-7 rounded-md text-tertiary hover:text-danger hover:bg-danger/10"
                          @click.stop="openDeleteConfirm(machine)"
                        >
                          <span class="icon-[lucide--trash-2] size-3.5"></span>
                        </Button>
                      </TooltipTrigger>
                      <TooltipContent>删除</TooltipContent>
                    </Tooltip>
                  </div>
                </div>

                <!-- 标签区 -->
                <div
                  v-if="parseTags(machine.tags).length > 0"
                  class="flex items-center gap-1.5 flex-wrap mt-3"
                >
                  <span
                    v-for="tag in parseTags(machine.tags).slice(0, 3)"
                    :key="tag"
                    class="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-accent/10 text-accent border border-accent/20"
                  >
                    {{ tag }}
                  </span>
                  <Tooltip v-if="parseTags(machine.tags).length > 3">
                    <TooltipTrigger>
                      <span class="text-[11px] text-tertiary cursor-help">
                        +{{ parseTags(machine.tags).length - 3 }}
                      </span>
                    </TooltipTrigger>
                    <TooltipContent>
                      <div class="flex gap-1">
                        <span
                          v-for="tag in parseTags(machine.tags).slice(3)"
                          :key="tag"
                          class="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-accent/10 text-accent"
                        >
                          {{ tag }}
                        </span>
                      </div>
                    </TooltipContent>
                  </Tooltip>
                </div>

                <!-- 底部：时间 + 连接按钮 -->
                <div class="flex justify-between items-center mt-4 pt-3 border-t border-subtle/60">
                  <span class="text-[11px] text-tertiary/80 flex items-center gap-1.5">
                    <span class="icon-[lucide--clock] size-3"></span>
                    {{ formatLastUpdated(machine.updatedAt) }}
                  </span>
                  <button
                    v-if="!isSelectionMode"
                    class="mm-connect-btn"
                    @click.stop="connectToMachine(machine)"
                    :disabled="isConnecting === machine.id"
                  >
                    <span
                      v-if="isConnecting === machine.id"
                      class="icon-[lucide--loader-2] size-3.5 animate-spin"
                    ></span>
                    <span v-else class="icon-[lucide--terminal] size-3.5"></span>
                    {{ isConnecting === machine.id ? "连接中..." : "连接" }}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>

      <Dialog v-model:open="showAddModal">
        <DialogContent
          class="sm:max-w-md bg-surface border border-subtle rounded-xl shadow-[0_24px_64px_rgba(0,0,0,0.55)]"
        >
          <DialogHeader>
            <DialogTitle class="text-primary">添加机器</DialogTitle>
            <DialogDescription class="text-secondary">填写 SSH 连接信息</DialogDescription>
          </DialogHeader>

          <div class="space-y-4 py-4">
            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">名称</label>
              <Input v-model="newMachine.name" placeholder="例如：生产服务器" class="app-input" />
            </div>

            <div class="flex gap-3">
              <div class="flex-1 space-y-1.5">
                <label class="text-sm font-medium text-secondary"
                  >主机地址 <span class="text-danger">*</span></label
                >
                <Input v-model="newMachine.host" placeholder="IP 或域名" class="app-input" />
              </div>
              <div class="w-24 space-y-1.5">
                <label class="text-sm font-medium text-secondary">端口</label>
                <Input v-model.number="newMachine.port" type="number" class="app-input" />
              </div>
            </div>

            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary"
                >用户名 <span class="text-danger">*</span></label
              >
              <Input v-model="newMachine.username" placeholder="root" class="app-input" />
            </div>

            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">认证方式</label>
              <div class="flex gap-2">
                <Button
                  :variant="newMachine.authType === 'password' ? 'default' : 'outline'"
                  size="sm"
                  :class="[
                    'flex-1 app-btn',
                    newMachine.authType === 'password' && 'app-btn-primary',
                  ]"
                  @click="newMachine.authType = 'password'"
                  >密码</Button
                >
                <Button
                  :variant="newMachine.authType === 'key' ? 'default' : 'outline'"
                  size="sm"
                  :class="['flex-1 app-btn', newMachine.authType === 'key' && 'app-btn-primary']"
                  @click="newMachine.authType = 'key'"
                  >密钥</Button
                >
              </div>
            </div>

            <div v-if="newMachine.authType === 'password'" class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">密码</label>
              <Input
                v-model="newMachine.password"
                type="password"
                placeholder="SSH 密码"
                class="app-input"
              />
            </div>
            <div v-else class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">私钥路径</label>
              <Input
                v-model="newMachine.privateKeyPath"
                placeholder="~/.ssh/id_rsa"
                class="app-input"
              />
            </div>

            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">操作系统</label>
              <div class="flex gap-2">
                <Button
                  :variant="newMachine.os === 'linux' ? 'default' : 'outline'"
                  size="sm"
                  :class="['flex-1 app-btn', newMachine.os === 'linux' && 'app-btn-primary']"
                  @click="newMachine.os = 'linux'"
                  >Linux</Button
                >
                <Button
                  :variant="newMachine.os === 'windows' ? 'default' : 'outline'"
                  size="sm"
                  :class="['flex-1 app-btn', newMachine.os === 'windows' && 'app-btn-primary']"
                  @click="newMachine.os = 'windows'"
                  >Windows</Button
                >
                <Button
                  :variant="newMachine.os === 'macos' ? 'default' : 'outline'"
                  size="sm"
                  :class="['flex-1 app-btn', newMachine.os === 'macos' && 'app-btn-primary']"
                  @click="newMachine.os = 'macos'"
                  >macOS</Button
                >
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              class="app-btn app-btn-secondary"
              @click="
                showAddModal = false;
                resetForm();
              "
              >取消</Button
            >
            <Button
              class="app-btn app-btn-primary"
              @click="handleAddMachine"
              :disabled="!newMachine.host || !newMachine.username"
              >添加</Button
            >
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog v-model:open="showEditModal">
        <DialogContent
          v-if="editingMachine"
          class="sm:max-w-md bg-surface border border-subtle rounded-xl shadow-[0_24px_64px_rgba(0,0,0,0.55)]"
        >
          <DialogHeader>
            <DialogTitle class="text-primary">编辑机器</DialogTitle>
          </DialogHeader>

          <div class="space-y-4 py-4">
            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">名称</label>
              <Input v-model="editingMachine.name" class="app-input" />
            </div>
            <div class="flex gap-3">
              <div class="flex-1 space-y-1.5">
                <label class="text-sm font-medium text-secondary">主机地址</label>
                <Input v-model="editingMachine.host" class="app-input" />
              </div>
              <div class="w-24 space-y-1.5">
                <label class="text-sm font-medium text-secondary">端口</label>
                <Input v-model.number="editingMachine.port" type="number" class="app-input" />
              </div>
            </div>
            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">用户名</label>
              <Input v-model="editingMachine.username" class="app-input" />
            </div>
            <div class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">认证方式</label>
              <div class="flex gap-2">
                <Button
                  :variant="editingMachine.authType === 'password' ? 'default' : 'outline'"
                  size="sm"
                  :class="[
                    'flex-1 app-btn',
                    editingMachine.authType === 'password' && 'app-btn-primary',
                  ]"
                  @click="editingMachine.authType = 'password'"
                  >密码</Button
                >
                <Button
                  :variant="editingMachine.authType === 'key' ? 'default' : 'outline'"
                  size="sm"
                  :class="[
                    'flex-1 app-btn',
                    editingMachine.authType === 'key' && 'app-btn-primary',
                  ]"
                  @click="editingMachine.authType = 'key'"
                  >密钥</Button
                >
              </div>
            </div>
            <div v-if="editingMachine.authType === 'password'" class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">密码</label>
              <Input
                v-model="editingMachine.password"
                type="password"
                placeholder="留空保持不变"
                class="app-input"
              />
            </div>
            <div v-else class="space-y-1.5">
              <label class="text-sm font-medium text-secondary">私钥路径</label>
              <Input v-model="editingMachine.privateKeyPath" class="app-input" />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              class="app-btn app-btn-secondary"
              @click="showEditModal = false"
              >取消</Button
            >
            <Button class="app-btn app-btn-primary" @click="handleUpdateMachine">保存</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog v-model:open="showDeleteConfirm">
        <DialogContent
          v-if="deletingMachine"
          class="sm:max-w-sm bg-surface border border-subtle rounded-xl shadow-[0_24px_64px_rgba(0,0,0,0.55)]"
        >
          <DialogHeader class="text-center">
            <div class="size-14 rounded-full bg-danger/15 grid place-items-center mx-auto mb-3">
              <span class="icon-[lucide--trash-2] size-7 text-danger"></span>
            </div>
            <DialogTitle class="text-primary">确认删除</DialogTitle>
            <DialogDescription class="text-secondary">
              确定删除
              <strong class="text-primary">{{
                deletingMachine.name || deletingMachine.host
              }}</strong
              >？此操作不可恢复。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter class="sm:justify-center gap-3">
            <Button
              variant="outline"
              class="app-btn app-btn-secondary"
              @click="showDeleteConfirm = false"
              >取消</Button
            >
            <Button variant="destructive" class="app-btn" @click="handleDeleteMachine">删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog v-model:open="showBatchDeleteConfirm">
        <DialogContent
          class="sm:max-w-sm bg-surface border border-subtle rounded-xl shadow-[0_24px_64px_rgba(0,0,0,0.55)]"
        >
          <DialogHeader class="text-center">
            <div class="size-14 rounded-full bg-danger/15 grid place-items-center mx-auto mb-3">
              <span class="icon-[lucide--trash-2] size-7 text-danger"></span>
            </div>
            <DialogTitle class="text-primary">批量删除确认</DialogTitle>
            <DialogDescription class="text-secondary">
              确定删除选中的
              <strong class="text-danger">{{ selectedCount }}</strong> 台机器？此操作不可恢复。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter class="sm:justify-center gap-3">
            <Button
              variant="outline"
              class="app-btn app-btn-secondary"
              @click="showBatchDeleteConfirm = false"
              >取消</Button
            >
            <Button variant="destructive" class="app-btn" @click="handleBatchDelete">删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  </TooltipProvider>
</template>
