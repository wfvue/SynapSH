<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
    connect: [sessionId: string];
}>();

interface Machine {
    id: string;
    name: string;
    host: string;
    port: number;
    username: string;
    password?: string;
    private_key_path?: string;
    auth_type: string;
    tags: string;
    os: string;
    created_at: string;
    updated_at: string;
}

interface MachineInput {
    name?: string;
    host: string;
    port?: number;
    username: string;
    password?: string;
    private_key_path?: string;
    auth_type: string;
    tags?: string[];
    os?: string;
}

const machines = ref<Machine[]>([]);
const isLoading = ref(false);
const error = ref("");
const searchQuery = ref("");

// 模态框状态
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
    private_key_path: "",
    auth_type: "password",
    tags: [],
    os: "linux",
});

const isConnecting = ref<string | null>(null);
const isTesting = ref<string | null>(null);

// 过滤后的机器列表
const filteredMachines = computed(() => {
    if (!searchQuery.value.trim()) {
        return machines.value;
    }
    const query = searchQuery.value.toLowerCase();
    return machines.value.filter(m =>
        m.name.toLowerCase().includes(query) ||
        m.host.toLowerCase().includes(query) ||
        m.username.toLowerCase().includes(query)
    );
});

// 加载机器列表
async function loadMachines() {
    isLoading.value = true;
    error.value = "";
    try {
        machines.value = await invoke<Machine[]>("list_machines");
    } catch (e) {
        error.value = String(e);
        console.error("加载机器列表失败:", e);
    } finally {
        isLoading.value = false;
    }
}

// 添加机器
async function handleAddMachine() {
    try {
        const machine = await invoke<Machine>("add_machine", { input: newMachine.value });
        machines.value.unshift(machine);
        showAddModal.value = false;
        resetForm();
    } catch (e) {
        error.value = String(e);
        console.error("添加机器失败:", e);
    }
}

// 更新机器
async function handleUpdateMachine() {
    if (!editingMachine.value) return;
    try {
        const input: MachineInput = {
            name: editingMachine.value.name,
            host: editingMachine.value.host,
            port: editingMachine.value.port,
            username: editingMachine.value.username,
            password: editingMachine.value.password,
            private_key_path: editingMachine.value.private_key_path,
            auth_type: editingMachine.value.auth_type,
            os: editingMachine.value.os,
        };
        const updated = await invoke<Machine>("update_machine", {
            id: editingMachine.value.id,
            input
        });
        const idx = machines.value.findIndex(m => m.id === updated.id);
        if (idx !== -1) {
            machines.value[idx] = updated;
        }
        showEditModal.value = false;
        editingMachine.value = null;
    } catch (e) {
        error.value = String(e);
        console.error("更新机器失败:", e);
    }
}

// 删除机器
async function handleDeleteMachine() {
    if (!deletingMachine.value) return;
    try {
        await invoke("delete_machine", { id: deletingMachine.value.id });
        machines.value = machines.value.filter(m => m.id !== deletingMachine.value!.id);
        showDeleteConfirm.value = false;
        deletingMachine.value = null;
    } catch (e) {
        error.value = String(e);
        console.error("删除机器失败:", e);
    }
}

// 测试连接
async function testConnection(machine: Machine) {
    isTesting.value = machine.id;
    try {
        const result = await invoke<boolean>("test_connection", {
            host: machine.host,
            port: machine.port,
            username: machine.username,
            password: machine.password,
            privateKey: machine.private_key_path
        });
        if (result) {
            alert("连接成功！");
        } else {
            alert("认证失败");
        }
    } catch (e) {
        alert("连接失败: " + String(e));
    } finally {
        isTesting.value = null;
    }
}

// 连接到机器
async function connectToMachine(machine: Machine) {
    isConnecting.value = machine.id;
    const sessionId = `session_${machine.id}_${Date.now()}`;
    try {
        await invoke("connect_ssh", {
            sessionId,
            params: {
                host: machine.host,
                port: machine.port,
                username: machine.username,
                password: machine.password || null,
                private_key: machine.private_key_path || null
            }
        });
        emit("connect", sessionId);
    } catch (e) {
        error.value = String(e);
        console.error("连接失败:", e);
        alert("连接失败: " + String(e));
    } finally {
        isConnecting.value = null;
    }
}

// 打开编辑模态框
function openEditModal(machine: Machine) {
    editingMachine.value = { ...machine };
    showEditModal.value = true;
}

// 打开删除确认
function openDeleteConfirm(machine: Machine) {
    deletingMachine.value = machine;
    showDeleteConfirm.value = true;
}

// 重置表单
function resetForm() {
    newMachine.value = {
        name: "",
        host: "",
        port: 22,
        username: "root",
        password: "",
        private_key_path: "",
        auth_type: "password",
        tags: [],
        os: "linux",
    };
}

// 解析 tags JSON
function parseTags(tagsJson: string): string[] {
    try {
        return JSON.parse(tagsJson);
    } catch {
        return [];
    }
}

onMounted(() => {
    loadMachines();
});
</script>

<template>
    <div class="machine-manager">
        <aside class="sidebar">
            <div class="brand">
                <div class="logo">⚡</div>
                <span>SynapSH</span>
            </div>
            <nav class="nav-menu">
                <a href="#" class="nav-item active">
                    <i class="icon-server">🖥</i>
                    机器管理
                </a>
                <a href="#" class="nav-item">
                    <i class="icon-terminal">>_</i>
                    命令中心
                </a>
                <a href="#" class="nav-item">
                    <i class="icon-batch">📋</i>
                    批处理任务
                </a>
                <a href="#" class="nav-item">
                    <i class="icon-settings">⚙</i>
                    设置
                </a>
            </nav>
            <div class="sidebar-footer">
                <div class="local-service">
                    <span>本地服务</span>
                    <div class="toggle-switch active"></div>
                </div>
            </div>
        </aside>

        <main class="main-content">
            <header class="top-bar">
                <div class="breadcrumbs">
                    <span class="crumb active">全部机器</span>
                    <span class="machine-count">({{ filteredMachines.length }})</span>
                </div>
                <div class="top-actions">
                    <button class="btn-refresh" @click="loadMachines" :disabled="isLoading">
                        {{ isLoading ? '刷新中...' : '🔄 刷新' }}
                    </button>
                </div>
            </header>

            <div class="toolbar">
                <div class="left-tools">
                    <button class="btn-primary" @click="showAddModal = true">
                        <span class="icon">+</span> 添加机器
                    </button>

                    <div class="search-box">
                        <span class="icon">🔍</span>
                        <input v-model="searchQuery" type="text" placeholder="搜索机器名称、IP或用户名" />
                    </div>
                </div>
            </div>

            <!-- 错误提示 -->
            <div v-if="error" class="error-bar">
                {{ error }}
                <button @click="error = ''">✕</button>
            </div>

            <!-- 加载状态 -->
            <div v-if="isLoading && machines.length === 0" class="loading-state">
                <div class="spinner"></div>
                <p>加载中...</p>
            </div>

            <!-- 空状态 -->
            <div v-else-if="machines.length === 0" class="empty-state">
                <div class="empty-icon">📡</div>
                <h3>暂无机器</h3>
                <p>点击上方"添加机器"按钮添加您的第一台服务器</p>
            </div>

            <!-- 机器网格 -->
            <div v-else class="machine-grid">
                <div v-for="machine in filteredMachines" :key="machine.id" class="machine-card">
                    <div class="card-header">
                        <div class="os-icon" :class="machine.os">
                            <span v-if="machine.os === 'linux'">🐧</span>
                            <span v-else-if="machine.os === 'windows'">🪟</span>
                            <span v-else>🍎</span>
                        </div>
                        <div class="machine-info">
                            <div class="machine-name">{{ machine.name }}</div>
                            <div class="machine-host">{{ machine.username }}@{{ machine.host }}:{{ machine.port }}</div>
                        </div>
                        <div class="card-actions">
                            <button class="action-btn" @click.stop="openEditModal(machine)" title="编辑">✏️</button>
                            <button class="action-btn danger" @click.stop="openDeleteConfirm(machine)"
                                title="删除">🗑</button>
                        </div>
                    </div>

                    <div class="card-tags" v-if="parseTags(machine.tags).length > 0">
                        <span v-for="tag in parseTags(machine.tags)" :key="tag" class="tag">{{ tag }}</span>
                    </div>

                    <div class="card-footer">
                        <button class="btn-test" @click.stop="testConnection(machine)"
                            :disabled="isTesting === machine.id">
                            {{ isTesting === machine.id ? '测试中...' : '测试连接' }}
                        </button>
                        <button class="btn-connect" @click="connectToMachine(machine)"
                            :disabled="isConnecting === machine.id">
                            {{ isConnecting === machine.id ? '连接中...' : '连接' }}
                        </button>
                    </div>
                </div>
            </div>
        </main>

        <!-- 添加机器模态框 -->
        <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
            <div class="modal">
                <h3>添加机器</h3>
                <div class="form-group">
                    <label>名称（可选）</label>
                    <input v-model="newMachine.name" placeholder="例如：生产服务器" />
                </div>
                <div class="form-row">
                    <div class="form-group flex-1">
                        <label>主机地址 *</label>
                        <input v-model="newMachine.host" placeholder="IP 或域名" />
                    </div>
                    <div class="form-group port-field">
                        <label>端口</label>
                        <input v-model.number="newMachine.port" type="number" placeholder="22" />
                    </div>
                </div>
                <div class="form-group">
                    <label>用户名 *</label>
                    <input v-model="newMachine.username" placeholder="root" />
                </div>
                <div class="form-group">
                    <label>认证方式</label>
                    <div class="auth-toggle">
                        <button :class="{ active: newMachine.auth_type === 'password' }"
                            @click="newMachine.auth_type = 'password'">密码</button>
                        <button :class="{ active: newMachine.auth_type === 'key' }"
                            @click="newMachine.auth_type = 'key'">密钥</button>
                    </div>
                </div>
                <div v-if="newMachine.auth_type === 'password'" class="form-group">
                    <label>密码</label>
                    <input v-model="newMachine.password" type="password" placeholder="SSH 密码" />
                </div>
                <div v-else class="form-group">
                    <label>私钥路径</label>
                    <input v-model="newMachine.private_key_path" placeholder="~/.ssh/id_rsa" />
                </div>
                <div class="form-group">
                    <label>操作系统</label>
                    <select v-model="newMachine.os">
                        <option value="linux">Linux</option>
                        <option value="windows">Windows</option>
                        <option value="macos">macOS</option>
                    </select>
                </div>
                <div class="modal-actions">
                    <button class="btn-cancel" @click="showAddModal = false; resetForm()">取消</button>
                    <button class="btn-primary" @click="handleAddMachine"
                        :disabled="!newMachine.host || !newMachine.username">确定</button>
                </div>
            </div>
        </div>

        <!-- 编辑机器模态框 -->
        <div v-if="showEditModal && editingMachine" class="modal-overlay" @click.self="showEditModal = false">
            <div class="modal">
                <h3>编辑机器</h3>
                <div class="form-group">
                    <label>名称</label>
                    <input v-model="editingMachine.name" />
                </div>
                <div class="form-row">
                    <div class="form-group flex-1">
                        <label>主机地址</label>
                        <input v-model="editingMachine.host" />
                    </div>
                    <div class="form-group port-field">
                        <label>端口</label>
                        <input v-model.number="editingMachine.port" type="number" />
                    </div>
                </div>
                <div class="form-group">
                    <label>用户名</label>
                    <input v-model="editingMachine.username" />
                </div>
                <div class="form-group">
                    <label>认证方式</label>
                    <div class="auth-toggle">
                        <button :class="{ active: editingMachine.auth_type === 'password' }"
                            @click="editingMachine.auth_type = 'password'">密码</button>
                        <button :class="{ active: editingMachine.auth_type === 'key' }"
                            @click="editingMachine.auth_type = 'key'">密钥</button>
                    </div>
                </div>
                <div v-if="editingMachine.auth_type === 'password'" class="form-group">
                    <label>密码</label>
                    <input v-model="editingMachine.password" type="password" placeholder="留空保持不变" />
                </div>
                <div v-else class="form-group">
                    <label>私钥路径</label>
                    <input v-model="editingMachine.private_key_path" />
                </div>
                <div class="modal-actions">
                    <button class="btn-cancel" @click="showEditModal = false; editingMachine = null">取消</button>
                    <button class="btn-primary" @click="handleUpdateMachine">保存</button>
                </div>
            </div>
        </div>

        <!-- 删除确认对话框 -->
        <div v-if="showDeleteConfirm && deletingMachine" class="modal-overlay" @click.self="showDeleteConfirm = false">
            <div class="modal modal-sm">
                <h3>确认删除</h3>
                <p class="confirm-text">确定要删除机器 <strong>{{ deletingMachine.name }}</strong> 吗？此操作不可恢复。</p>
                <div class="modal-actions">
                    <button class="btn-cancel" @click="showDeleteConfirm = false; deletingMachine = null">取消</button>
                    <button class="btn-danger" @click="handleDeleteMachine">删除</button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.machine-manager {
    display: flex;
    width: 100vw;
    height: 100vh;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    overflow: hidden;
}

/* Sidebar */
.sidebar {
    width: 220px;
    background: linear-gradient(180deg, rgba(18, 22, 32, 0.95) 0%, rgba(12, 16, 24, 0.98) 100%);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 16px;
}

.brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 1.2rem;
    font-weight: bold;
    margin-bottom: 32px;
    padding: 8px;
}

.brand .logo {
    font-size: 1.5rem;
}

.nav-menu {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
}

.nav-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border-radius: 10px;
    color: var(--text-secondary);
    text-decoration: none;
    transition: all 0.2s ease;
    font-size: 0.95rem;
}

.nav-item:hover {
    background-color: rgba(255, 255, 255, 0.06);
    color: var(--text-primary);
}

.nav-item.active {
    background: linear-gradient(135deg, rgba(99, 102, 241, 0.15) 0%, rgba(139, 92, 246, 0.1) 100%);
    color: var(--accent);
    border-left: 3px solid var(--accent);
}

.sidebar-footer {
    border-top: 1px solid var(--border);
    padding-top: 16px;
}

.local-service {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.85rem;
    color: var(--text-secondary);
}

.toggle-switch {
    width: 40px;
    height: 22px;
    background-color: var(--success);
    border-radius: 11px;
    position: relative;
    cursor: pointer;
}

.toggle-switch::after {
    content: '';
    position: absolute;
    right: 3px;
    top: 3px;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

/* Main Content */
.main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: linear-gradient(135deg, #0f1419 0%, #1a1f2e 50%, #151922 100%);
    overflow: hidden;
}

.top-bar {
    height: 56px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 24px;
    border-bottom: 1px solid var(--border);
    background: rgba(0, 0, 0, 0.2);
}

.breadcrumbs {
    display: flex;
    align-items: center;
    gap: 8px;
}

.crumb.active {
    font-weight: 600;
    font-size: 1.1rem;
}

.machine-count {
    color: var(--text-secondary);
    font-size: 0.9rem;
}

.btn-refresh {
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border);
    color: var(--text-primary);
    padding: 8px 16px;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
}

.btn-refresh:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.12);
}

.btn-refresh:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.toolbar {
    padding: 16px 24px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.left-tools {
    display: flex;
    gap: 16px;
    align-items: center;
}

.btn-primary {
    background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-weight: 500;
    transition: all 0.2s;
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.25);
}

.btn-primary:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(99, 102, 241, 0.35);
}

.btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

.btn-primary .icon {
    font-size: 1.2rem;
}

.search-box {
    display: flex;
    align-items: center;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0 14px;
    height: 42px;
    width: 280px;
    transition: all 0.2s;
}

.search-box:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(125, 211, 252, 0.1);
}

.search-box .icon {
    margin-right: 10px;
    opacity: 0.6;
}

.search-box input {
    background: transparent;
    border: none;
    color: white;
    outline: none;
    width: 100%;
    font-size: 0.9rem;
}

.search-box input::placeholder {
    color: var(--text-muted);
}

/* Error Bar */
.error-bar {
    margin: 0 24px;
    padding: 12px 16px;
    background: rgba(255, 100, 100, 0.15);
    border: 1px solid rgba(255, 100, 100, 0.3);
    border-radius: 8px;
    color: #ff7a7a;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.error-bar button {
    background: none;
    border: none;
    color: inherit;
    cursor: pointer;
    font-size: 1.1rem;
}

/* Loading & Empty States */
.loading-state,
.empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
}

.spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

.empty-icon {
    font-size: 4rem;
    margin-bottom: 16px;
}

.empty-state h3 {
    margin-bottom: 8px;
    color: var(--text-primary);
}

/* Machine Grid */
.machine-grid {
    padding: 0 24px 24px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 20px;
    overflow-y: auto;
    flex: 1;
}

.machine-card {
    background: linear-gradient(135deg, rgba(30, 35, 45, 0.8) 0%, rgba(25, 30, 40, 0.9) 100%);
    border: 1px solid var(--border);
    border-radius: 16px;
    padding: 20px;
    transition: all 0.25s ease;
}

.machine-card:hover {
    border-color: rgba(99, 102, 241, 0.4);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
}

.card-header {
    display: flex;
    gap: 14px;
    margin-bottom: 16px;
}

.os-icon {
    width: 52px;
    height: 52px;
    background: linear-gradient(135deg, #e67e22 0%, #d35400 100%);
    border-radius: 14px;
    display: grid;
    place-items: center;
    font-size: 1.6rem;
    box-shadow: 0 4px 12px rgba(230, 126, 34, 0.25);
}

.os-icon.windows {
    background: linear-gradient(135deg, #0078d4 0%, #0063b1 100%);
    box-shadow: 0 4px 12px rgba(0, 120, 212, 0.25);
}

.os-icon.macos {
    background: linear-gradient(135deg, #555555 0%, #333333 100%);
    box-shadow: 0 4px 12px rgba(85, 85, 85, 0.25);
}

.machine-info {
    flex: 1;
    min-width: 0;
}

.machine-name {
    font-weight: 600;
    font-size: 1.05rem;
    margin-bottom: 6px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.machine-host {
    font-size: 0.85rem;
    color: var(--text-secondary);
    font-family: 'SF Mono', 'Monaco', 'Inconsolata', monospace;
}

.card-actions {
    display: flex;
    gap: 6px;
}

.action-btn {
    width: 32px;
    height: 32px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    display: grid;
    place-items: center;
    transition: all 0.2s;
    font-size: 0.9rem;
}

.action-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    border-color: var(--border);
}

.action-btn.danger:hover {
    background: rgba(255, 100, 100, 0.15);
    border-color: rgba(255, 100, 100, 0.3);
}

.card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
}

.tag {
    background: rgba(99, 102, 241, 0.15);
    color: #a5b4fc;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 0.75rem;
}

.card-footer {
    display: flex;
    gap: 10px;
    padding-top: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.btn-test,
.btn-connect {
    flex: 1;
    padding: 10px;
    border-radius: 10px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
}

.btn-test {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid var(--border);
    color: var(--text-primary);
}

.btn-test:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.1);
}

.btn-connect {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    border: none;
    color: white;
    box-shadow: 0 4px 12px rgba(16, 185, 129, 0.25);
}

.btn-connect:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 6px 16px rgba(16, 185, 129, 0.35);
}

.btn-test:disabled,
.btn-connect:disabled {
    opacity: 0.6;
    cursor: not-allowed;
}

/* Modal */
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
    display: grid;
    place-items: center;
    z-index: 100;
}

.modal {
    background: linear-gradient(135deg, #1e2330 0%, #171c28 100%);
    padding: 28px;
    border-radius: 20px;
    width: 440px;
    max-width: 90vw;
    border: 1px solid var(--border);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
}

.modal-sm {
    width: 380px;
}

.modal h3 {
    margin-bottom: 24px;
    font-size: 1.25rem;
}

.form-group {
    margin-bottom: 18px;
}

.form-group label {
    display: block;
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-bottom: 8px;
}

.form-group input,
.form-group select {
    width: 100%;
    padding: 12px 14px;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--border);
    color: white;
    border-radius: 10px;
    font-size: 0.95rem;
    transition: all 0.2s;
}

.form-group input:focus,
.form-group select:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 3px rgba(125, 211, 252, 0.1);
}

.form-group select {
    cursor: pointer;
}

.form-row {
    display: flex;
    gap: 12px;
}

.flex-1 {
    flex: 1;
}

.port-field {
    width: 100px;
}

.auth-toggle {
    display: flex;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 10px;
    padding: 4px;
}

.auth-toggle button {
    flex: 1;
    padding: 10px;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s;
}

.auth-toggle button.active {
    background: var(--accent);
    color: #0f141f;
    font-weight: 500;
}

.confirm-text {
    color: var(--text-secondary);
    line-height: 1.6;
    margin-bottom: 24px;
}

.confirm-text strong {
    color: var(--text-primary);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 24px;
}

.btn-cancel {
    padding: 10px 20px;
    border-radius: 10px;
    cursor: pointer;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-primary);
    transition: all 0.2s;
}

.btn-cancel:hover {
    background: rgba(255, 255, 255, 0.06);
}

.btn-danger {
    padding: 10px 20px;
    border-radius: 10px;
    cursor: pointer;
    border: none;
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    color: white;
    font-weight: 500;
    transition: all 0.2s;
}

.btn-danger:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.modal-actions .btn-primary {
    padding: 10px 24px;
}
</style>
