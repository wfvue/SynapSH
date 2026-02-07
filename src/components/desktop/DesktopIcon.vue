<script setup lang="ts">
export interface DesktopIconItem {
    id: string;
    label: string;
    icon: string; // Iconify class name, e.g. "icon-[mdi--folder]"
    color: string;
    app?: string;
}

const props = defineProps<{
    item: DesktopIconItem;
    selected: boolean;
}>();

const emit = defineEmits<{
    select: [id: string];
    open: [app: string | undefined];
}>();

function handleClick() {
    emit("select", props.item.id);
}

function handleDblClick() {
    emit("open", props.item.app);
}
</script>

<template>
    <button class="desktop-icon" :class="{ selected }" @click.stop="handleClick" @dblclick.stop="handleDblClick">
        <div class="icon-wrapper" :style="{ background: item.color }">
            <span :class="item.icon"></span>
        </div>
        <span class="icon-label">{{ item.label }}</span>
    </button>
</template>

<style scoped>
.desktop-icon {
    background: transparent;
    border: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    color: var(--text-primary);
    cursor: pointer;
    padding: 8px;
    border-radius: 8px;
    transition: background 0.15s ease;
    width: 80px;
}

.desktop-icon.selected {
    background: rgba(255, 255, 255, 0.12);
}

.desktop-icon:hover {
    background: rgba(255, 255, 255, 0.06);
}

.icon-wrapper {
    width: 56px;
    height: 56px;
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow:
        0 4px 12px rgba(0, 0, 0, 0.15),
        0 1px 3px rgba(0, 0, 0, 0.1),
        inset 0 1px 0 rgba(255, 255, 255, 0.2);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.desktop-icon:hover .icon-wrapper {
    transform: scale(1.08) translateY(-2px);
    box-shadow:
        0 8px 24px rgba(0, 0, 0, 0.25),
        0 2px 6px rgba(0, 0, 0, 0.15),
        inset 0 1px 0 rgba(255, 255, 255, 0.25);
}

.icon-wrapper span {
    font-size: 32px;
    color: rgba(255, 255, 255, 0.95);
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.2));
}

.icon-label {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.9);
    text-align: center;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.5);
    max-width: 72px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
