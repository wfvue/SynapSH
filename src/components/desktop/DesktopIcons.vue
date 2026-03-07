<script setup lang="ts">
import { ref } from "vue";
import DesktopIcon, { type DesktopIconItem } from "./DesktopIcon.vue";

const props = defineProps<{
    items: DesktopIconItem[];
}>();

const emit = defineEmits<{
    openApp: [app: string];
}>();

const selectedIcon = ref<string | null>(null);

function handleSelect(id: string) {
    selectedIcon.value = id;
}

function handleOpen(app: string | undefined) {
    if (app) {
        emit("openApp", app);
    }
}

function clearSelection() {
    selectedIcon.value = null;
}

defineExpose({ clearSelection });
</script>

<template>
    <section class="relative z-20 grid grid-flow-row auto-rows-auto gap-5 p-6 w-[140px]">
        <DesktopIcon v-for="item in items" :key="item.id" :item="item" :selected="selectedIcon === item.id"
            @select="handleSelect" @open="handleOpen" />
    </section>
</template>


/* Scoped styles replaced by Tailwind CSS */
