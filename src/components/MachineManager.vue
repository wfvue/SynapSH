<script setup lang="ts">
import { ref } from "vue";
// import { invoke } from "@tauri-apps/api/core";

const emit = defineEmits<{
    connect: [sessionId: string];
}>();

interface Machine {
    id: string;
    name: string;
    host: string;
    username: string;
    tags: string[];
    os: "linux" | "windows" | "macos";
    status: "online" | "offline";
    cpu?: string;
    memory?: string;
    storage?: string;
}

const machines = ref<Machine[]>([
    {
        id: "1",
        name: "kr",
        host: "52.141.0.221",
        username: "root",
        tags: ["生产环境", "Web"],
        os: "linux",
        status: "offline",
    },
    {
        id: "2",
        name: "us",
        host: "142.171.222.118",
        username: "admin",
        tags: ["测试环境", "DB"],
        os: "linux",
        status: "online",
        cpu: "0.7%",
        memory: "0.2/1.9G",
        storage: "5.4/98.3G",
    },
]);

const showAddModal = ref(false);
const newMachine = ref({
    name: "",
    host: "",
    port: 22,
    username: "",
    password: "",
    authType: "password" as "password" | "key",
    privateKey: "",
});

// const isConnecting = ref(false);

async function connectToMachine(machine: Machine) {
    // In a real app, we might prompt for password if not saved, or use saved key
    // For now, we simulate a connection or use the "add" flow logic for new connections
    console.log("Connecting to", machine.host);

    // Simulation of connection process
    // In reality, this would likely re-use the ConnectionPanel logic or invoke the backend directly
    // For this prototype, we'll just emit a dummy session ID if it's "online" or try to connect

    if (machine.status === 'online') {
        emit("connect", `session_${machine.id}_${Date.now()}`);
    } else {
        // Trigger connection flow (simplified)
        emit("connect", `session_${machine.id}_${Date.now()}`);
    }
}

async function handleAddMachine() {
    // Add machine logic here
    console.log("Adding machine", newMachine.value);
    machines.value.push({
        id: Date.now().toString(),
        name: newMachine.value.name || newMachine.value.host,
        host: newMachine.value.host,
        username: newMachine.value.username,
        tags: [],
        os: "linux",
        status: "offline"
    });
    showAddModal.value = false;
}

</script>

<template>
    <div class="machine-manager">
        <aside class="sidebar">
            <div class="brand">
                <div class="logo"></div>
                <span>GMSSON</span>
            </div>
            <nav class="nav-menu">
                <a href="#" class="nav-item active">
                    <i class="icon-server"></i>
                    机器管理
                </a>
                <a href="#" class="nav-item">
                    <i class="icon-terminal"></i>
                    命令中心
                </a>
                <a href="#" class="nav-item">
                    <i class="icon-batch"></i>
                    批处理任务
                </a>
                <a href="#" class="nav-item">
                    <i class="icon-settings"></i>
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
                    <span class="crumb active">全部分组</span>
                </div>
                <div class="top-actions">
                    <button class="icon-btn edit-mode"><i class="icon-edit"></i></button>
                </div>
            </header>

            <div class="toolbar">
                <div class="left-tools">
                    <button class="btn-primary" @click="showAddModal = true">
                        <i class="icon-plus"></i>
                    </button>
                    <div class="tool-group">
                        <button class="btn-tool"><i class="icon-folder"></i></button>
                        <button class="btn-tool"><i class="icon-import"></i></button>
                        <button class="btn-tool"><i class="icon-export"></i></button>
                    </div>
                    <button class="btn-tool"><i class="icon-refresh"></i></button>

                    <div class="search-box">
                        <i class="icon-search"></i>
                        <input type="text" placeholder="搜索机器名称、IP或备注" />
                    </div>
                </div>

                <div class="right-tools">
                    <span class="machine-count">{{ machines.length }}/2</span>
                    <button class="btn-tool"><i class="icon-refresh-sm"></i></button>
                    <div class="view-toggle">
                        <button class="active"><i class="icon-grid"></i></button>
                        <button><i class="icon-list"></i></button>
                    </div>
                </div>
            </div>

            <div class="machine-grid">
                <div v-for="machine in machines" :key="machine.id" class="machine-card"
                    @click="connectToMachine(machine)">
                    <div class="card-header">
                        <div class="os-icon" :class="machine.os">
                            <i class="icon-linux" v-if="machine.os === 'linux'"></i>
                        </div>
                        <div class="machine-info">
                            <div class="machine-name">{{ machine.name }}</div>
                            <div class="machine-host">{{ machine.host }} <i class="icon-copy"></i></div>
                        </div>
                        <div class="status-indicator" :class="machine.status"></div>
                    </div>

                    <div class="card-stats">
                        <div class="stat-item">
                            <label>CPU</label>
                            <div class="progress-bar">
                                <div class="progress" :style="{ width: machine.cpu?.replace('%', '') + '%' || '0%' }">
                                </div>
                            </div>
                            <span class="value">{{ machine.cpu || '--' }}</span>
                        </div>
                        <div class="stat-item">
                            <label>内存</label>
                            <div class="progress-bar">
                                <!-- parser logic needed for real width, simple mock here -->
                                <div class="progress" style="width: 10%"></div>
                            </div>
                            <span class="value">{{ machine.memory || '--' }}</span>
                        </div>
                        <div class="stat-item">
                            <label>存储</label>
                            <div class="progress-bar">
                                <div class="progress" style="width: 5%"></div>
                            </div>
                            <span class="value">{{ machine.storage || '--' }}</span>
                        </div>
                    </div>
                </div>
            </div>
        </main>

        <!-- Add Machine Modal (Simplified) -->
        <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
            <div class="modal">
                <h3>添加机器</h3>
                <div class="form-row">
                    <input v-model="newMachine.host" placeholder="主机地址 (IP)" />
                </div>
                <div class="form-row">
                    <input v-model="newMachine.username" placeholder="用户名" />
                </div>
                <div class="modal-actions">
                    <button @click="showAddModal = false">取消</button>
                    <button class="btn-primary" @click="handleAddMachine">确定</button>
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
    width: 240px;
    background-color: rgba(18, 22, 32, 0.5);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 16px;
}

.brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 1.1rem;
    font-weight: bold;
    margin-bottom: 30px;
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
    gap: 10px;
    padding: 10px 12px;
    border-radius: 8px;
    color: var(--text-secondary);
    text-decoration: none;
    transition: all 0.2s;
}

.nav-item:hover,
.nav-item.active {
    background-color: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
}

.nav-item.active {
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
    font-size: 0.9rem;
}

.toggle-switch {
    width: 36px;
    height: 20px;
    background-color: var(--success);
    border-radius: 10px;
    position: relative;
    cursor: pointer;
}

.toggle-switch::after {
    content: '';
    position: absolute;
    right: 2px;
    top: 2px;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
}

/* Main Content */
.main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #1a1b1e;
    /* Darker bg for content area */
}

.top-bar {
    height: 50px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
}

.crumb.active {
    font-weight: 500;
}

.toolbar {
    padding: 16px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.left-tools,
.right-tools {
    display: flex;
    gap: 12px;
    align-items: center;
}

.tool-group {
    display: flex;
    gap: 1px;
    background: var(--border);
    border-radius: 6px;
    overflow: hidden;
}

.btn-primary {
    background-color: #4f46e5;
    color: white;
    border: none;
    width: 36px;
    height: 36px;
    border-radius: 6px;
    display: grid;
    place-items: center;
    cursor: pointer;
}

.btn-tool {
    background-color: rgba(255, 255, 255, 0.05);
    border: none;
    width: 36px;
    height: 36px;
    color: var(--text-secondary);
    display: grid;
    place-items: center;
    cursor: pointer;
    border-radius: 6px;
}

.search-box {
    display: flex;
    align-items: center;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0 10px;
    height: 36px;
    width: 240px;
}

.search-box input {
    background: transparent;
    border: none;
    color: white;
    margin-left: 8px;
    outline: none;
    width: 100%;
}

.view-toggle {
    display: flex;
    background: rgba(0, 0, 0, 0.2);
    border-radius: 6px;
    padding: 2px;
}

.view-toggle button {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    width: 32px;
    height: 32px;
    border-radius: 4px;
    cursor: pointer;
}

.view-toggle button.active {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
}

/* Grid */
.machine-grid {
    padding: 0 20px 20px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
    overflow-y: auto;
}

.machine-card {
    background-color: rgba(30, 30, 35, 0.6);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 16px;
    cursor: pointer;
    transition: transform 0.2s, background-color 0.2s;
}

.machine-card:hover {
    background-color: rgba(40, 40, 45, 0.8);
    transform: translateY(-2px);
}

.card-header {
    display: flex;
    gap: 12px;
    margin-bottom: 20px;
    position: relative;
}

.os-icon {
    width: 48px;
    height: 48px;
    background: #e67e22;
    /* Mock orange for Ubuntu/Linux */
    border-radius: 10px;
    display: grid;
    place-items: center;
    color: white;
    font-weight: bold;
}

.machine-info {
    flex: 1;
}

.machine-name {
    font-weight: 600;
    margin-bottom: 4px;
}

.machine-host {
    font-size: 0.85rem;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    gap: 6px;
}

.status-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--text-muted);
}

.status-indicator.online {
    background-color: var(--success);
    box-shadow: 0 0 8px rgba(93, 228, 199, 0.4);
}

.card-stats {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    padding-top: 16px;
}

.stat-item label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-bottom: 6px;
}

.progress-bar {
    height: 4px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    margin-bottom: 6px;
    overflow: hidden;
}

.progress-bar .progress {
    height: 100%;
    background-color: var(--text-muted);
    border-radius: 2px;
}

.card-stats .value {
    font-size: 0.75rem;
    color: var(--text-secondary);
}

/* Modal */
.modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    z-index: 100;
}

.modal {
    background: #1e1e1e;
    padding: 24px;
    border-radius: 12px;
    width: 400px;
    border: 1px solid var(--border);
}

.modal h3 {
    margin-bottom: 20px;
}

.form-row {
    margin-bottom: 16px;
}

.form-row input {
    width: 100%;
    padding: 10px;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--border);
    color: white;
    border-radius: 6px;
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
}

.modal-actions button {
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    border: 1px solid var(--border);
    background: transparent;
    color: white;
}

.modal-actions button.btn-primary {
    background: #4f46e5;
    border-color: #4f46e5;
}

/* Icons (Mock) */
.icon-plus::before {
    content: '+';
}

.icon-server::before {
    content: '🖥';
}

.icon-terminal::before {
    content: '>_';
}

.icon-search::before {
    content: '🔍';
}

.icon-linux::before {
    content: '🐧';
}
</style>
