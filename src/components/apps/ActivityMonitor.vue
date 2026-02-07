<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import CpuChart from "./monitor/CpuChart.vue";
import MemoryChart from "./monitor/MemoryChart.vue";
import DiskChart, { type DiskInfo } from "./monitor/DiskChart.vue";
import NetworkChart from "./monitor/NetworkChart.vue";
import ProcessList, { type ProcessInfo } from "./monitor/ProcessList.vue";
import ProcessDetail from "./monitor/ProcessDetail.vue";
import SystemOverview from "./monitor/SystemOverview.vue";

const props = defineProps<{
    sessionId: string;
}>();

type TabId = "overview" | "cpu" | "memory" | "disk" | "network" | "processes";

const activeTab = ref<TabId>("overview");

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
    hostname: "Loading...",
    uptime: "Loading...",
    loadAverage: [0, 0, 0],
    cpuCores: 0,
    kernelVersion: "Loading...",
    totalMemory: 0,
});

let refreshInterval: number | null = null;
let lastNetworkStats: NetworkInfo | null = null;

async function refreshData() {
    try {
        const stats = await invoke<SystemStats>("get_system_stats", { sessionId: props.sessionId });

        // 更新 CPU
        cpuHistory.value = [...cpuHistory.value.slice(1), stats.cpuPercent];

        // 更新网络 (计算速率)
        if (lastNetworkStats) {
            // 计算每秒字节数 (假设间隔是 2000ms, 但最好根据 timestamp 算)
            // 这里我们只是简单展示两次通过 SSH 获取数据的差值，这代表了这个间隔内的流量
            // 如果间隔是 2s，那么速率其实是 diff/2。但图表展示的是"流量"，
            // 我们可以直接展示差值作为"Recent Traffic"
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
    } catch (e) {
        console.error("Failed to fetch system stats:", e);
        // Error handling: maybe stop refreshing or show error?
        // systemInfo.value.hostname = "Connection Error";
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
    // 从列表中移除已终止的进程
    processes.value = processes.value.filter(p => p.pid !== pid);
}

onMounted(() => {
    refreshData(); // Fetch immediately
    refreshInterval = window.setInterval(refreshData, 2000);
});

onUnmounted(() => {
    if (refreshInterval) {
        clearInterval(refreshInterval);
    }
});
</script>

<template>
    <div class="activity-monitor">
        <!-- 侧边栏 -->
        <aside class="monitor-sidebar">
            <div v-for="tab in tabs" :key="tab.id" class="tab-item" :class="{ active: activeTab === tab.id }"
                @click="activeTab = tab.id">
                <span :class="tab.icon"></span>
                <span class="tab-label">{{ tab.label }}</span>
            </div>
        </aside>

        <!-- 内容区 -->
        <main class="monitor-content">
            <!-- 概览页 -->
            <div v-if="activeTab === 'overview'" class="tab-panel overview-panel">
                <SystemOverview :hostname="systemInfo.hostname" :uptime="systemInfo.uptime"
                    :load-average="systemInfo.loadAverage" :cpu-cores="systemInfo.cpuCores"
                    :kernel-version="systemInfo.kernelVersion" :total-memory="systemInfo.totalMemory" />
                <div class="overview-charts">
                    <CpuChart :cpu-data="cpuHistory" :core-count="systemInfo.cpuCores" />
                    <MemoryChart :total="memoryData.total" :used="memoryData.used" :free="memoryData.free"
                        :cached="memoryData.cached" />
                </div>
                <div class="overview-charts">
                    <NetworkChart :rx-data="networkRxHistory" :tx-data="networkTxHistory" />
                    <DiskChart :disks="diskData" />
                </div>
            </div>

            <!-- CPU 页 -->
            <div v-else-if="activeTab === 'cpu'" class="tab-panel">
                <CpuChart :cpu-data="cpuHistory" :core-count="systemInfo.cpuCores" />
            </div>

            <!-- 内存页 -->
            <div v-else-if="activeTab === 'memory'" class="tab-panel">
                <MemoryChart :total="memoryData.total" :used="memoryData.used" :free="memoryData.free"
                    :cached="memoryData.cached" />
            </div>

            <!-- 磁盘页 -->
            <div v-else-if="activeTab === 'disk'" class="tab-panel">
                <DiskChart :disks="diskData" />
            </div>

            <!-- 网络页 -->
            <div v-else-if="activeTab === 'network'" class="tab-panel">
                <NetworkChart :rx-data="networkRxHistory" :tx-data="networkTxHistory" />
            </div>

            <!-- 进程页 -->
            <div v-else-if="activeTab === 'processes'" class="tab-panel">
                <ProcessList 
                    :processes="processes" 
                    :session-id="sessionId"
                    @view-detail="handleViewDetail" 
                />
            </div>
        </main>

        <!-- 进程详情抽屉 -->
        <ProcessDetail
            :process="selectedProcess"
            :session-id="sessionId"
            :visible="detailVisible"
            @close="handleCloseDetail"
            @killed="handleProcessKilled"
        />
    </div>
</template>

<style scoped>
.activity-monitor {
    height: 100%;
    display: grid;
    grid-template-columns: 140px 1fr;
    background: rgba(14, 18, 28, 0.5);
    border-radius: 0 0 16px 16px;
    overflow: hidden;
}

.monitor-sidebar {
    display: flex;
    flex-direction: column;
    padding: 12px 8px;
    background: rgba(0, 0, 0, 0.2);
    border-right: 1px solid rgba(255, 255, 255, 0.04);
}

.tab-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    border-radius: 10px;
    cursor: pointer;
    color: rgba(255, 255, 255, 0.5);
    transition: background 0.2s, color 0.2s;
}

.tab-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.7);
}

.tab-item.active {
    background: rgba(125, 211, 252, 0.12);
    color: #7dd3fc;
}

.tab-item span:first-child {
    font-size: 18px;
}

.tab-label {
    font-size: 0.8rem;
    font-weight: 500;
}

.monitor-content {
    padding: 16px;
    overflow-y: auto;
}

.tab-panel {
    height: 100%;
}

.overview-panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.overview-charts {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    min-height: 200px;
}
</style>
