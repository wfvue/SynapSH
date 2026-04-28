<!--
  TrafficLights.vue - Windows 风格窗口控制按钮组件
  方形窗口控制按钮：最小化、最大化/还原、关闭
-->
<script setup lang="ts">
defineProps<{
  isMaximized?: boolean;
  isFullscreen?: boolean;
}>();

const emit = defineEmits<{
  close: [];
  minimize: [];
  maximize: [];
}>();
</script>

<template>
  <div class="flex h-full">
    <!-- 最小化 -->
    <button
      class="w-[46px] h-full bg-transparent flex items-center justify-center transition-colors duration-100 hover:bg-white/10 active:bg-white/20 border-none outline-none focus:outline-none"
      @click.stop="emit('minimize')"
      title="最小化"
    >
      <span class="icon-[mdi--minus] text-[16px] text-foreground/80 opacity-80"></span>
    </button>
    <!-- 最大化/还原 -->
    <button
      class="w-[46px] h-full bg-transparent flex items-center justify-center transition-colors duration-100 hover:bg-white/10 active:bg-white/20 border-none outline-none focus:outline-none"
      @click.stop="emit('maximize')"
      :title="isMaximized ? '还原向下' : '最大化'"
    >
      <span
        v-if="isMaximized"
        class="icon-[mdi--window-restore] text-[14px] text-foreground/80 opacity-80"
      ></span>
      <span
        v-else
        class="icon-[mdi--checkbox-blank-outline] text-[14px] text-foreground/80 opacity-80"
      ></span>
    </button>
    <!-- 关闭 -->
    <button
      class="w-[46px] h-full bg-transparent flex items-center justify-center transition-colors duration-100 hover:bg-[#c42b1c] hover:text-white group rounded-tr-lg border-none outline-none focus:outline-none"
      :class="{ 'rounded-none': isMaximized }"
      @click.stop="emit('close')"
      title="关闭"
    >
      <span
        class="icon-[mdi--close] text-[16px] text-foreground/80 opacity-80 group-hover:text-white group-hover:opacity-100"
      ></span>
    </button>
  </div>
</template>
