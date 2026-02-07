<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import * as echarts from "echarts";

export interface DiskInfo {
    name: string;
    total: number;
    used: number;
    mountPoint: string;
}

const props = defineProps<{
    disks: DiskInfo[];
}>();

const chartRef = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;

function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function initChart() {
    if (!chartRef.value) return;
    chart = echarts.init(chartRef.value, undefined, { renderer: "canvas" });
    updateChart();
}

function updateChart() {
    if (!chart || props.disks.length === 0) return;

    const option: echarts.EChartsOption = {
        grid: {
            top: 20,
            right: 20,
            bottom: 40,
            left: 60,
        },
        xAxis: {
            type: "category",
            data: props.disks.map(d => d.mountPoint),
            axisLine: { lineStyle: { color: "rgba(255,255,255,0.1)" } },
            axisLabel: {
                color: "rgba(255,255,255,0.5)",
                fontSize: 10,
                rotate: props.disks.length > 4 ? 30 : 0,
            },
            axisTick: { show: false },
        },
        yAxis: {
            type: "value",
            axisLine: { show: false },
            axisLabel: {
                color: "rgba(255,255,255,0.5)",
                fontSize: 10,
                formatter: (value: number) => formatBytes(value),
            },
            splitLine: { lineStyle: { color: "rgba(255,255,255,0.06)" } },
        },
        series: [
            {
                name: "已使用",
                type: "bar",
                stack: "total",
                barWidth: "50%",
                itemStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: "#3b82f6" },
                        { offset: 1, color: "#60a5fa" },
                    ]),
                    borderRadius: [4, 4, 0, 0],
                },
                data: props.disks.map(d => d.used),
            },
            {
                name: "可用",
                type: "bar",
                stack: "total",
                barWidth: "50%",
                itemStyle: {
                    color: "rgba(255, 255, 255, 0.08)",
                    borderRadius: [4, 4, 0, 0],
                },
                data: props.disks.map(d => d.total - d.used),
            },
        ],
        tooltip: {
            trigger: "axis",
            backgroundColor: "rgba(14, 18, 28, 0.9)",
            borderColor: "rgba(255,255,255,0.1)",
            textStyle: { color: "#fff", fontSize: 12 },
            formatter: (params: any) => {
                const disk = props.disks[params[0].dataIndex];
                const usedPercent = ((disk.used / disk.total) * 100).toFixed(1);
                return `${disk.mountPoint}<br/>
                已使用: ${formatBytes(disk.used)} (${usedPercent}%)<br/>
                总计: ${formatBytes(disk.total)}`;
            },
        },
    };

    chart.setOption(option);
}

watch(() => props.disks, updateChart, { deep: true });

onMounted(() => {
    initChart();
    window.addEventListener("resize", () => chart?.resize());
});

onUnmounted(() => {
    chart?.dispose();
    window.removeEventListener("resize", () => chart?.resize());
});
</script>

<template>
    <div class="disk-chart">
        <div class="chart-header">
            <span class="chart-title">磁盘使用</span>
            <span class="chart-subtitle">{{ disks.length }} 个分区</span>
        </div>
        <div ref="chartRef" class="chart-container"></div>
    </div>
</template>

<style scoped>
.disk-chart {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 16px;
    padding: 16px;
}

.chart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
}

.chart-title {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 500;
}

.chart-subtitle {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
}

.chart-container {
    flex: 1;
    min-height: 150px;
}
</style>
