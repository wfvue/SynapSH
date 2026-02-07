<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import CpuChart from "./monitor/CpuChart.vue";
import MemoryChart from "./monitor/MemoryChart.vue";
import DiskChart, { type DiskInfo } from "./monitor/DiskChart.vue";
import NetworkChart from "./monitor/NetworkChart.vue";
import ProcessList, { type ProcessInfo } from "./monitor/ProcessList.vue";
import SystemOverview from "./monitor/SystemOverview.vue";

const props = defineProps<{
    sessionId: string;
}>();

type TabId = "overview" | "cpu" | "memory" | "disk" | "network" | "processes";

const activeTab = ref<TabId>("overview");

const tabs: { id: TabId; label: string; icon: string }[] = [
    { id: "overview", label: "概览", icon: "icon-[mdi--view-dashboard]" },
    { id: "cpu", label: "CPU", icon: "icon-[mdi--chip]" },
    { id: "memory", label: "内存", icon: "icon-[mdi--memory]" },
    { id: "disk", label: "磁盘", icon: "icon-[mdi--harddisk]" },
    { id: "network", label: "网络", icon: "icon-[mdi--access-point-network]" },
    { id: "processes", label: "进程", icon: "icon-[mdi--format-list-bulleted]" },
];

// 模拟数据 - 后续接入真实后端
const cpuHistory = ref<number[]>(Array(60).fill(0).map(() => Math.random() * 30 + 10));
const memoryData = ref({
    total: 16 * 1024 * 1024 * 1024,
    used: 8.5 * 1024 * 1024 * 1024,
    free: 4 * 1024 * 1024 * 1024,
    cached: 3.5 * 1024 * 1024 * 1024,
});
const diskData = ref<DiskInfo[]>([
    { name: "sda1", total: 500 * 1024 * 1024 * 1024, used: 234 * 1024 * 1024 * 1024, mountPoint: "/" },
    { name: "sda2", total: 1000 * 1024 * 1024 * 1024, used: 456 * 1024 * 1024 * 1024, mountPoint: "/home" },
    { name: "sdb1", total: 2000 * 1024 * 1024 * 1024, used: 1200 * 1024 * 1024 * 1024, mountPoint: "/data" },
]);
const networkRxHistory = ref<number[]>(Array(60).fill(0).map(() => Math.random() * 1024 * 1024));
const networkTxHistory = ref<number[]>(Array(60).fill(0).map(() => Math.random() * 512 * 1024));
const processes = ref<ProcessInfo[]>([
    { pid: 1, name: "systemd", cpu: 0.1, memory: 0.5, user: "root" },
    { pid: 234, name: "sshd", cpu: 0.2, memory: 0.3, user: "root" },
    { pid: 456, name: "nginx", cpu: 2.5, memory: 1.2, user: "www-data" },
    { pid: 789, name: "node", cpu: 15.3, memory: 8.7, user: "deploy" },
    { pid: 1024, name: "postgres", cpu: 5.2, memory: 12.4, user: "postgres" },
    { pid: 1234, name: "redis-server", cpu: 1.1, memory: 2.8, user: "redis" },
    { pid: 2048, name: "python3", cpu: 8.7, memory: 4.2, user: "deploy" },
    { pid: 3096, name: "docker", cpu: 3.4, memory: 5.6, user: "root" },
]);
const systemInfo = ref({
    hostname: "prod-server-01",
    uptime: "15 days, 7:23:45",
    loadAverage: [0.82, 1.24, 1.56] as [number, number, number],
    cpuCores: 8,
    kernelVersion: "5.15.0-generic",
    totalMemory: 16 * 1024 * 1024 * 1024,
});

let refreshInterval: number | null = null;

function refreshData() {
    // 模拟数据更新
    cpuHistory.value = [...cpuHistory.value.slice(1), Math.random() * 40 + 20];
    networkRxHistory.value = [...networkRxHistory.value.slice(1), Math.random() * 2 * 1024 * 1024];
    networkTxHistory.value = [...networkTxHistory.value.slice(1), Math.random() * 1024 * 1024];

    // 更新内存
    const memChange = (Math.random() - 0.5) * 0.5 * 1024 * 1024 * 1024;
    memoryData.value.used = Math.max(1024 * 1024 * 1024, Math.min(memoryData.value.total * 0.9, memoryData.value.used + memChange));
    memoryData.value.free = memoryData.value.total - memoryData.value.used - memoryData.value.cached;

    // 更新进程 CPU/内存
    processes.value = processes.value.map(p => ({
        ...p,
        cpu: Math.max(0, p.cpu + (Math.random() - 0.5) * 2),
        memory: Math.max(0, p.memory + (Math.random() - 0.5) * 0.5),
    }));
}

function handleKillProcess(pid: number) {
    console.log(`Kill process: ${pid}`);
    processes.value = processes.value.filter(p => p.pid !== pid);
}

onMounted(() => {
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
                    <CpuChart :cpu-data="cpuHistory" />
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
                <CpuChart :cpu-data="cpuHistory" />
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
                <ProcessList :processes="processes" @kill="handleKillProcess" />
            </div>
        </main>
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
