<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import * as echarts from "echarts";

const props = defineProps<{
    total: number;
    used: number;
    free: number;
    cached: number;
}>();

const chartRef = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;

const usedPercent = computed(() =>
    props.total > 0 ? ((props.used / props.total) * 100).toFixed(1) : "0"
);

function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function initChart() {
    if (!chartRef.value) return;
    chart = echarts.init(chartRef.value, undefined, { renderer: "canvas" });
    updateChart();
}

function updateChart() {
    if (!chart) return;

    const option: echarts.EChartsOption = {
        series: [
            {
                name: "内存",
                type: "pie",
                radius: ["60%", "80%"],
                center: ["50%", "50%"],
                avoidLabelOverlap: false,
                itemStyle: {
                    borderRadius: 4,
                    borderColor: "rgba(14, 18, 28, 0.9)",
                    borderWidth: 2,
                },
                label: { show: false },
                emphasis: {
                    label: { show: false },
                    scale: true,
                    scaleSize: 6,
                },
                data: [
                    {
                        value: props.used,
                        name: "已使用",
                        itemStyle: {
                            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
                                { offset: 0, color: "#f59e0b" },
                                { offset: 1, color: "#fbbf24" },
                            ]),
                        },
                    },
                    {
                        value: props.cached,
                        name: "缓存",
                        itemStyle: { color: "rgba(59, 130, 246, 0.6)" },
                    },
                    {
                        value: props.free,
                        name: "可用",
                        itemStyle: { color: "rgba(255, 255, 255, 0.08)" },
                    },
                ],
            },
        ],
        tooltip: {
            trigger: "item",
            backgroundColor: "rgba(14, 18, 28, 0.9)",
            borderColor: "rgba(255,255,255,0.1)",
            textStyle: { color: "#fff", fontSize: 12 },
            formatter: (params: any) => {
                return `${params.name}: ${formatBytes(params.value)}`;
            },
        },
    };

    chart.setOption(option);
}

watch(() => [props.used, props.free, props.cached], updateChart);

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
    <div class="memory-chart">
        <div class="chart-header">
            <span class="chart-title">内存使用率</span>
            <span class="chart-value">{{ usedPercent }}%</span>
        </div>
        <div class="chart-wrapper">
            <div ref="chartRef" class="chart-container"></div>
            <div class="chart-center">
                <div class="center-value">{{ formatBytes(used) }}</div>
                <div class="center-label">已使用</div>
            </div>
        </div>
        <div class="memory-legend">
            <div class="legend-item">
                <span class="legend-dot legend-dot--used"></span>
                <span class="legend-label">已使用</span>
                <span class="legend-value">{{ formatBytes(used) }}</span>
            </div>
            <div class="legend-item">
                <span class="legend-dot legend-dot--cached"></span>
                <span class="legend-label">缓存</span>
                <span class="legend-value">{{ formatBytes(cached) }}</span>
            </div>
            <div class="legend-item">
                <span class="legend-dot legend-dot--free"></span>
                <span class="legend-label">可用</span>
                <span class="legend-value">{{ formatBytes(free) }}</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.memory-chart {
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
    margin-bottom: 8px;
}

.chart-title {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 500;
}

.chart-value {
    font-size: 1.25rem;
    font-weight: 600;
    color: #fbbf24;
}

.chart-wrapper {
    position: relative;
    flex: 1;
    min-height: 160px;
}

.chart-container {
    width: 100%;
    height: 100%;
}

.chart-center {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    text-align: center;
}

.center-value {
    font-size: 1rem;
    font-weight: 600;
    color: #fff;
}

.center-label {
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.5);
}

.memory-legend {
    display: flex;
    justify-content: space-around;
    margin-top: 8px;
    padding-top: 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.legend-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
}

.legend-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
}

.legend-dot--used {
    background: linear-gradient(135deg, #f59e0b, #fbbf24);
}

.legend-dot--cached {
    background: rgba(59, 130, 246, 0.6);
}

.legend-dot--free {
    background: rgba(255, 255, 255, 0.2);
}

.legend-label {
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.5);
}

.legend-value {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.8);
    font-weight: 500;
}
</style>
