<script setup lang="ts">
/**
 * MachineManager.vue - 机器管理组件
 * 
 * 使用 shadcn-vue 组件 + 自定义卡片布局
 */
import { ref, onMounted, computed } from "vue";
import { api } from "@/lib/api";

// shadcn-vue components
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { Separator } from "@/components/ui/separator";

const emit = defineEmits<{
    connect: [
        payload: {
            sessionId: string;
            machineId: string;
            machineName: string;
            host: string;
        }
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

type ConnectionStatus = 'unknown' | 'testing' | 'online' | 'offline';

const machines = ref<Machine[]>([]);
const machineStatus = ref<Record<string, ConnectionStatus>>({});
const isLoading = ref(false);
const error = ref("");
const searchQuery = ref("");

const showAddModal = ref(false);
const showEditModal = ref(false);
const showDeleteConfirm = ref(false);
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

const filteredMachines = computed(() => {
    if (!searchQuery.value.trim()) {
        return machines.value;
    }
    const query = searchQuery.value.toLowerCase();
    return machines.value.filter(m =>
        m.name.toLowerCase().includes(query) ||
        m.host.toLowerCase().includes(query) ||
        m.username.toLowerCase().includes(query) ||
        parseTags(m.tags).some(t => t.toLowerCase().includes(query))
    );
});

function getStatusColor(machineId: string): string {
    const status = machineStatus.value[machineId] || 'unknown';
    switch (status) {
        case 'online': return 'bg-green-500';
        case 'offline': return 'bg-red-500';
        case 'testing': return 'bg-yellow-500 animate-pulse';
        default: return 'bg-gray-300 dark:bg-gray-600';
    }
}

function getStatusTitle(machineId: string): string {
    const status = machineStatus.value[machineId] || 'unknown';
    switch (status) {
        case 'online': return '在线';
        case 'offline': return '离线';
        case 'testing': return '测试中...';
        default: return '未知状态';
    }
}

async function loadMachines() {
    isLoading.value = true;
    error.value = "";
    try {
        machines.value = await api.listMachines();
        machineStatus.value = {};
    } catch (e) {
        error.value = String(e);
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
        const idx = machines.value.findIndex(m => m.id === updated.id);
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
        machines.value = machines.value.filter(m => m.id !== deletingMachine.value!.id);
        showDeleteConfirm.value = false;
        deletingMachine.value = null;
    } catch (e) {
        error.value = String(e);
    }
}

async function testConnection(machine: Machine) {
    machineStatus.value[machine.id] = 'testing';
    try {
        const result = await api.testConnection({
            host: machine.host,
            port: machine.port,
            username: machine.username,
            password: machine.password,
            privateKey: machine.privateKeyPath
        });
        machineStatus.value[machine.id] = result ? 'online' : 'offline';
    } catch (e) {
        machineStatus.value[machine.id] = 'offline';
    }
}

async function testAllConnections() {
    for (const machine of machines.value) {
        testConnection(machine);
    }
}

async function connectToMachine(machine: Machine) {
    isConnecting.value = machine.id;
    const sessionId = `session_${machine.id}_${Date.now()}`;
    
    // 根据认证类型选择正确的凭据，避免空字符串导致后端误判
    const isKeyAuth = machine.authType === 'key';
    const password = !isKeyAuth && machine.password ? machine.password : undefined;
    const privateKey = isKeyAuth && machine.privateKeyPath ? machine.privateKeyPath : undefined;
    
    try {
        await Promise.race([
            api.connectSSH(sessionId, {
                host: machine.host,
                port: machine.port,
                username: machine.username,
                password: password,
                privateKey: privateKey
            }),
            new Promise((_, reject) => setTimeout(() => reject(new Error("连接超时 (15秒)")), 15000))
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
        machineStatus.value[machine.id] = 'offline';
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

    if (diffMins < 1) return '刚刚';
    if (diffMins < 60) return `${diffMins} 分钟前`;
    if (diffHours < 24) return `${diffHours} 小时前`;
    if (diffDays < 7) return `${diffDays} 天前`;
    return date.toLocaleDateString('zh-CN');
}

onMounted(() => {
    loadMachines();
});
</script>

<template>
    <TooltipProvider>
        <div class="flex h-screen w-screen bg-background text-foreground overflow-hidden">
            <!-- Sidebar -->
            <aside class="w-56 bg-card border-r border-border flex flex-col shrink-0" data-tauri-drag-region>
                <div class="h-14 flex items-center px-5 gap-2.5 font-semibold select-none"
                    style="-webkit-app-region: drag">
                    <div class="size-7 bg-primary text-primary-foreground rounded-lg grid place-items-center text-sm">⚡
                    </div>
                    <span>SynapSH</span>
                </div>

                <nav class="flex-1 px-3 py-2 space-y-1">
                    <a href="#"
                        class="flex items-center gap-2.5 px-3 py-2 rounded-lg bg-accent text-accent-foreground text-sm font-medium">
                        <span class="icon-[lucide--server] size-4"></span>
                        <span>机器</span>
                        <Badge variant="secondary" class="ml-auto text-xs">{{ machines.length }}</Badge>
                    </a>
                    <a href="#"
                        class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-muted-foreground hover:bg-accent/50 hover:text-foreground text-sm transition-colors">
                        <span class="icon-[lucide--key-round] size-4"></span>
                        <span>密钥管理</span>
                    </a>
                    <a href="#"
                        class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-muted-foreground hover:bg-accent/50 hover:text-foreground text-sm transition-colors">
                        <span class="icon-[lucide--settings] size-4"></span>
                        <span>设置</span>
                    </a>
                </nav>

                <Separator />
                <div class="p-4 text-xs text-muted-foreground">v1.0.0</div>
            </aside>

            <!-- Main Content -->
            <main class="flex-1 flex flex-col bg-muted/40 overflow-hidden">
                <!-- Header -->
                <header class="h-14 flex items-center justify-between px-6 border-b border-border bg-background"
                    data-tauri-drag-region>
                    <h1 class="text-lg font-semibold flex items-center gap-2" style="-webkit-app-region: drag">
                        <span class="icon-[lucide--server] size-5 text-primary"></span>
                        全部机器
                    </h1>

                    <div class="flex items-center gap-2">
                        <!-- Search -->
                        <div class="relative">
                            <span
                                class="icon-[lucide--search] size-4 absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"></span>
                            <Input v-model="searchQuery" placeholder="搜索机器..." class="pl-9 w-44 h-8" />
                        </div>

                        <Tooltip>
                            <TooltipTrigger as-child>
                                <Button variant="outline" size="sm" @click="testAllConnections">
                                    <span class="icon-[lucide--wifi] size-4"></span>
                                    测试连接
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>测试所有机器连接</TooltipContent>
                        </Tooltip>

                        <Tooltip>
                            <TooltipTrigger as-child>
                                <Button variant="outline" size="icon-sm" @click="loadMachines" :disabled="isLoading">
                                    <span class="icon-[lucide--refresh-cw] size-4"
                                        :class="{ 'animate-spin': isLoading }"></span>
                                </Button>
                            </TooltipTrigger>
                            <TooltipContent>刷新</TooltipContent>
                        </Tooltip>

                        <Button size="sm" @click="showAddModal = true">
                            <span class="icon-[lucide--plus] size-4"></span>
                            添加机器
                        </Button>
                    </div>
                </header>

                <!-- Error Bar -->
                <div v-if="error"
                    class="mx-6 mt-4 p-3 bg-destructive/10 border border-destructive/20 rounded-lg text-destructive text-sm flex justify-between items-center">
                    <span>{{ error }}</span>
                    <Button variant="ghost" size="icon-sm" @click="error = ''">
                        <span class="icon-[lucide--x] size-4"></span>
                    </Button>
                </div>

                <!-- Loading State -->
                <div v-if="isLoading && machines.length === 0"
                    class="flex-1 flex flex-col items-center justify-center text-muted-foreground gap-3">
                    <span class="icon-[lucide--loader-2] size-8 animate-spin"></span>
                    <p class="text-sm">加载中...</p>
                </div>

                <!-- Empty State -->
                <div v-else-if="machines.length === 0"
                    class="flex-1 flex flex-col items-center justify-center text-muted-foreground">
                    <div class="size-16 rounded-xl bg-muted grid place-items-center mb-4">
                        <span class="icon-[lucide--server-off] size-8"></span>
                    </div>
                    <h3 class="text-base font-semibold text-foreground mb-1">暂无机器</h3>
                    <p class="text-sm mb-4">添加您的第一台服务器</p>
                    <Button @click="showAddModal = true">
                        <span class="icon-[lucide--plus] size-4"></span>
                        添加机器
                    </Button>
                </div>

                <!-- No Results -->
                <div v-else-if="filteredMachines.length === 0"
                    class="flex-1 flex flex-col items-center justify-center text-muted-foreground">
                    <span class="icon-[lucide--search-x] size-10 mb-3"></span>
                    <h3 class="text-base font-semibold text-foreground mb-1">未找到匹配的机器</h3>
                    <p class="text-sm">尝试其他关键词</p>
                </div>

                <!-- Machines Grid -->
                <div v-else class="flex-1 overflow-y-auto p-6">
                    <div class="grid grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-4">
                        <div v-for="machine in filteredMachines" :key="machine.id"
                            class="group bg-card border border-border rounded-xl p-4 cursor-pointer transition-all hover:border-primary/50 hover:shadow-md"
                            @dblclick="connectToMachine(machine)">
                            <!-- Card Header -->
                            <div class="flex items-start gap-3 mb-3">
                                <!-- OS Icon -->
                                <div class="size-10 rounded-lg bg-muted grid place-items-center text-muted-foreground">
                                    <span v-if="machine.os === 'windows'"
                                        class="icon-[mdi--microsoft-windows] size-5"></span>
                                    <span v-else-if="machine.os === 'macos'" class="icon-[mdi--apple] size-5"></span>
                                    <span v-else class="icon-[mdi--linux] size-5"></span>
                                </div>

                                <!-- Info -->
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2">
                                        <span class="font-medium truncate">{{ machine.name || machine.host }}</span>
                                        <Tooltip>
                                            <TooltipTrigger>
                                                <span class="size-2 rounded-full shrink-0"
                                                    :class="getStatusColor(machine.id)"></span>
                                            </TooltipTrigger>
                                            <TooltipContent>{{ getStatusTitle(machine.id) }}</TooltipContent>
                                        </Tooltip>
                                    </div>
                                    <p class="text-xs text-muted-foreground font-mono truncate mt-0.5">
                                        {{ machine.username }}@{{ machine.host }}:{{ machine.port }}
                                    </p>
                                </div>

                                <!-- Action Buttons -->
                                <div class="flex gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                                    <Tooltip>
                                        <TooltipTrigger as-child>
                                            <Button variant="ghost" size="icon-sm"
                                                @click.stop="testConnection(machine)">
                                                <span class="icon-[lucide--wifi] size-4"></span>
                                            </Button>
                                        </TooltipTrigger>
                                        <TooltipContent>测试连接</TooltipContent>
                                    </Tooltip>
                                    <Tooltip>
                                        <TooltipTrigger as-child>
                                            <Button variant="ghost" size="icon-sm" @click.stop="openEditModal(machine)">
                                                <span class="icon-[lucide--pencil] size-4"></span>
                                            </Button>
                                        </TooltipTrigger>
                                        <TooltipContent>编辑</TooltipContent>
                                    </Tooltip>
                                    <Tooltip>
                                        <TooltipTrigger as-child>
                                            <Button variant="ghost" size="icon-sm" class="hover:text-destructive"
                                                @click.stop="openDeleteConfirm(machine)">
                                                <span class="icon-[lucide--trash-2] size-4"></span>
                                            </Button>
                                        </TooltipTrigger>
                                        <TooltipContent>删除</TooltipContent>
                                    </Tooltip>
                                </div>
                            </div>

                            <!-- Tags -->
                            <div v-if="parseTags(machine.tags).length > 0"
                                class="flex items-center gap-1.5 flex-wrap mb-3">
                                <Badge v-for="tag in parseTags(machine.tags).slice(0, 3)" :key="tag" variant="secondary"
                                    class="text-xs">{{ tag }}</Badge>
                                <span v-if="parseTags(machine.tags).length > 3"
                                    class="text-xs text-muted-foreground">+{{ parseTags(machine.tags).length - 3
                                    }}</span>
                            </div>

                            <!-- Card Footer -->
                            <div class="flex justify-between items-center pt-3 border-t border-border">
                                <span class="text-xs text-muted-foreground flex items-center gap-1">
                                    <span class="icon-[lucide--clock] size-3"></span>
                                    {{ formatLastUpdated(machine.updatedAt) }}
                                </span>
                                <Button size="sm" @click.stop="connectToMachine(machine)"
                                    :disabled="isConnecting === machine.id">
                                    <span v-if="isConnecting === machine.id"
                                        class="icon-[lucide--loader-2] size-4 animate-spin"></span>
                                    <span v-else class="icon-[lucide--terminal] size-4"></span>
                                    {{ isConnecting === machine.id ? '连接中...' : '连接' }}
                                </Button>
                            </div>
                        </div>
                    </div>
                </div>
            </main>

            <!-- Add Modal -->
            <Dialog v-model:open="showAddModal">
                <DialogContent class="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle>添加机器</DialogTitle>
                        <DialogDescription>填写 SSH 连接信息</DialogDescription>
                    </DialogHeader>

                    <div class="space-y-4 py-4">
                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">名称</label>
                            <Input v-model="newMachine.name" placeholder="例如：生产服务器" />
                        </div>

                        <div class="flex gap-3">
                            <div class="flex-1 space-y-1.5">
                                <label class="text-sm font-medium">主机地址 <span class="text-destructive">*</span></label>
                                <Input v-model="newMachine.host" placeholder="IP 或域名" />
                            </div>
                            <div class="w-20 space-y-1.5">
                                <label class="text-sm font-medium">端口</label>
                                <Input v-model.number="newMachine.port" type="number" />
                            </div>
                        </div>

                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">用户名 <span class="text-destructive">*</span></label>
                            <Input v-model="newMachine.username" placeholder="root" />
                        </div>

                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">认证方式</label>
                            <div class="flex gap-2">
                                <Button :variant="newMachine.authType === 'password' ? 'default' : 'outline'" size="sm"
                                    class="flex-1" @click="newMachine.authType = 'password'">密码</Button>
                                <Button :variant="newMachine.authType === 'key' ? 'default' : 'outline'" size="sm"
                                    class="flex-1" @click="newMachine.authType = 'key'">密钥</Button>
                            </div>
                        </div>

                        <div v-if="newMachine.authType === 'password'" class="space-y-1.5">
                            <label class="text-sm font-medium">密码</label>
                            <Input v-model="newMachine.password" type="password" placeholder="SSH 密码" />
                        </div>
                        <div v-else class="space-y-1.5">
                            <label class="text-sm font-medium">私钥路径</label>
                            <Input v-model="newMachine.privateKeyPath" placeholder="~/.ssh/id_rsa" />
                        </div>

                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">操作系统</label>
                            <div class="flex gap-2">
                                <Button :variant="newMachine.os === 'linux' ? 'default' : 'outline'" size="sm"
                                    class="flex-1" @click="newMachine.os = 'linux'">Linux</Button>
                                <Button :variant="newMachine.os === 'windows' ? 'default' : 'outline'" size="sm"
                                    class="flex-1" @click="newMachine.os = 'windows'">Windows</Button>
                                <Button :variant="newMachine.os === 'macos' ? 'default' : 'outline'" size="sm"
                                    class="flex-1" @click="newMachine.os = 'macos'">macOS</Button>
                            </div>
                        </div>
                    </div>

                    <DialogFooter>
                        <Button variant="outline" @click="showAddModal = false; resetForm()">取消</Button>
                        <Button @click="handleAddMachine"
                            :disabled="!newMachine.host || !newMachine.username">添加</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            <!-- Edit Modal -->
            <Dialog v-model:open="showEditModal">
                <DialogContent v-if="editingMachine" class="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle>编辑机器</DialogTitle>
                    </DialogHeader>

                    <div class="space-y-4 py-4">
                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">名称</label>
                            <Input v-model="editingMachine.name" />
                        </div>
                        <div class="flex gap-3">
                            <div class="flex-1 space-y-1.5">
                                <label class="text-sm font-medium">主机地址</label>
                                <Input v-model="editingMachine.host" />
                            </div>
                            <div class="w-20 space-y-1.5">
                                <label class="text-sm font-medium">端口</label>
                                <Input v-model.number="editingMachine.port" type="number" />
                            </div>
                        </div>
                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">用户名</label>
                            <Input v-model="editingMachine.username" />
                        </div>
                        <div class="space-y-1.5">
                            <label class="text-sm font-medium">认证方式</label>
                            <div class="flex gap-2">
                                <Button :variant="editingMachine.authType === 'password' ? 'default' : 'outline'"
                                    size="sm" class="flex-1" @click="editingMachine.authType = 'password'">密码</Button>
                                <Button :variant="editingMachine.authType === 'key' ? 'default' : 'outline'" size="sm"
                                    class="flex-1" @click="editingMachine.authType = 'key'">密钥</Button>
                            </div>
                        </div>
                        <div v-if="editingMachine.authType === 'password'" class="space-y-1.5">
                            <label class="text-sm font-medium">密码</label>
                            <Input v-model="editingMachine.password" type="password" placeholder="留空保持不变" />
                        </div>
                        <div v-else class="space-y-1.5">
                            <label class="text-sm font-medium">私钥路径</label>
                            <Input v-model="editingMachine.privateKeyPath" />
                        </div>
                    </div>

                    <DialogFooter>
                        <Button variant="outline" @click="showEditModal = false">取消</Button>
                        <Button @click="handleUpdateMachine">保存</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>

            <!-- Delete Confirm Modal -->
            <Dialog v-model:open="showDeleteConfirm">
                <DialogContent v-if="deletingMachine" class="sm:max-w-sm">
                    <DialogHeader class="text-center">
                        <div class="size-12 rounded-full bg-destructive/10 grid place-items-center mx-auto mb-2">
                            <span class="icon-[lucide--trash-2] size-6 text-destructive"></span>
                        </div>
                        <DialogTitle>确认删除</DialogTitle>
                        <DialogDescription>
                            确定删除 <strong>{{ deletingMachine.name || deletingMachine.host }}</strong>？此操作不可恢复。
                        </DialogDescription>
                    </DialogHeader>
                    <DialogFooter class="sm:justify-center gap-2">
                        <Button variant="outline" @click="showDeleteConfirm = false">取消</Button>
                        <Button variant="destructive" @click="handleDeleteMachine">删除</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    </TooltipProvider>
</template>
