<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
    cpuData: number[];
}>();

const chartRef = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;

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
            min: 0,
            max: 100,
            splitNumber: 4,
            axisLine: { show: false },
            axisLabel: { color: "rgba(255,255,255,0.5)", fontSize: 10, formatter: "{value}%" },
            splitLine: { lineStyle: { color: "rgba(255,255,255,0.06)" } },
        },
        series: [
            {
                name: "CPU",
                type: "line",
                smooth: true,
                symbol: "none",
                sampling: "lttb",
                lineStyle: {
                    width: 2,
                    color: new echarts.graphic.LinearGradient(0, 0, 1, 0, [
                        { offset: 0, color: "#10b981" },
                        { offset: 1, color: "#34d399" },
                    ]),
                },
                areaStyle: {
                    color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                        { offset: 0, color: "rgba(16, 185, 129, 0.4)" },
                        { offset: 1, color: "rgba(16, 185, 129, 0.02)" },
                    ]),
                },
                data: props.cpuData,
            },
        ],
        tooltip: {
            trigger: "axis",
            backgroundColor: "rgba(14, 18, 28, 0.9)",
            borderColor: "rgba(255,255,255,0.1)",
            textStyle: { color: "#fff", fontSize: 12 },
            formatter: (params: any) => {
                const value = params[0]?.value ?? 0;
                return `CPU: ${value.toFixed(1)}%`;
            },
        },
    };

    chart.setOption(option);
}

function updateChart() {
    if (!chart) return;
    chart.setOption({
        series: [{ data: props.cpuData }],
    });
}

watch(() => props.cpuData, updateChart, { deep: true });

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
    <div class="cpu-chart">
        <div class="chart-header">
            <span class="chart-title">CPU 使用率</span>
            <span class="chart-value">{{ (cpuData[cpuData.length - 1] ?? 0).toFixed(1) }}%</span>
        </div>
        <div ref="chartRef" class="chart-container"></div>
    </div>
</template>

<style scoped>
.cpu-chart {
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

.chart-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: #34d399;
}

.chart-container {
    flex: 1;
    min-height: 150px;
}
</style>
