<!--
  DesktopWallpaper.vue - 壁纸背景组件
  动态渐变和光效，响应外观设置变化
  支持自定义上传壁纸
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { useLocalStorage } from "@vueuse/core";

// 壁纸类型
interface Wallpaper {
    id: number;
    name: string;
    preview: string;
    type: "gradient" | "image";
    url?: string;
}

// 预设壁纸配置
const presetWallpapers = [
    {
        id: 0,
        name: "默认渐变",
        gradient: "linear-gradient(140deg, #111827, #1f2937 45%, #2b1f3b 90%)",
        glows: [
            { color: "rgba(94, 234, 212, 0.6)", top: "-60px", left: "10%" },
            { color: "rgba(125, 211, 252, 0.5)", right: "12%", top: "22%" },
            { color: "rgba(251, 191, 36, 0.35)", bottom: "-120px", right: "18%" },
        ],
    },
    {
        id: 1,
        name: "深空蓝",
        gradient: "linear-gradient(140deg, #0c1445, #1a237e 45%, #0d47a1 90%)",
        glows: [
            { color: "rgba(59, 130, 246, 0.6)", top: "-40px", left: "15%" },
            { color: "rgba(96, 165, 250, 0.5)", right: "10%", top: "30%" },
            { color: "rgba(147, 197, 253, 0.4)", bottom: "-100px", right: "25%" },
        ],
    },
    {
        id: 2,
        name: "暗夜紫",
        gradient: "linear-gradient(140deg, #1a0533, #4a1259 45%, #7c3aed 90%)",
        glows: [
            { color: "rgba(139, 92, 246, 0.6)", top: "-50px", left: "20%" },
            { color: "rgba(167, 139, 250, 0.5)", right: "15%", top: "25%" },
            { color: "rgba(196, 181, 253, 0.4)", bottom: "-80px", left: "30%" },
        ],
    },
    {
        id: 3,
        name: "极光绿",
        gradient: "linear-gradient(140deg, #0d2818, #1b4332 45%, #2d6a4f 90%)",
        glows: [
            { color: "rgba(52, 211, 153, 0.6)", top: "-60px", left: "10%" },
            { color: "rgba(110, 231, 183, 0.5)", right: "20%", top: "20%" },
            { color: "rgba(167, 243, 208, 0.4)", bottom: "-100px", right: "15%" },
        ],
    },
];

// 从本地存储读取当前壁纸ID和自定义壁纸
const wallpaperId = useLocalStorage("appearance-wallpaper", 0);
const customWallpapers = useLocalStorage<string[]>("appearance-custom-wallpapers", []);

// 当前壁纸类型和样式
const isCustomWallpaper = computed(() => wallpaperId.value >= 100);
const customWallpaperUrl = computed(() => {
    if (!isCustomWallpaper.value) return null;
    const index = wallpaperId.value - 100;
    return customWallpapers.value[index] || null;
});

// 当前预设壁纸配置
const currentPresetWallpaper = computed(() => {
    if (isCustomWallpaper.value) return null;
    return presetWallpapers.find((w) => w.id === wallpaperId.value) || presetWallpapers[0];
});

// 监听壁纸变化事件
function handleWallpaperChange(event: CustomEvent<{ id: number; wallpaper?: Wallpaper }>) {
    wallpaperId.value = event.detail.id;
}

onMounted(() => {
    window.addEventListener("wallpaper-change", handleWallpaperChange as EventListener);
});

onUnmounted(() => {
    window.removeEventListener("wallpaper-change", handleWallpaperChange as EventListener);
});
</script>

<template>
    <div class="wallpaper">
        <!-- 自定义图片壁纸 -->
        <div v-if="isCustomWallpaper && customWallpaperUrl" class="wallpaper-image"
            :style="{ backgroundImage: `url(${customWallpaperUrl})` }"></div>

        <!-- 预设渐变壁纸 -->
        <template v-else-if="currentPresetWallpaper">
            <div class="wallpaper-gradient" :style="{ background: currentPresetWallpaper.gradient }"></div>
            <span v-for="(glow, index) in currentPresetWallpaper.glows" :key="index" class="glow" :style="{
                background: `radial-gradient(circle, ${glow.color}, transparent 70%)`,
                top: glow.top,
                left: glow.left,
                right: glow.right,
                bottom: glow.bottom,
            }"></span>
            <span class="wave wave-a"></span>
            <span class="wave wave-b"></span>
        </template>
    </div>
</template>

<style scoped>
.wallpaper {
    position: absolute;
    inset: 0;
    z-index: 0;
    overflow: hidden;
}

.wallpaper-gradient {
    position: absolute;
    inset: 0;
    transition: background 0.5s ease;
}

.wallpaper-image {
    position: absolute;
    inset: 0;
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
    transition: background-image 0.5s ease;
}

.glow {
    position: absolute;
    width: 380px;
    height: 380px;
    border-radius: 50%;
    filter: blur(60px);
    opacity: 0.7;
    transition: all 0.5s ease;
}

.wave {
    position: absolute;
    width: 120%;
    height: 240px;
    left: -10%;
    border-radius: 999px;
    opacity: 0.22;
    filter: blur(20px);
}

.wave-a {
    bottom: 32%;
    background: linear-gradient(90deg, rgba(125, 211, 252, 0.6), rgba(94, 234, 212, 0.2));
}

.wave-b {
    bottom: 18%;
    background: linear-gradient(90deg, rgba(59, 130, 246, 0.2), rgba(251, 191, 36, 0.5));
}
</style>
