<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ProcessInfo {
    pid: number;
    name: string;
    cpu: number;
    memory: number;
    user: string;
    status: string;
    statusDesc: string;
    startTime: string;
    elapsedTime: string;
    rss: number;
    vsz: number;
    command: string;
}

const props = defineProps<{
    processes: ProcessInfo[];
    sessionId: string;
}>();

const emit = defineEmits<{
    viewDetail: [process: ProcessInfo];
}>();

const searchQuery = ref("");
const sortKey = ref<keyof ProcessInfo>("cpu");
const sortAsc = ref(false);
const killingPid = ref<number | null>(null);

// 列配置
const columns = [
    { key: "pid", label: "PID", width: "80px", sortable: true },
    { key: "name", label: "进程名", width: "2fr", sortable: true },
    { key: "user", label: "用户", width: "90px", sortable: true },
    { key: "statusDesc", label: "状态", width: "80px", sortable: true },
    { key: "cpu", label: "CPU %", width: "70px", sortable: true },
    { key: "memory", label: "内存 %", width: "70px", sortable: true },
    { key: "rss", label: "物理内存", width: "100px", sortable: true },
    { key: "elapsedTime", label: "CPU时间", width: "90px", sortable: true },
];

// 格式化字节
function formatBytes(kb: number): string {
    if (kb === 0) return "0 B";
    const bytes = kb * 1024;
    const sizes = ["KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    if (i === 0) return `${kb} KB`;
    return parseFloat((bytes / Math.pow(1024, i)).toFixed(1)) + " " + sizes[i - 1];
}

// 获取状态样式
function getStatusClass(status: string): string {
    const firstChar = status.charAt(0);
    switch (firstChar) {
        case 'R': return 'status-running';
        case 'S': return 'status-sleeping';
        case 'Z': return 'status-zombie';
        case 'D': return 'status-disk';
        case 'T': return 'status-stopped';
        default: return 'status-default';
    }
}

const filteredProcesses = computed(() => {
    let list = [...props.processes];

    // 搜索过滤
    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        list = list.filter(p =>
            p.name.toLowerCase().includes(query) ||
            p.user.toLowerCase().includes(query) ||
            String(p.pid).includes(query) ||
            p.command.toLowerCase().includes(query)
        );
    }

    // 排序
    list.sort((a, b) => {
        let cmp = 0;
        const aVal = a[sortKey.value];
        const bVal = b[sortKey.value];
        
        if (typeof aVal === 'number' && typeof bVal === 'number') {
            cmp = aVal - bVal;
        } else if (typeof aVal === 'string' && typeof bVal === 'string') {
            cmp = aVal.localeCompare(bVal);
        }
        return sortAsc.value ? cmp : -cmp;
    });

    return list;
});

function toggleSort(key: keyof ProcessInfo) {
    if (sortKey.value === key) {
        sortAsc.value = !sortAsc.value;
    } else {
        sortKey.value = key;
        sortAsc.value = false;
    }
}

function getSortIcon(key: keyof ProcessInfo) {
    if (sortKey.value !== key) return "";
    return sortAsc.value ? "↑" : "↓";
}

async function handleKillProcess(pid: number, event: Event) {
    event.stopPropagation();
    if (!confirm(`确定要终止进程 ${pid} 吗？`)) return;
    
    killingPid.value = pid;
    try {
        await invoke("kill_process", { sessionId: props.sessionId, pid, signal: 15 });
    } catch (e) {
        console.error("Failed to kill process:", e);
        alert(`终止进程失败: ${e}`);
    } finally {
        killingPid.value = null;
    }
}

async function handleForceKill(pid: number, event: Event) {
    event.stopPropagation();
    if (!confirm(`确定要强制终止进程 ${pid} 吗？(SIGKILL)`)) return;
    
    killingPid.value = pid;
    try {
        await invoke("kill_process", { sessionId: props.sessionId, pid, signal: 9 });
    } catch (e) {
        console.error("Failed to force kill process:", e);
        alert(`强制终止进程失败: ${e}`);
    } finally {
        killingPid.value = null;
    }
}

function handleRowClick(process: ProcessInfo) {
    emit('viewDetail', process);
}
</script>

<template>
    <div class="process-list">
        <div class="list-toolbar">
            <div class="search-box">
                <span class="icon-[mdi--magnify] search-icon"></span>
                <input v-model="searchQuery" type="text" placeholder="搜索进程 (名称/PID/用户/命令)..." class="search-input" />
            </div>
            <div class="process-count">共 {{ filteredProcesses.length }} 个进程</div>
        </div>

        <div class="list-header">
            <span 
                v-for="col in columns" 
                :key="col.key"
                class="col" 
                :class="{ sortable: col.sortable }"
                :style="{ width: col.width, flex: col.width.includes('fr') ? col.width : 'none' }"
                @click="col.sortable && toggleSort(col.key as keyof ProcessInfo)"
            >
                {{ col.label }} {{ col.sortable ? getSortIcon(col.key as keyof ProcessInfo) : '' }}
            </span>
            <span class="col col-action"></span>
        </div>

        <div class="list-body">
            <div 
                v-for="proc in filteredProcesses" 
                :key="proc.pid" 
                class="list-row"
                @click="handleRowClick(proc)"
            >
                <span class="col col-pid">{{ proc.pid }}</span>
                <span class="col col-name" :title="proc.command">
                    <span class="process-icon">{{ proc.name.charAt(0).toUpperCase() }}</span>
                    {{ proc.name }}
                </span>
                <span class="col col-user">{{ proc.user }}</span>
                <span class="col col-status">
                    <span class="status-badge" :class="getStatusClass(proc.status)">
                        {{ proc.statusDesc }}
                    </span>
                </span>
                <span class="col col-cpu" :class="{ high: proc.cpu > 50 }">{{ proc.cpu.toFixed(1) }}%</span>
                <span class="col col-mem" :class="{ high: proc.memory > 50 }">{{ proc.memory.toFixed(1) }}%</span>
                <span class="col col-rss">{{ formatBytes(proc.rss) }}</span>
                <span class="col col-elapsed">{{ proc.elapsedTime }}</span>
                <span class="col col-action">
                    <button 
                        class="kill-btn" 
                        title="终止进程" 
                        :disabled="killingPid === proc.pid"
                        @click="handleKillProcess(proc.pid, $event)"
                    >
                        <span class="icon-[mdi--close]"></span>
                    </button>
                    <button 
                        class="force-kill-btn" 
                        title="强制终止" 
                        :disabled="killingPid === proc.pid"
                        @click="handleForceKill(proc.pid, $event)"
                    >
                        <span class="icon-[mdi--lightning-bolt]"></span>
                    </button>
                </span>
            </div>
            <div v-if="filteredProcesses.length === 0" class="empty-state">
                <span class="icon-[mdi--magnify]"></span>
                <p>未找到匹配的进程</p>
            </div>
        </div>
    </div>
</template>

<style scoped>
.process-list {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 16px;
    overflow: hidden;
}

.list-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.search-box {
    position: relative;
    display: flex;
    align-items: center;
}

.search-icon {
    position: absolute;
    left: 10px;
    color: rgba(255, 255, 255, 0.4);
    font-size: 16px;
}

.search-input {
    width: 280px;
    padding: 8px 10px 8px 32px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    color: #fff;
    font-size: 0.8rem;
    outline: none;
    transition: border-color 0.2s;
}

.search-input::placeholder {
    color: rgba(255, 255, 255, 0.3);
}

.search-input:focus {
    border-color: rgba(125, 211, 252, 0.5);
}

.process-count {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
}

.list-header {
    display: flex;
    gap: 8px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.03);
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.list-header .col {
    flex-shrink: 0;
}

.sortable {
    cursor: pointer;
    user-select: none;
    transition: color 0.2s;
}

.sortable:hover {
    color: rgba(255, 255, 255, 0.8);
}

.list-body {
    flex: 1;
    overflow-y: auto;
    overflow-x: auto;
}

.list-row {
    display: flex;
    gap: 8px;
    padding: 10px 16px;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.8);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
    transition: background 0.2s;
    cursor: pointer;
    align-items: center;
}

.list-row:hover {
    background: rgba(255, 255, 255, 0.04);
}

.col {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.col-pid {
    color: rgba(255, 255, 255, 0.5);
    font-family: monospace;
    font-size: 0.75rem;
}

.col-name {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
}

.process-icon {
    width: 22px;
    height: 22px;
    background: linear-gradient(135deg, rgba(125, 211, 252, 0.3), rgba(125, 211, 252, 0.1));
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.7rem;
    font-weight: 600;
    color: #7dd3fc;
}

.col-user {
    color: rgba(255, 255, 255, 0.5);
}

.status-badge {
    display: inline-block;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 0.65rem;
    font-weight: 500;
}

.status-running {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
}

.status-sleeping {
    background: rgba(125, 211, 252, 0.15);
    color: #7dd3fc;
}

.status-zombie {
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
}

.status-disk {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
}

.status-stopped {
    background: rgba(156, 163, 175, 0.15);
    color: #9ca3af;
}

.status-default {
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.6);
}

.col-cpu,
.col-mem,
.col-rss,
.col-threads,
.col-nice,
.col-elapsed {
    font-family: monospace;
    text-align: right;
}

.col-cpu.high,
.col-mem.high {
    color: #f59e0b;
    font-weight: 500;
}

.col-action {
    display: flex;
    gap: 4px;
    justify-content: flex-end;
}

.kill-btn,
.force-kill-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background 0.2s;
}

.kill-btn {
    background: rgba(255, 107, 107, 0.1);
    color: #ff6b6b;
}

.force-kill-btn {
    background: rgba(245, 158, 11, 0.1);
    color: #fbbf24;
}

.list-row:hover .kill-btn,
.list-row:hover .force-kill-btn {
    opacity: 1;
}

.kill-btn:hover:not(:disabled) {
    background: rgba(255, 107, 107, 0.25);
}

.force-kill-btn:hover:not(:disabled) {
    background: rgba(245, 158, 11, 0.25);
}

.kill-btn:disabled,
.force-kill-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: rgba(255, 255, 255, 0.3);
    gap: 12px;
}

.empty-state span {
    font-size: 32px;
}

.empty-state p {
    font-size: 0.85rem;
}
</style>
