<script setup lang="ts">


export interface Tab {
    id: string;
    title: string;
    icon?: string;
    view: "machine-manager" | "desktop";
}

const props = defineProps<{
    tabs: Tab[];
    activeTabId: string;
}>();

const emit = defineEmits<{
    (e: "switch-tab", id: string): void;
    (e: "close-tab", id: string): void;
    (e: "new-tab"): void;
}>();
</script>

<template>
    <div class="tab-bar">
        <div class="tab-list">
            <div v-for="tab in tabs" :key="tab.id" class="tab-item" :class="{ active: tab.id === activeTabId }"
                @click="emit('switch-tab', tab.id)">
                <span class="tab-icon" :class="tab.icon || 'icon-[mdi--terminal]'"></span>
                <span class="tab-title">{{ tab.title }}</span>
                <button class="close-btn" @click.stop="emit('close-tab', tab.id)">
                    <span class="icon-[mdi--close]"></span>
                </button>
            </div>
            <button class="new-tab-btn" @click="emit('new-tab')">
                <span class="icon-[mdi--plus]"></span>
            </button>
        </div>
        <div class="drag-region" data-tauri-drag-region></div>
    </div>
</template>

<style scoped>
.tab-bar {
    display: flex;
    height: 38px;
    background: #18181b;
    /* Zinc 900 */
    border-bottom: 1px solid #27272a;
    /* Zinc 800 */
    user-select: none;
    padding-left: 80px;
    /* Space for traffic lights */
}

.tab-list {
    display: flex;
    flex: 1;
    overflow-x: auto;
    align-items: flex-end;
    /* Align tabs to bottom */
}

/* Hide scrollbar */
.tab-list::-webkit-scrollbar {
    display: none;
}

.tab-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 12px;
    min-width: 140px;
    max-width: 240px;
    height: 32px;
    background: #27272a;
    /* Inactive tab bg */
    color: #a1a1aa;
    /* Zinc 400 */
    border-radius: 6px 6px 0 0;
    margin-right: 2px;
    cursor: default;
    font-size: 13px;
    transition: all 0.2s ease;
    position: relative;
}

.tab-item:hover {
    background: #3f3f46;
    /* Zinc 700 */
    color: #e4e4e7;
    /* Zinc 200 */
}

.tab-item.active {
    background: var(--background);
    /* Match app bg */
    color: var(--foreground);
    box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.2);
    z-index: 10;
}

.tab-icon {
    font-size: 16px;
    opacity: 0.8;
}

.tab-title {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    border: none;
    background: transparent;
    color: inherit;
    opacity: 0;
    cursor: pointer;
    transition: all 0.2s;
}

.tab-item:hover .close-btn {
    opacity: 0.6;
}

.close-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    opacity: 1 !important;
}

.new-tab-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    background: transparent;
    color: #a1a1aa;
    cursor: pointer;
    margin-left: 4px;
    border-radius: 6px;
}

.new-tab-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
}

.drag-region {
    flex: 1;
    /* Take up remaining space */
    height: 100%;
}
</style>
