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
    <!-- 标题栏区域 - 与插件标题栏高度一致，留出右侧按钮空间 -->
    <div class="h-[40px] flex items-end select-none relative" style="padding-right: var(--tauri-frame-controls-width, 138px);">
        <!-- Brand / Icon - 拖拽区域 -->
        <div class="w-11 h-[32px] flex items-center justify-center shrink-0" data-tauri-drag-region>
            <img src="/icons/32x32.png" class="w-4 h-4 opacity-60" />
        </div>

        <!-- Tabs Container -->
        <div class="flex overflow-x-auto items-end [&::-webkit-scrollbar]:hidden pl-1" style="max-width: calc(100% - var(--tauri-frame-controls-width, 138px) - 100px);">
            <!-- Tab Items -->
            <div
                v-for="(tab, index) in tabs"
                :key="tab.id"
                class="group/tab relative flex items-center h-[32px] min-w-[140px] max-w-[200px] px-3 mr-0.5 cursor-default text-[13px] transition-all duration-150 ease-out"
                :class="[
                    tab.id === activeTabId
                        ? 'bg-[#35363a] text-[#e8eaed] z-10 rounded-t-md'
                        : 'text-[#9aa0a6] hover:bg-[#292a2d] hover:text-[#bdc1c6] rounded-t-md'
                ]"
                @click="emit('switch-tab', tab.id)"
            >
                <!-- Tab Content -->
                <span class="text-[14px] mr-2" :class="tab.icon || 'icon-[mdi--terminal]'"></span>
                <span class="flex-1 whitespace-nowrap overflow-hidden text-ellipsis text-[12px]">{{ tab.title }}</span>
                
                <!-- Close Button -->
                <button
                    class="flex items-center justify-center w-4 h-4 ml-1.5 rounded-full border-none bg-transparent text-[#9aa0a6] opacity-0 transition-all duration-150 group-hover/tab:opacity-100 hover:!opacity-100 hover:bg-[#5f6368] hover:text-white"
                    :class="{ 'opacity-100': tab.id === activeTabId }"
                    @click.stop="emit('close-tab', tab.id)"
                >
                    <span class="icon-[mdi--close] text-[12px]"></span>
                </button>
            </div>

            <!-- New Tab Button -->
            <button
                class="flex items-center justify-center w-7 h-7 mb-0.5 ml-1 rounded-full border-none bg-transparent text-[#9aa0a6] cursor-pointer transition-all duration-150 hover:bg-[#3c4043] hover:text-[#bdc1c6]"
                @click="emit('new-tab')"
                title="New Tab"
            >
                <span class="icon-[mdi--plus] text-[18px]"></span>
            </button>
        </div>

        <!-- Drag Region Spacer - 可拖拽区域，占据剩余空间直到按钮区域 -->
        <div class="flex-1 h-[32px] min-w-[20px]" data-tauri-drag-region></div>
    </div>

    <!-- Bottom Border Line -->
    <div class="h-px bg-[#3c4043]"></div>
</template>
