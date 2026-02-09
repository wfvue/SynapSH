<script setup lang="ts">
import { useAppearance } from '../../composables/useAppearance';
import AppIcon from './AppIcon.vue';

export interface DockItem {
    id: string;
    label: string;
    icon: string;
    color: string;
    app?: string;
}

defineProps<{
    items: DockItem[];
    openApps: string[];
}>();

const emit = defineEmits<{
    openApp: [app: string];
}>();

const { dockIconSize } = useAppearance();

function handleClick(app: string | undefined) {
    if (app) {
        emit("openApp", app);
    }
}
</script>

<template>
    <section
        class="absolute bottom-2 left-1/2 -translate-x-1/2 flex items-end gap-2 px-3 py-2 rounded-2xl bg-black/40 border border-white/10 backdrop-blur-3xl z-40 transition-all duration-300">
        <button v-for="item in items" :key="item.id"
            class="relative flex items-center justify-center transition-all duration-200 hover:-translate-y-2 hover:scale-110 active:translate-y-[-2px] border-none bg-transparent cursor-pointer group origin-bottom"
            :class="{ '-translate-y-0.5': item.app && openApps.includes(item.app) }" :title="item.label"
            :style="{ width: `${dockIconSize}px`, height: `${dockIconSize}px` }" @click.stop="handleClick(item.app)">

            <AppIcon :icon="item.icon" :background="item.color" :size="dockIconSize"
                class="shadow-md group-hover:brightness-110" />

            <span class="absolute -bottom-2 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-white/80 shadow-sm"
                v-if="item.app && openApps.includes(item.app)"></span>
        </button>
    </section>
</template>
