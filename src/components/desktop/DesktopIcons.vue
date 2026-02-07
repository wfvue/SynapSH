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
    <section class="desktop-icons">
        <DesktopIcon v-for="item in items" :key="item.id" :item="item" :selected="selectedIcon === item.id"
            @select="handleSelect" @open="handleOpen" />
    </section>
</template>

<style scoped>
.desktop-icons {
    position: relative;
    z-index: 2;
    display: grid;
    grid-auto-rows: 100px;
    gap: 16px;
    padding: 28px 24px;
    width: 140px;
}
</style>
