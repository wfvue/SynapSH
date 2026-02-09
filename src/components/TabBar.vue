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
    <div class="flex h-[38px] border-b border-white/10 select-none">
        <!-- 交通灯按钮占位区域 - 完全透明，不可拖拽 -->
        <div class="w-[80px] h-full shrink-0" style="-webkit-app-region: no-drag"></div>
        <div class="flex flex-1 overflow-x-auto items-end bg-neutral-900 [&::-webkit-scrollbar]:hidden pl-4">
            <div v-for="tab in tabs" :key="tab.id"
                class="group flex items-center gap-2 px-3 min-w-[140px] max-w-[240px] h-8 bg-white text-neutral-400 rounded-t-md mr-0.5 cursor-default text-[13px] transition-all duration-200 ease-out relative hover:bg-neutral-700 hover:text-neutral-200"
                :class="{ 'bg-[var(--background)] text-[var(--foreground)] shadow-[0_-2px_10px_rgba(0,0,0,0.2)] z-10': tab.id === activeTabId }"
                @click="emit('switch-tab', tab.id)">
                <span class="text-base opacity-80" :class="tab.icon || 'icon-[mdi--terminal]'"></span>
                <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis">{{ tab.title }}</span>
                <button
                    class="close-btn flex items-center justify-center w-[18px] h-[18px] rounded-full border-none bg-transparent text-inherit opacity-0 cursor-pointer transition-all duration-200 group-hover:opacity-60 hover:!opacity-100 hover:bg-white/20"
                    @click.stop="emit('close-tab', tab.id)">
                    <span class="icon-[mdi--close]"></span>
                </button>
            </div>
            <button
                class="flex items-center justify-center w-8 h-8 border-none bg-transparent text-neutral-400 cursor-pointer ml-1 rounded-md hover:bg-white/10 hover:text-white transition-colors"
                @click="emit('new-tab')">
                <span class="icon-[mdi--plus]"></span>
            </button>
        </div>
        <div class="flex-1 h-full" data-tauri-drag-region></div>
    </div>
</template>
