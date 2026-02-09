<!--
  ProcessList.vue - 进程列表组件
  展示系统进程列表，支持搜索、排序和操作
-->
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
        case 'R': return 'bg-emerald-500/15 text-emerald-400';
        case 'S': return 'bg-sky-400/15 text-sky-400';
        case 'Z': return 'bg-amber-500/15 text-amber-400';
        case 'D': return 'bg-red-500/15 text-red-400';
        case 'T': return 'bg-gray-400/15 text-gray-400';
        default: return 'bg-white/10 text-foreground/60';
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
    <div class="h-full flex flex-col bg-card/50 rounded-2xl border border-border/50 overflow-hidden">
        <div class="flex justify-between items-center px-4 py-3 border-b border-border/50">
            <div class="relative flex items-center">
                <span class="icon-[mdi--magnify] absolute left-2.5 text-muted-foreground text-base"></span>
                <input v-model="searchQuery" type="text" placeholder="搜索进程 (名称/PID/用户/命令)..."
                    class="w-[280px] py-1.5 pl-8 pr-2.5 bg-muted/50 border border-border/50 rounded-lg text-sm text-foreground placeholder:text-muted-foreground/50 outline-none focus:border-primary/50 transition-colors" />
            </div>
            <div class="text-xs text-muted-foreground">共 {{ filteredProcesses.length }} 个进程</div>
        </div>

        <div class="flex gap-2 px-4 py-2.5 bg-muted/30 text-[0.7rem] text-muted-foreground uppercase tracking-wider border-b border-border/30">
            <span v-for="col in columns" :key="col.key" class="shrink-0 cursor-pointer select-none hover:text-foreground/80 transition-colors"
                :class="col.sortable ? 'hover:text-foreground' : ''"
                :style="{ width: col.width, flex: col.width.includes('fr') ? col.width : 'none' }"
                @click="col.sortable && toggleSort(col.key as keyof ProcessInfo)">
                {{ col.label }} {{ col.sortable ? getSortIcon(col.key as keyof ProcessInfo) : '' }}
            </span>
            <span class="col-action"></span>
        </div>

        <div class="flex-1 overflow-auto">
            <div v-for="proc in filteredProcesses" :key="proc.pid"
                class="flex gap-2 px-4 py-2.5 text-sm text-foreground/80 border-b border-border/20 hover:bg-muted/30 transition-colors cursor-pointer items-center"
                @click="handleRowClick(proc)">
                <span class="text-muted-foreground font-mono text-xs w-[80px] shrink-0">{{ proc.pid }}</span>
                <span class="flex items-center gap-2 font-medium flex-[2] min-w-0 truncate" :title="proc.command">
                    <span
                        class="w-[22px] h-[22px] bg-gradient-to-br from-sky-400/30 to-sky-400/10 rounded-md flex items-center justify-center text-xs font-semibold text-sky-400 shrink-0">{{ proc.name.charAt(0).toUpperCase() }}</span>
                    {{ proc.name }}
                </span>
                <span class="w-[90px] shrink-0 text-muted-foreground truncate">{{ proc.user }}</span>
                <span class="w-[80px] shrink-0">
                    <span class="inline-block px-1.5 py-0.5 rounded text-[0.65rem] font-medium"
                        :class="getStatusClass(proc.status)">
                        {{ proc.statusDesc }}
                    </span>
                </span>
                <span class="w-[70px] shrink-0 font-mono text-right"
                    :class="proc.cpu > 50 ? 'text-amber-500 font-medium' : ''">{{ proc.cpu.toFixed(1) }}%</span>
                <span class="w-[70px] shrink-0 font-mono text-right"
                    :class="proc.memory > 50 ? 'text-amber-500 font-medium' : ''">{{ proc.memory.toFixed(1) }}%</span>
                <span class="w-[100px] shrink-0 font-mono text-right">{{ formatBytes(proc.rss) }}</span>
                <span class="w-[90px] shrink-0 font-mono text-right">{{ proc.elapsedTime }}</span>
                <span class="flex gap-1 justify-end">
                    <button class="flex items-center justify-center w-6 h-6 rounded-md bg-red-500/10 text-red-500 opacity-0 group-hover:opacity-100 hover:bg-red-500/25 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                        title="终止进程" :disabled="killingPid === proc.pid" @click="handleKillProcess(proc.pid, $event)">
                        <span class="icon-[mdi--close] text-sm"></span>
                    </button>
                    <button class="flex items-center justify-center w-6 h-6 rounded-md bg-amber-500/10 text-amber-500 opacity-0 group-hover:opacity-100 hover:bg-amber-500/25 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                        title="强制终止" :disabled="killingPid === proc.pid" @click="handleForceKill(proc.pid, $event)">
                        <span class="icon-[mdi--lightning-bolt] text-sm"></span>
                    </button>
                </span>
            </div>
            <div v-if="filteredProcesses.length === 0" class="flex flex-col items-center justify-center py-10 text-muted-foreground/50 gap-3">
                <span class="icon-[mdi--magnify] text-3xl"></span>
                <p class="text-sm">未找到匹配的进程</p>
            </div>
        </div>
    </div>
</template>
