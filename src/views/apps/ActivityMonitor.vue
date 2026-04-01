<!--
  ActivityMonitor.vue - 活动监视器应用
  系统资源监控面板，支持 CPU、内存、磁盘、网络和进程监控
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import { api } from "@/lib/api";
import CpuChart from "./monitor/CpuChart.vue";
import MemoryChart from "./monitor/MemoryChart.vue";
import DiskChart, { type DiskInfo } from "./monitor/DiskChart.vue";
import NetworkChart from "./monitor/NetworkChart.vue";
import ProcessList, { type ProcessInfo } from "./monitor/ProcessList.vue";
import ProcessDetail from "./monitor/ProcessDetail.vue";
import SystemOverview from "./monitor/SystemOverview.vue";
import { AlertCircle, RefreshCw } from "lucide-vue-next";

const props = defineProps<{
    sessionId: string;
}>();

type TabId = "overview" | "cpu" | "memory" | "disk" | "network" | "processes";
type ConnectionStatus = "connecting" | "connected" | "error" | "idle";

const activeTab = ref<TabId>("overview");

// 连接状态
const connectionStatus = ref<ConnectionStatus>("idle");
const errorMessage = ref<string>("");

// 进程详情抽屉状态
const selectedProcess = ref<ProcessInfo | null>(null);
const detailVisible = ref(false);

const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: "overview", label: "概览", icon: "icon-[mdi--view-dashboard]" },
    { id: "cpu", label: "CPU", icon: "icon-[mdi--chip]" },
    { id: "memory", label: "内存", icon: "icon-[mdi--memory]" },
    { id: "disk", label: "磁盘", icon: "icon-[mdi--harddisk]" },
    { id: "network", label: "网络", icon: "icon-[mdi--access-point-network]" },
    { id: "processes", label: "进程", icon: "icon-[mdi--format-list-bulleted]" },
];

// 接口定义 (与 Rust 后端一致)
interface MemoryInfo {
    total: number;
    used: number;
    free: number;
    cached: number;
}

interface NetworkInfo {
    rxBytes: number;
    txBytes: number;
}

interface SystemInfo {
    hostname: string;
    uptime: string;
    loadAverage: [number, number, number];
    cpuCores: number;
    kernelVersion: string;
    totalMemory: number;
}

interface SystemStats {
    cpuPercent: number;
    memory: MemoryInfo;
    disks: DiskInfo[];
    network: NetworkInfo;
    processes: ProcessInfo[];
    system: SystemInfo;
}

// 状态数据
const cpuHistory = ref<number[]>(Array(60).fill(0));
const networkRxHistory = ref<number[]>(Array(60).fill(0));
const networkTxHistory = ref<number[]>(Array(60).fill(0));
const memoryData = ref<MemoryInfo>({ total: 0, used: 0, free: 0, cached: 0 });
const diskData = ref<DiskInfo[]>([]);
const processes = ref<ProcessInfo[]>([]);
const systemInfo = ref<SystemInfo>({
    hostname: "Unknown",
    uptime: "---",
    loadAverage: [0, 0, 0],
    cpuCores: 1,
    kernelVersion: "---",
    totalMemory: 0,
});

let refreshInterval: number | null = null;
let lastNetworkStats: NetworkInfo | null = null;

async function refreshData() {
    if (!props.sessionId || props.sessionId === "default-session") {
        connectionStatus.value = "error";
        errorMessage.value = "未连接到 SSH 会话";
        return;
    }

    connectionStatus.value = "connecting";
    
    try {
        const stats = await api.getSystemStats(props.sessionId);

        // 更新 CPU
        cpuHistory.value = [...cpuHistory.value.slice(1), stats.cpuPercent];

        // 更新网络 (计算速率)
        if (lastNetworkStats) {
            const rxDiff = Math.max(0, stats.network.rxBytes - lastNetworkStats.rxBytes);
            const txDiff = Math.max(0, stats.network.txBytes - lastNetworkStats.txBytes);

            networkRxHistory.value = [...networkRxHistory.value.slice(1), rxDiff];
            networkTxHistory.value = [...networkTxHistory.value.slice(1), txDiff];
        }
        lastNetworkStats = stats.network;

        // 更新其他数据
        memoryData.value = stats.memory;
        diskData.value = stats.disks;
        processes.value = stats.processes;
        systemInfo.value = stats.system;
        
        connectionStatus.value = "connected";
        errorMessage.value = "";
    } catch (e: any) {
        console.error("Failed to fetch system stats:", e);
        connectionStatus.value = "error";
        errorMessage.value = e?.toString?.() || "获取系统数据失败";
    }
}

function handleViewDetail(process: ProcessInfo) {
    selectedProcess.value = process;
    detailVisible.value = true;
}

function handleCloseDetail() {
    detailVisible.value = false;
    selectedProcess.value = null;
}

function handleProcessKilled(pid: number) {
    processes.value = processes.value.filter(p => p.pid !== pid);
}

// 监听 sessionId 变化
watch(() => props.sessionId, (newSessionId) => {
    if (newSessionId && newSessionId !== "default-session") {
        refreshData();
    }
}, { immediate: true });

onMounted(() => {
    refreshData();
    refreshInterval = window.setInterval(refreshData, 2000);
});

onUnmounted(() => {
    if (refreshInterval) {
        clearInterval(refreshInterval);
    }
});
</script>

<template>
    <div class="flex-1 flex h-full min-h-0 bg-background">
        <!-- 连接状态提示 -->
        <div v-if="connectionStatus === 'error'"
            class="absolute top-3 left-1/2 -translate-x-1/2 flex items-center gap-2.5 px-4 py-2.5 bg-destructive/90 rounded-lg text-white text-sm z-50 shadow-lg">
            <AlertCircle class="w-4.5 h-4.5" />
            <span>{{ errorMessage }}</span>
            <button
                class="flex items-center gap-1 px-2.5 py-1 bg-white/20 hover:bg-white/30 rounded text-xs transition-colors"
                @click="refreshData">
                <RefreshCw class="w-3.5 h-3.5" />
                重试
            </button>
        </div>

        <!-- 侧边栏 -->
        <aside class="w-[140px] flex flex-col p-2 bg-muted/30 border-r border-border overflow-y-auto">
            <div v-for="tab in tabs" :key="tab.id"
                class="flex items-center gap-2.5 px-3 py-2.5 rounded-lg cursor-pointer text-sm transition-colors"
                :class="activeTab === tab.id
                    ? 'bg-primary/10 text-primary font-medium'
                    : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'"
                @click="activeTab = tab.id">
                <span :class="tab.icon" class="text-lg"></span>
                <span>{{ tab.label }}</span>
            </div>
        </aside>

        <!-- 内容区 -->
        <main class="flex-1 p-4 overflow-y-auto">
            <!-- 概览页 -->
            <div v-if="activeTab === 'overview'" class="flex flex-col gap-4">
                <SystemOverview :hostname="systemInfo.hostname" :uptime="systemInfo.uptime"
                    :load-average="systemInfo.loadAverage" :cpu-cores="systemInfo.cpuCores"
                    :kernel-version="systemInfo.kernelVersion" :total-memory="systemInfo.totalMemory" />
                <div class="grid grid-cols-2 gap-4">
                    <CpuChart :cpu-data="cpuHistory" :core-count="systemInfo.cpuCores" />
                    <MemoryChart :total="memoryData.total" :used="memoryData.used" :free="memoryData.free"
                        :cached="memoryData.cached" />
                </div>
                <div class="grid grid-cols-2 gap-4">
                    <NetworkChart :rx-data="networkRxHistory" :tx-data="networkTxHistory" />
                    <DiskChart :disks="diskData" />
                </div>
            </div>

            <!-- CPU 页 -->
            <div v-else-if="activeTab === 'cpu'" class="h-full">
                <CpuChart :cpu-data="cpuHistory" :core-count="systemInfo.cpuCores" />
            </div>

            <!-- 内存页 -->
            <div v-else-if="activeTab === 'memory'" class="h-full">
                <MemoryChart :total="memoryData.total" :used="memoryData.used" :free="memoryData.free"
                    :cached="memoryData.cached" />
            </div>

            <!-- 磁盘页 -->
            <div v-else-if="activeTab === 'disk'" class="h-full">
                <DiskChart :disks="diskData" />
            </div>

            <!-- 网络页 -->
            <div v-else-if="activeTab === 'network'" class="h-full">
                <NetworkChart :rx-data="networkRxHistory" :tx-data="networkTxHistory" />
            </div>

            <!-- 进程页 -->
            <div v-else-if="activeTab === 'processes'" class="h-full">
                <ProcessList :processes="processes" :session-id="sessionId" @view-detail="handleViewDetail" />
            </div>
        </main>

        <!-- 进程详情抽屉 -->
        <ProcessDetail :process="selectedProcess" :session-id="sessionId" :visible="detailVisible"
            @close="handleCloseDetail" @killed="handleProcessKilled" />
    </div>
</template>

