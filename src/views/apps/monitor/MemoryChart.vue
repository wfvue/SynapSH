<!--
  MemoryChart.vue - 内存使用率图表组件
  使用 ECharts 展示内存使用环形图
-->
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
    <div class="h-full flex flex-col bg-card/50 rounded-2xl p-4 border border-border/50">
        <div class="flex justify-between items-center mb-2">
            <span class="text-sm font-medium text-foreground/80">内存使用率</span>
            <span class="text-xl font-semibold text-amber-400">{{ usedPercent }}%</span>
        </div>
        <div class="relative flex-1 min-h-[160px]">
            <div ref="chartRef" class="w-full h-full"></div>
            <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-center">
                <div class="text-base font-semibold text-foreground">{{ formatBytes(used) }}</div>
                <div class="text-[0.7rem] text-muted-foreground">已使用</div>
            </div>
        </div>
        <div class="flex justify-around mt-2 pt-3 border-t border-border/30">
            <div class="flex flex-col items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-gradient-to-br from-amber-500 to-amber-300"></span>
                <span class="text-[0.7rem] text-muted-foreground">已使用</span>
                <span class="text-xs text-foreground/80 font-medium">{{ formatBytes(used) }}</span>
            </div>
            <div class="flex flex-col items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-blue-500/60"></span>
                <span class="text-[0.7rem] text-muted-foreground">缓存</span>
                <span class="text-xs text-foreground/80 font-medium">{{ formatBytes(cached) }}</span>
            </div>
            <div class="flex flex-col items-center gap-1">
                <span class="w-2 h-2 rounded-full bg-white/20"></span>
                <span class="text-[0.7rem] text-muted-foreground">可用</span>
                <span class="text-xs text-foreground/80 font-medium">{{ formatBytes(free) }}</span>
            </div>
        </div>
    </div>
</template>
