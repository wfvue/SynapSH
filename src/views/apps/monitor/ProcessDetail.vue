<!--
  ProcessDetail.vue - 进程详情抽屉组件
  展示进程详细信息，支持进程操作（终止、暂停、恢复）
-->
<script setup lang="ts">
import { api } from "@/lib/api";
import type { ProcessInfo } from "./ProcessList.vue";

const props = defineProps<{
    process: ProcessInfo | null;
    sessionId: string;
    visible: boolean;
}>();

const emit = defineEmits<{
    close: [];
    killed: [pid: number];
}>();

// 格式化字节
function formatBytes(kb: number): string {
    if (kb === 0) return "0 B";
    const bytes = kb * 1024;
    const sizes = ["KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    if (i === 0) return `${kb} KB`;
    return parseFloat((bytes / Math.pow(1024, i)).toFixed(2)) + " " + sizes[i - 1];
}

// 获取状态颜色
function getStatusColor(status: string): string {
    const firstChar = status.charAt(0);
    switch (firstChar) {
        case 'R': return '#34d399';
        case 'S': return '#7dd3fc';
        case 'Z': return '#fbbf24';
        case 'D': return '#f87171';
        case 'T': return '#9ca3af';
        default: return 'rgba(255, 255, 255, 0.5)';
    }
}

async function handleKill(signal: number) {
    if (!props.process) return;
    
    const signalNames: Record<number, string> = {
        1: 'SIGHUP',
        9: 'SIGKILL',
        15: 'SIGTERM',
        18: 'SIGCONT',
        19: 'SIGSTOP',
    };
    
    const signalName = signalNames[signal] || `Signal ${signal}`;
    if (!confirm(`确定要向进程 ${props.process.pid} 发送 ${signalName} 信号吗？`)) return;
    
    try {
        await api.killProcess(props.sessionId, props.process.pid, signal);
        emit('killed', props.process.pid);
        emit('close');
    } catch (e) {
        console.error("Failed to kill process:", e);
        alert(`终止进程失败: ${e}`);
    }
}

function handleClose() {
    emit('close');
}

function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
        handleClose();
    }
}
</script>

<template>
    <Teleport to="body">
        <Transition name="drawer">
            <div v-if="visible && process" class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex justify-end"
                @click="handleBackdropClick">
                <div
                    class="w-[420px] h-full bg-gradient-to-b from-[#0e121c] to-[#151b2b] border-l border-white/[0.06] flex flex-col shadow-[-8px_0_32px_rgba(0,0,0,0.4)]">
                    <div class="flex justify-between items-center p-5 border-b border-white/[0.06]">
                        <div class="flex items-center gap-3">
                            <span
                                class="w-12 h-12 bg-gradient-to-br from-sky-400/30 to-sky-400/10 rounded-xl flex items-center justify-center text-2xl font-bold text-sky-400">{{ process.name.charAt(0).toUpperCase() }}</span>
                            <div class="flex flex-col gap-0.5">
                                <h3 class="text-lg font-semibold text-foreground/90">{{ process.name }}</h3>
                                <span class="text-xs text-muted-foreground font-mono">PID: {{ process.pid }}</span>
                            </div>
                        </div>
                        <button
                            class="w-9 h-9 rounded-xl bg-white/5 text-muted-foreground hover:bg-white/10 hover:text-foreground/80 flex items-center justify-center text-lg transition-colors"
                            @click="handleClose">
                            <span class="icon-[mdi--close]"></span>
                        </button>
                    </div>

                    <div class="flex-1 overflow-y-auto p-5">
                        <!-- 状态概览 -->
                        <div class="grid grid-cols-3 gap-3 mb-6">
                            <div class="bg-white/[0.03] border border-white/[0.05] rounded-xl p-4 text-center">
                                <span class="block text-[0.7rem] text-muted-foreground uppercase tracking-wider mb-1.5">状态</span>
                                <span class="block text-xl font-semibold" :style="{ color: getStatusColor(process.status) }">
                                    {{ process.statusDesc }}
                                </span>
                            </div>
                            <div class="bg-white/[0.03] border border-white/[0.05] rounded-xl p-4 text-center">
                                <span class="block text-[0.7rem] text-muted-foreground uppercase tracking-wider mb-1.5">CPU</span>
                                <span class="block text-xl font-semibold"
                                    :class="process.cpu > 50 ? 'text-amber-500' : 'text-foreground/90'">
                                    {{ process.cpu.toFixed(1) }}%
                                </span>
                            </div>
                            <div class="bg-white/[0.03] border border-white/[0.05] rounded-xl p-4 text-center">
                                <span class="block text-[0.7rem] text-muted-foreground uppercase tracking-wider mb-1.5">内存</span>
                                <span class="block text-xl font-semibold"
                                    :class="process.memory > 50 ? 'text-amber-500' : 'text-foreground/90'">
                                    {{ process.memory.toFixed(1) }}%
                                </span>
                            </div>
                        </div>

                        <!-- 详细信息网格 -->
                        <div class="mb-6">
                            <h4 class="flex items-center gap-2 text-sm font-medium text-foreground/70 mb-3 pb-2 border-b border-white/[0.05]">
                                <span class="icon-[mdi--information-outline] text-base text-sky-400/70"></span>
                                基本信息
                            </h4>
                            <div class="grid grid-cols-2 gap-3">
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">进程 ID</span>
                                    <span class="text-sm text-foreground/80 font-mono">{{ process.pid }}</span>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">用户</span>
                                    <span class="text-sm text-foreground/80">{{ process.user }}</span>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">状态</span>
                                    <span class="text-sm text-foreground/80">{{ process.statusDesc }} ({{ process.status }})</span>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">启动时间</span>
                                    <span class="text-sm text-foreground/80">{{ process.startTime }}</span>
                                </div>
                            </div>
                        </div>

                        <!-- 内存信息 -->
                        <div class="mb-6">
                            <h4 class="flex items-center gap-2 text-sm font-medium text-foreground/70 mb-3 pb-2 border-b border-white/[0.05]">
                                <span class="icon-[mdi--memory] text-base text-sky-400/70"></span>
                                内存使用
                            </h4>
                            <div class="grid grid-cols-2 gap-3">
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">物理内存 (RSS)</span>
                                    <span class="text-sm text-foreground/80 font-mono">{{ formatBytes(process.rss) }}</span>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">虚拟内存 (VSZ)</span>
                                    <span class="text-sm text-foreground/80 font-mono">{{ formatBytes(process.vsz) }}</span>
                                </div>
                                <div class="flex flex-col gap-1">
                                    <span class="text-[0.7rem] text-muted-foreground">内存占比</span>
                                    <span class="text-sm text-foreground/80 font-mono">{{ process.memory.toFixed(2) }}%</span>
                                </div>
                            </div>
                        </div>

                        <!-- 命令行 -->
                        <div class="mb-6">
                            <h4 class="flex items-center gap-2 text-sm font-medium text-foreground/70 mb-3 pb-2 border-b border-white/[0.05]">
                                <span class="icon-[mdi--console] text-base text-sky-400/70"></span>
                                命令行
                            </h4>
                            <div class="bg-black/30 border border-white/[0.06] rounded-xl p-3 overflow-x-auto">
                                <code
                                    class="font-mono text-xs text-sky-400/90 leading-relaxed break-all">{{ process.command }}</code>
                            </div>
                        </div>

                        <!-- 操作按钮 -->
                        <div class="mb-6">
                            <h4 class="flex items-center gap-2 text-sm font-medium text-foreground/70 mb-3 pb-2 border-b border-white/[0.05]">
                                <span class="icon-[mdi--cog-outline] text-base text-sky-400/70"></span>
                                进程操作
                            </h4>
                            <div class="grid grid-cols-2 gap-2">
                                <button
                                    class="flex items-center justify-center gap-1.5 py-2.5 px-3 rounded-lg text-xs font-medium bg-red-500/10 text-red-500 hover:bg-red-500/20 transition-colors"
                                    @click="handleKill(15)">
                                    <span class="icon-[mdi--close] text-sm"></span>
                                    终止进程 (SIGTERM)
                                </button>
                                <button
                                    class="flex items-center justify-center gap-1.5 py-2.5 px-3 rounded-lg text-xs font-medium bg-amber-500/10 text-amber-500 hover:bg-amber-500/20 transition-colors"
                                    @click="handleKill(9)">
                                    <span class="icon-[mdi--lightning-bolt] text-sm"></span>
                                    强制终止 (SIGKILL)
                                </button>
                                <button
                                    class="flex items-center justify-center gap-1.5 py-2.5 px-3 rounded-lg text-xs font-medium bg-sky-400/10 text-sky-400 hover:bg-sky-400/20 transition-colors"
                                    @click="handleKill(19)">
                                    <span class="icon-[mdi--pause] text-sm"></span>
                                    暂停 (SIGSTOP)
                                </button>
                                <button
                                    class="flex items-center justify-center gap-1.5 py-2.5 px-3 rounded-lg text-xs font-medium bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 transition-colors"
                                    @click="handleKill(18)">
                                    <span class="icon-[mdi--play] text-sm"></span>
                                    恢复 (SIGCONT)
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<style scoped>
/* 过渡动画 */
.drawer-enter-active,
.drawer-leave-active {
    transition: opacity 0.3s ease;
}

.drawer-enter-from,
.drawer-leave-to {
    opacity: 0;
}

.drawer-enter-active>div,
.drawer-leave-active>div {
    transition: transform 0.3s ease;
}

.drawer-enter-from>div,
.drawer-leave-to>div {
    transform: translateX(100%);
}
</style>
