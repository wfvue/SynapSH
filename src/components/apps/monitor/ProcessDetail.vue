<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
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
        await invoke("kill_process", { 
            sessionId: props.sessionId, 
            pid: props.process.pid, 
            signal 
        });
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
            <div v-if="visible && process" class="drawer-overlay" @click="handleBackdropClick">
                <div class="drawer-panel">
                    <div class="drawer-header">
                        <div class="process-title">
                            <span class="process-icon-large">{{ process.name.charAt(0).toUpperCase() }}</span>
                            <div class="process-title-text">
                                <h3 class="process-name">{{ process.name }}</h3>
                                <span class="process-pid">PID: {{ process.pid }}</span>
                            </div>
                        </div>
                        <button class="close-btn" @click="handleClose">
                            <span class="icon-[mdi--close]"></span>
                        </button>
                    </div>

                    <div class="drawer-content">
                        <!-- 状态概览 -->
                        <div class="status-overview">
                            <div class="status-item">
                                <span class="status-label">状态</span>
                                <span class="status-value" :style="{ color: getStatusColor(process.status) }">
                                    {{ process.statusDesc }}
                                </span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">CPU</span>
                                <span class="status-value" :class="{ high: process.cpu > 50 }">
                                    {{ process.cpu.toFixed(1) }}%
                                </span>
                            </div>
                            <div class="status-item">
                                <span class="status-label">内存</span>
                                <span class="status-value" :class="{ high: process.memory > 50 }">
                                    {{ process.memory.toFixed(1) }}%
                                </span>
                            </div>
                        </div>

                        <!-- 详细信息网格 -->
                        <div class="detail-section">
                            <h4 class="section-title">
                                <span class="icon-[mdi--information-outline]"></span>
                                基本信息
                            </h4>
                            <div class="detail-grid">
                                <div class="detail-item">
                                    <span class="detail-label">进程 ID</span>
                                    <span class="detail-value">{{ process.pid }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">父进程 ID</span>
                                    <span class="detail-value">{{ process.ppid || '-' }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">用户</span>
                                    <span class="detail-value">{{ process.user }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">优先级 (Nice)</span>
                                    <span class="detail-value">{{ process.nice }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">线程数</span>
                                    <span class="detail-value">{{ process.threads }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">原始状态</span>
                                    <span class="detail-value">{{ process.status }}</span>
                                </div>
                            </div>
                        </div>

                        <!-- 内存信息 -->
                        <div class="detail-section">
                            <h4 class="section-title">
                                <span class="icon-[mdi--memory]"></span>
                                内存使用
                            </h4>
                            <div class="detail-grid">
                                <div class="detail-item">
                                    <span class="detail-label">物理内存 (RSS)</span>
                                    <span class="detail-value">{{ formatBytes(process.rss) }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">虚拟内存 (VSZ)</span>
                                    <span class="detail-value">{{ formatBytes(process.vsz) }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">内存占比</span>
                                    <span class="detail-value">{{ process.memory.toFixed(2) }}%</span>
                                </div>
                            </div>
                        </div>

                        <!-- 时间信息 -->
                        <div class="detail-section">
                            <h4 class="section-title">
                                <span class="icon-[mdi--clock-outline]"></span>
                                时间信息
                            </h4>
                            <div class="detail-grid">
                                <div class="detail-item">
                                    <span class="detail-label">启动时间</span>
                                    <span class="detail-value">{{ process.startTime }}</span>
                                </div>
                                <div class="detail-item">
                                    <span class="detail-label">运行时长</span>
                                    <span class="detail-value">{{ process.elapsedTime }}</span>
                                </div>
                            </div>
                        </div>

                        <!-- 命令行 -->
                        <div class="detail-section">
                            <h4 class="section-title">
                                <span class="icon-[mdi--console]"></span>
                                命令行
                            </h4>
                            <div class="command-box">
                                <code>{{ process.command }}</code>
                            </div>
                        </div>

                        <!-- 操作按钮 -->
                        <div class="detail-section">
                            <h4 class="section-title">
                                <span class="icon-[mdi--cog-outline]"></span>
                                进程操作
                            </h4>
                            <div class="action-buttons">
                                <button class="action-btn btn-terminate" @click="handleKill(15)">
                                    <span class="icon-[mdi--close]"></span>
                                    终止进程 (SIGTERM)
                                </button>
                                <button class="action-btn btn-force-kill" @click="handleKill(9)">
                                    <span class="icon-[mdi--lightning-bolt]"></span>
                                    强制终止 (SIGKILL)
                                </button>
                                <button class="action-btn btn-pause" @click="handleKill(19)">
                                    <span class="icon-[mdi--pause]"></span>
                                    暂停 (SIGSTOP)
                                </button>
                                <button class="action-btn btn-resume" @click="handleKill(18)">
                                    <span class="icon-[mdi--play]"></span>
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
.drawer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(4px);
    z-index: 1000;
    display: flex;
    justify-content: flex-end;
}

.drawer-panel {
    width: 420px;
    height: 100%;
    background: linear-gradient(180deg, #0e121c 0%, #151b2b 100%);
    border-left: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    box-shadow: -8px 0 32px rgba(0, 0, 0, 0.4);
}

.drawer-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.process-title {
    display: flex;
    align-items: center;
    gap: 12px;
}

.process-icon-large {
    width: 48px;
    height: 48px;
    background: linear-gradient(135deg, rgba(125, 211, 252, 0.3), rgba(125, 211, 252, 0.1));
    border-radius: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.5rem;
    font-weight: 700;
    color: #7dd3fc;
}

.process-title-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.process-name {
    font-size: 1.1rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
    margin: 0;
}

.process-pid {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
    font-family: monospace;
}

.close-btn {
    width: 36px;
    height: 36px;
    border: none;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    color: rgba(255, 255, 255, 0.5);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.2rem;
    transition: all 0.2s;
}

.close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.8);
}

.drawer-content {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
}

.status-overview {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
    margin-bottom: 24px;
}

.status-item {
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 12px;
    padding: 16px;
    text-align: center;
}

.status-label {
    display: block;
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.4);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 6px;
}

.status-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
}

.status-value.high {
    color: #f59e0b;
}

.detail-section {
    margin-bottom: 24px;
}

.section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.85rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.7);
    margin: 0 0 12px 0;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.section-title span {
    font-size: 1rem;
    color: rgba(125, 211, 252, 0.7);
}

.detail-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
}

.detail-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.detail-label {
    font-size: 0.7rem;
    color: rgba(255, 255, 255, 0.4);
}

.detail-value {
    font-size: 0.85rem;
    color: rgba(255, 255, 255, 0.8);
    font-family: monospace;
}

.command-box {
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.06);
    border-radius: 10px;
    padding: 12px;
    overflow-x: auto;
}

.command-box code {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.75rem;
    color: rgba(125, 211, 252, 0.9);
    line-height: 1.5;
    word-break: break-all;
}

.action-buttons {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
}

.action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 12px;
    border: none;
    border-radius: 8px;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
}

.action-btn span {
    font-size: 0.9rem;
}

.btn-terminate {
    background: rgba(255, 107, 107, 0.1);
    color: #ff6b6b;
}

.btn-terminate:hover {
    background: rgba(255, 107, 107, 0.2);
}

.btn-force-kill {
    background: rgba(245, 158, 11, 0.1);
    color: #fbbf24;
}

.btn-force-kill:hover {
    background: rgba(245, 158, 11, 0.2);
}

.btn-pause {
    background: rgba(125, 211, 252, 0.1);
    color: #7dd3fc;
}

.btn-pause:hover {
    background: rgba(125, 211, 252, 0.2);
}

.btn-resume {
    background: rgba(16, 185, 129, 0.1);
    color: #34d399;
}

.btn-resume:hover {
    background: rgba(16, 185, 129, 0.2);
}

/* 过渡动画 */
.drawer-enter-active,
.drawer-leave-active {
    transition: opacity 0.3s ease;
}

.drawer-enter-from,
.drawer-leave-to {
    opacity: 0;
}

.drawer-enter-active .drawer-panel,
.drawer-leave-active .drawer-panel {
    transition: transform 0.3s ease;
}

.drawer-enter-from .drawer-panel,
.drawer-leave-to .drawer-panel {
    transform: translateX(100%);
}
</style>
