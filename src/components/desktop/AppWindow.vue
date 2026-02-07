<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
    title: string;
    appId: string;
    active: boolean;
    zIndex: number;
    offset: number;
    statusText?: string;
    statusOnline?: boolean;
}>();

const emit = defineEmits<{
    close: [];
    focus: [];
}>();

const windowStyle = computed(() => ({
    top: `calc(8vh + ${props.offset}px)`,
    left: `calc(50% + ${props.offset}px)`,
    zIndex: props.zIndex,
}));
</script>

<template>
    <div class="app-window" :class="[`app-window--${appId}`, { active }]" :style="windowStyle"
        @mousedown="emit('focus')">
        <header class="app-titlebar">
            <div class="window-controls">
                <button class="control control--close" @click.stop="emit('close')"></button>
                <button class="control control--min"></button>
                <button class="control control--max"></button>
            </div>
            <div class="app-title">{{ title }}</div>
            <div class="title-actions">
                <span v-if="statusText" class="status-pill" :class="{ online: statusOnline }">
                    {{ statusText }}
                </span>
            </div>
        </header>

        <div class="app-body">
            <slot />
        </div>
    </div>
</template>

<style scoped>
.app-window {
    pointer-events: auto;
    position: absolute;
    transform: translateX(-50%);
    border-radius: 18px;
    background: rgba(14, 18, 28, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: var(--shadow-strong);
    backdrop-filter: blur(20px);
    overflow: hidden;
    transition: box-shadow 0.2s ease, transform 0.2s ease;
}

.app-window.active {
    box-shadow: 0 28px 80px rgba(0, 0, 0, 0.5);
    transform: translateX(-50%) translateY(-2px);
}

.app-window--terminal {
    width: min(1120px, 92vw);
    height: min(720px, 80vh);
}

.app-window--files {
    width: min(980px, 90vw);
    height: min(680px, 76vh);
}

.app-window--monitor,
.app-window--settings,
.app-window--app-center {
    width: min(860px, 88vw);
    height: min(560px, 70vh);
}

.app-titlebar {
    display: grid;
    grid-template-columns: 120px 1fr 160px;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(18, 22, 32, 0.8);
}

.window-controls {
    display: flex;
    gap: 8px;
}

.control {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 255, 255, 0.25);
    cursor: pointer;
}

.control--close {
    background: #ff6b6b;
}

.control--min {
    background: #ffd166;
}

.control--max {
    background: #9ae66e;
}

.app-title {
    text-align: center;
    font-size: 0.9rem;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    text-transform: uppercase;
}

.title-actions {
    display: flex;
    justify-content: flex-end;
}

.status-pill {
    font-size: 0.72rem;
    padding: 4px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
}

.status-pill.online {
    background: rgba(94, 234, 212, 0.18);
    color: #bff4ea;
}

.app-body {
    height: calc(100% - 48px);
}
</style>
