<script setup lang="ts">
import AppIcon from './AppIcon.vue';

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
    <button
        class="flex flex-col items-center gap-1 p-2 rounded transition-colors w-24 hover:bg-white/10 select-none cursor-pointer group bg-transparent border border-transparent outline-none focus:outline-none"
        :class="{ 'bg-blue-500/30 border-blue-500/40 hover:bg-blue-500/40': selected }" @click.stop="handleClick"
        @dblclick.stop="handleDblClick">

        <AppIcon :icon="item.icon" :background="item.color" :size="48"
            class="transition-transform duration-200 ease-out mb-1" />

        <span class="text-[12px] text-white text-center drop-shadow-md max-w-full truncate px-1 rounded leading-tight">
            {{ item.label }}
        </span>
    </button>
</template>
