<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
    rxData: number[];
    txData: number[];
}>();

const chartRef = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;

function formatSpeed(bytesPerSec: number): string {
    if (bytesPerSec === 0) return "0 B/s";
    const k = 1024;
    const sizes = ["B/s", "KB/s", "MB/s", "GB/s"];
    const i = Math.floor(Math.log(bytesPerSec) / Math.log(k));
    return parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

const currentRx = computed(() => formatSpeed(props.rxData[props.rxData.length - 1] ?? 0));
const currentTx = computed(() => formatSpeed(props.txData[props.txData.length - 1] ?? 0));

function initChart() {
    if (!chartRef.value) return;
    chart = echarts.init(chartRef.value, undefined, { renderer: "canvas" });

    const option: echarts.EChartsOption = {
        grid: {
            top: 20,
            right: 20,
            bottom: 30,
            left: 50,
        },
        legend: {
            show: true,
            bottom: 0,
            textStyle: { color: "rgba(255,255,255,0.5)", fontSize: 10 },
            itemWidth: 12,
            itemHeight: 4,
        },
        xAxis: {
            type: "category",
            boundaryGap: false,
            data: Array.from({ length: 60 }, (_, i) => `${60 - i}s`),
            axisLine: { lineStyle: { color: "rgba(255,255,255,0.1)" } },
            axisLabel: {
                color: "rgba(255,255,255,0.5)",
                fontSize: 10,
                interval: 9,
            },
            axisTick: { show: false },
        },
        yAxis: {
            type: "value",
            axisLine: { show: false },
            axisLabel: {
                color: "rgba(255,255,255,0.5)",
                fontSize: 10,
                formatter: (value: number) => formatSpeed(value),
            },
            splitLine: { lineStyle: { color: "rgba(255,255,255,0.06)" } },
        },
        series: [
            {
                name: "下载",
                type: "line",
                smooth: true,
                symbol: "none",
                lineStyle: {
                    width: 2,
                    color: "#3b82f6",
                },
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: "rgba(59, 130, 246, 0.3)" },
                        { offset: 1, color: "rgba(59, 130, 246, 0.02)" },
                    ]),
                },
                data: props.rxData,
            },
            {
                name: "上传",
                type: "line",
                smooth: true,
                symbol: "none",
                lineStyle: {
                    width: 2,
                    color: "#f59e0b",
                },
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: "rgba(245, 158, 11, 0.3)" },
                        { offset: 1, color: "rgba(245, 158, 11, 0.02)" },
                    ]),
                },
                data: props.txData,
            },
        ],
        tooltip: {
            trigger: "axis",
            backgroundColor: "rgba(14, 18, 28, 0.9)",
            borderColor: "rgba(255,255,255,0.1)",
            textStyle: { color: "#fff", fontSize: 12 },
        },
    };

    chart.setOption(option);
}

function updateChart() {
    if (!chart) return;
    chart.setOption({
        series: [
            { data: props.rxData },
            { data: props.txData },
        ],
    });
}

watch(() => [props.rxData, props.txData], updateChart, { deep: true });

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
    <div class="network-chart">
        <div class="chart-header">
            <span class="chart-title">网络流量</span>
            <div class="chart-stats">
                <span class="stat stat--rx">↓ {{ currentRx }}</span>
                <span class="stat stat--tx">↑ {{ currentTx }}</span>
            </div>
        </div>
        <div ref="chartRef" class="chart-container"></div>
    </div>
</template>

<style scoped>
.network-chart {
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

.chart-stats {
    display: flex;
    gap: 12px;
}

.stat {
    font-size: 0.8rem;
    font-weight: 500;
}

.stat--rx {
    color: #60a5fa;
}

.stat--tx {
    color: #fbbf24;
}

.chart-container {
    flex: 1;
    min-height: 150px;
}
</style>
