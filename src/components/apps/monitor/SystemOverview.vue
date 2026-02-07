<script setup lang="ts">
const props = defineProps<{
    hostname?: string;
    uptime?: string;
    loadAverage?: [number, number, number];
    cpuCores?: number;
    kernelVersion?: string;
    totalMemory?: number;
}>();

function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}
</script>

<template>
    <div class="system-overview">
        <div class="info-card">
            <span class="icon-[mdi--server] card-icon"></span>
            <div class="card-content">
                <span class="card-label">主机名</span>
                <span class="card-value">{{ hostname || "---" }}</span>
            </div>
        </div>

        <div class="info-card">
            <span class="icon-[mdi--clock-outline] card-icon"></span>
            <div class="card-content">
                <span class="card-label">运行时间</span>
                <span class="card-value">{{ uptime || "---" }}</span>
            </div>
        </div>

        <div class="info-card">
            <span class="icon-[mdi--chip] card-icon"></span>
            <div class="card-content">
                <span class="card-label">CPU 核心</span>
                <span class="card-value">{{ cpuCores ?? "---" }} 核</span>
            </div>
        </div>

        <div class="info-card">
            <span class="icon-[mdi--memory] card-icon"></span>
            <div class="card-content">
                <span class="card-label">总内存</span>
                <span class="card-value">{{ totalMemory ? formatBytes(totalMemory) : "---" }}</span>
            </div>
        </div>

        <div class="info-card">
            <span class="icon-[mdi--gauge] card-icon"></span>
            <div class="card-content">
                <span class="card-label">负载平均</span>
                <span class="card-value" v-if="loadAverage">
                    {{ loadAverage[0].toFixed(2) }} / {{ loadAverage[1].toFixed(2) }} / {{ loadAverage[2].toFixed(2) }}
                </span>
                <span class="card-value" v-else>---</span>
            </div>
        </div>

        <div class="info-card">
            <span class="icon-[mdi--linux] card-icon"></span>
            <div class="card-content">
                <span class="card-label">内核版本</span>
                <span class="card-value">{{ kernelVersion || "---" }}</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.system-overview {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 12px;
}

.info-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.04);
    border-radius: 14px;
    transition: border-color 0.2s, background 0.2s;
}

.info-card:hover {
    background: rgba(255, 255, 255, 0.04);
    border-color: rgba(255, 255, 255, 0.08);
}

.card-icon {
    font-size: 24px;
    color: rgba(125, 211, 252, 0.7);
}

.card-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.card-label {
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.4);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.card-value {
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.9);
    font-weight: 500;
}
</style>
