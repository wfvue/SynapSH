<!--
  CpuChart.vue - CPU 使用率图表组件
  使用 ECharts 展示 CPU 历史使用率曲线
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from "vue";
import * as echarts from "echarts";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";

const { text } = useInterfaceLanguage();

const props = defineProps<{
  cpuData: number[];
  coreCount?: number;
}>();

const chartRef = ref<HTMLDivElement | null>(null);
let chart: echarts.ECharts | null = null;
const resizeChart = () => chart?.resize();

const currentValue = computed(() => {
  const val = props.cpuData[props.cpuData.length - 1] ?? 0;
  return (Number.isFinite(val) ? val : 0).toFixed(1);
});

const avgValue = computed(() => {
  const values = props.cpuData.filter(Number.isFinite);
  if (values.length === 0) return "0.0";
  const sum = values.reduce((a, b) => a + b, 0);
  return (sum / values.length).toFixed(1);
});

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
  window.addEventListener("resize", resizeChart);
});

onUnmounted(() => {
  chart?.dispose();
  window.removeEventListener("resize", resizeChart);
});
</script>

<template>
  <div class="h-full flex flex-col bg-card/50 rounded-2xl p-4 border border-border/50">
    <div class="flex justify-between items-center mb-3">
      <div class="flex items-center gap-2.5">
        <span class="text-sm font-medium text-foreground/80">{{ text("CPU usage", "CPU 使用率") }}</span>
        <span
          v-if="coreCount"
          class="text-[0.7rem] text-muted-foreground bg-muted/50 px-2 py-0.5 rounded"
          >{{ coreCount }} {{ text("cores", "核") }}</span
        >
      </div>
      <div class="flex gap-4">
        <div class="flex flex-col items-end gap-0.5">
          <span class="text-[0.65rem] text-muted-foreground uppercase">{{ text("Current", "当前") }}</span>
          <span class="text-[1.1rem] font-semibold text-emerald-400 font-mono"
            >{{ currentValue }}%</span
          >
        </div>
        <div class="flex flex-col items-end gap-0.5">
          <span class="text-[0.65rem] text-muted-foreground uppercase">{{ text("Average", "平均") }}</span>
          <span class="text-[0.9rem] font-semibold text-foreground/70 font-mono"
            >{{ avgValue }}%</span
          >
        </div>
      </div>
    </div>
    <div ref="chartRef" class="flex-1 min-h-[150px]"></div>
  </div>
</template>
