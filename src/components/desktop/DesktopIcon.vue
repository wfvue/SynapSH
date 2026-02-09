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
        class="flex flex-col items-center gap-1.5 p-2 rounded-lg transition-colors w-24 hover:bg-white/20 select-none cursor-pointer group bg-transparent border-none outline-none focus:outline-none"
        :class="{ 'bg-white/10': selected }" @click.stop="handleClick" @dblclick.stop="handleDblClick">

        <AppIcon :icon="item.icon" :background="item.color" :size="56"
            class="group-hover:scale-105 group-hover:-translate-y-0.5 group-hover:shadow-[0_8px_24px_rgba(0,0,0,0.25),0_2px_6px_rgba(0,0,0,0.15),inset_0_1px_0_rgba(255,255,255,0.25)] transition-transform duration-200 ease-out" />

        <span
            class="text-[12px] text-white/90 text-center drop-shadow-[0_1px_3px_rgba(0,0,0,0.8)] max-w-full truncate px-1 rounded leading-tight"
            :class="{ 'bg-[#0061D8] text-white': selected }">
            {{ item.label }}
        </span>
    </button>
</template>
