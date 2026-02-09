<!--
  SystemOverview.vue - 系统概览组件
  展示主机名、运行时间、CPU、内存等系统信息
-->
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

// 格式化运行时间
function formatUptime(uptime: string | undefined): string {
    if (!uptime) return "---";
    // 将 "up 3 days, 6 hours, 8 minutes" 转换为 "3天6时8分"
    return uptime
        .replace(/^up\s+/, "")  // 移除开头的 "up "
        .replace(/days?/g, "天")
        .replace(/hours?/g, "时")
        .replace(/minutes?/g, "分")
        .replace(/,/g, "")      // 移除逗号
        .trim();
}
</script>

<template>
    <div class="grid grid-cols-3 max-[800px]:grid-cols-2 max-[500px]:grid-cols-1 gap-3">
        <div
            class="flex items-center gap-3 px-4 py-4 bg-card/50 border border-border/50 rounded-xl hover:bg-card/70 hover:border-border/80 transition-colors">
            <span class="icon-[mdi--server] text-2xl text-sky-400/70"></span>
            <div class="flex flex-col gap-0.5 min-w-0">
                <span class="text-[0.7rem] text-muted-foreground uppercase tracking-wider">主机名</span>
                <span class="text-sm text-foreground/90 font-medium truncate">{{ hostname || "---" }}</span>
            </div>
        </div>

        <div
            class="flex items-center gap-3 px-4 py-4 bg-card/50 border border-border/50 rounded-xl hover:bg-card/70 hover:border-border/80 transition-colors">
            <span class="icon-[mdi--clock-outline] text-2xl text-sky-400/70"></span>
            <div class="flex flex-col gap-0.5 min-w-0">
                <span class="text-[0.7rem] text-muted-foreground uppercase tracking-wider">运行时间</span>
                <span class="text-sm text-foreground/90 font-medium truncate">{{ formatUptime(uptime) }}</span>
            </div>
        </div>

        <div
            class="flex items-center gap-3 px-4 py-4 bg-card/50 border border-border/50 rounded-xl hover:bg-card/70 hover:border-border/80 transition-colors">
            <span class="icon-[mdi--chip] text-2xl text-sky-400/70"></span>
            <div class="flex flex-col gap-0.5 min-w-0">
                <span class="text-[0.7rem] text-muted-foreground uppercase tracking-wider">CPU 核心</span>
                <span class="text-sm text-foreground/90 font-medium truncate">{{ cpuCores ?? "---" }} 核</span>
            </div>
        </div>

        <div
            class="flex items-center gap-3 px-4 py-4 bg-card/50 border border-border/50 rounded-xl hover:bg-card/70 hover:border-border/80 transition-colors">
            <span class="icon-[mdi--memory] text-2xl text-sky-400/70"></span>
            <div class="flex flex-col gap-0.5 min-w-0">
                <span class="text-[0.7rem] text-muted-foreground uppercase tracking-wider">总内存</span>
                <span class="text-sm text-foreground/90 font-medium truncate">{{ totalMemory ? formatBytes(totalMemory) : "---" }}</span>
            </div>
        </div>

        <div
            class="flex items-center gap-3 px-4 py-4 bg-card/50 border border-border/50 rounded-xl hover:bg-card/70 hover:border-border/80 transition-colors">
            <span class="icon-[mdi--gauge] text-2xl text-sky-400/70"></span>
            <div class="flex flex-col gap-0.5 min-w-0">
                <span class="text-[0.7rem] text-muted-foreground uppercase tracking-wider">负载平均</span>
                <span class="text-sm text-foreground/90 font-medium truncate" v-if="loadAverage">
                    {{ loadAverage[0].toFixed(2) }} / {{ loadAverage[1].toFixed(2) }} / {{ loadAverage[2].toFixed(2) }}
                </span>
                <span class="text-sm text-foreground/90 font-medium truncate" v-else>---</span>
            </div>
        </div>

        <div
            class="flex items-center gap-3 px-4 py-4 bg-card/50 border border-border/50 rounded-xl hover:bg-card/70 hover:border-border/80 transition-colors">
            <span class="icon-[mdi--linux] text-2xl text-sky-400/70"></span>
            <div class="flex flex-col gap-0.5 min-w-0">
                <span class="text-[0.7rem] text-muted-foreground uppercase tracking-wider">内核版本</span>
                <span class="text-sm text-foreground/90 font-medium truncate">{{ kernelVersion || "---" }}</span>
            </div>
        </div>
    </div>
</template>
