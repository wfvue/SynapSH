<script setup lang="ts">
export interface DockItem {
    id: string;
    label: string;
    icon: string;
    app?: string;
}

defineProps<{
    items: DockItem[];
    openApps: string[];
}>();

const emit = defineEmits<{
    openApp: [app: string];
}>();

function handleClick(app: string | undefined) {
    if (app) {
        emit("openApp", app);
    }
}
</script>

<template>
    <section class="dock">
        <button v-for="item in items" :key="item.id" class="dock-item"
            :class="{ active: item.app && openApps.includes(item.app) }" :title="item.label"
            @click.stop="handleClick(item.app)">
            <span :class="item.icon"></span>
            <span class="dock-indicator" v-if="item.app && openApps.includes(item.app)"></span>
        </button>
    </section>
</template>

<style scoped>
.dock {
    position: absolute;
    bottom: 8px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 16px;
    background: rgba(255, 255, 255, 0.15);
    border: 1px solid rgba(255, 255, 255, 0.2);
    backdrop-filter: blur(24px);
    z-index: 4;
}

.dock-item {
    position: relative;
    width: 48px;
    height: 48px;
    border-radius: 12px;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.dock-item:hover {
    transform: translateY(-8px) scale(1.15);
}

.dock-item.active {
    transform: translateY(-2px);
}

.dock-item span:first-child {
    font-size: 32px;
    color: rgba(255, 255, 255, 0.9);
    filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
}

.dock-indicator {
    position: absolute;
    bottom: -6px;
    left: 50%;
    transform: translateX(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.8);
}
</style>
