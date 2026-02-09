<!--
  TrafficLights.vue - macOS 风格交通灯按钮组件
  红黄绿三色窗口控制按钮，支持悬停显示图标
-->
<script setup lang="ts">
const emit = defineEmits<{
    close: [];
    minimize: [];
    maximize: [];
}>();
</script>

<template>
    <div class="flex gap-2 group">
        <button
            class="w-4 h-4 rounded-full bg-[#ff5f56] border border-[#e0443e] flex items-center justify-center overflow-hidden transition-all hover:brightness-90 active:brightness-75"
            @click.stop="emit('close')"
        >
            <span class="icon-[mdi--close] opacity-0 group-hover:opacity-100 text-[10px] text-black/50 leading-none"></span>
        </button>
        <button
            class="w-4 h-4 rounded-full bg-[#ffbd2e] border border-[#dea123] flex items-center justify-center overflow-hidden transition-all hover:brightness-90 active:brightness-75"
            @click.stop="emit('minimize')"
        >
            <span class="icon-[mdi--minus] opacity-0 group-hover:opacity-100 text-[8px] text-black/50 leading-none"></span>
        </button>
        <button
            class="w-4 h-4 rounded-full bg-[#27c93f] border border-[#1aab29] flex items-center justify-center overflow-hidden transition-all hover:brightness-90 active:brightness-75"
            @click.stop="emit('maximize')"
        >
            <span class="icon-[mdi--square-outline] opacity-0 group-hover:opacity-100 text-[7px] text-black/50 leading-none"></span>
        </button>
    </div>
</template>
