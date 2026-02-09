<!--
  AppearancePanel.vue - 外观设置面板
  设置主题、壁纸、强调色等外观配置
  使用 shadcn-vue 组件和 VueUse 实现真实主题切换
-->
<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import { Slider } from "@/components/ui/slider";

// 使用 VueUse 的 useColorMode 管理主题（与 App.vue 同步）
const mode = useColorMode({
    emitAuto: true,
    storageKey: "vueuse-color-mode",
    attribute: "class",
    modes: {
        dark: "dark",
        light: "light",
        auto: "auto",
    },
});

// 本地存储外观设置
const accentColor = useLocalStorage("appearance-accent-color", "#3b82f6");
const wallpaperId = useLocalStorage("appearance-wallpaper", 0);
const customWallpapers = useLocalStorage<string[]>("appearance-custom-wallpapers", []);

// 计算当前主题模式
const theme = computed({
    get: () => mode.value,
    set: (value) => {
        mode.value = value;
    },
});

// 强调色选项
const accentColors = [
    { value: "#3b82f6", name: "蓝色" },
    { value: "#8b5cf6", name: "紫色" },
    { value: "#ec4899", name: "粉色" },
    { value: "#ef4444", name: "红色" },
    { value: "#f97316", name: "橙色" },
    { value: "#22c55e", name: "绿色" },
    { value: "#06b6d4", name: "青色" },
];

// 预设壁纸选项
const presetWallpapers = [
    { id: 0, name: "默认渐变", preview: "linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)", type: "gradient" },
    { id: 1, name: "深空蓝", preview: "linear-gradient(135deg, #0c1445 0%, #1a237e 100%)", type: "gradient" },
    { id: 2, name: "暗夜紫", preview: "linear-gradient(135deg, #1a0533 0%, #4a1259 100%)", type: "gradient" },
    { id: 3, name: "极光绿", preview: "linear-gradient(135deg, #0d2818 0%, #1b4332 100%)", type: "gradient" },
];

// 合并预设和自定义壁纸
const allWallpapers = computed(() => {
    const custom = customWallpapers.value.map((url, index) => ({
        id: 100 + index,
        name: `自定义 ${index + 1}`,
        preview: `url(${url})`,
        type: "image" as const,
        url,
    }));
    return [...presetWallpapers, ...custom];
});

// 壁纸拖拽上传
const isDragging = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);

function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging.value = true;
}

function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    isDragging.value = false;
}

function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging.value = false;
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
        handleFiles(files);
    }
}

function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
        handleFiles(input.files);
    }
}

function handleFiles(files: FileList) {
    Array.from(files).forEach((file) => {
        if (file.type.startsWith("image/")) {
            const reader = new FileReader();
            reader.onload = (e) => {
                const dataUrl = e.target?.result as string;
                if (dataUrl) {
                    customWallpapers.value = [...customWallpapers.value, dataUrl];
                }
            };
            reader.readAsDataURL(file);
        }
    });
}

function removeCustomWallpaper(id: number) {
    const index = id - 100;
    if (index >= 0 && index < customWallpapers.value.length) {
        customWallpapers.value = customWallpapers.value.filter((_, i) => i !== index);
        // 如果删除的是当前壁纸，切换回默认
        if (wallpaperId.value === id) {
            wallpaperId.value = 0;
        }
    }
}

// 应用强调色到 CSS 变量
function applyAccentColor(color: string) {
    document.documentElement.style.setProperty("--accent-color", color);
    accentColor.value = color;
}

// 应用壁纸
function applyWallpaper(id: number) {
    wallpaperId.value = id;
    // 发送事件通知壁纸组件
    const wallpaper = allWallpapers.value.find((w) => w.id === id);
    window.dispatchEvent(
        new CustomEvent("wallpaper-change", {
            detail: { id, wallpaper },
        })
    );
}

// 初始化时应用设置
watch(
    accentColor,
    (color) => {
        document.documentElement.style.setProperty("--accent-color", color);
    },
    { immediate: true }
);

watch(
    wallpaperId,
    (id) => {
        const wallpaper = allWallpapers.value.find((w) => w.id === id);
        window.dispatchEvent(
            new CustomEvent("wallpaper-change", {
                detail: { id, wallpaper },
            })
        );
    },
    { immediate: true }
);
</script>

<template>
    <div class="settings-panel">
        <h2 class="panel-title">外观</h2>
        <p class="panel-subtitle">个性化您的桌面外观</p>

        <section class="settings-section">
            <h3 class="section-title">主题</h3>

            <div class="theme-options">
                <div class="theme-option" :class="{ active: theme === 'light' }" @click="theme = 'light'">
                    <div class="theme-preview light-preview">
                        <span class="icon-[mdi--white-balance-sunny]"></span>
                    </div>
                    <span class="theme-label">浅色</span>
                </div>
                <div class="theme-option" :class="{ active: theme === 'dark' }" @click="theme = 'dark'">
                    <div class="theme-preview dark-preview">
                        <span class="icon-[mdi--moon-waning-crescent]"></span>
                    </div>
                    <span class="theme-label">深色</span>
                </div>
                <div class="theme-option" :class="{ active: theme === 'auto' }" @click="theme = 'auto'">
                    <div class="theme-preview auto-preview">
                        <span class="icon-[mdi--theme-light-dark]"></span>
                    </div>
                    <span class="theme-label">自动</span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">强调色</h3>

            <div class="accent-colors">
                <div v-for="color in accentColors" :key="color.value" class="accent-color"
                    :class="{ active: accentColor === color.value }" :style="{ background: color.value }"
                    :title="color.name" @click="applyAccentColor(color.value)">
                    <span v-if="accentColor === color.value" class="icon-[mdi--check]"></span>
                </div>
            </div>
        </section>

        <section class="settings-section">
            <h3 class="section-title">壁纸</h3>

            <!-- 拖拽上传区域 -->
            <div class="wallpaper-upload" :class="{ dragging: isDragging }" @dragover="handleDragOver"
                @dragleave="handleDragLeave" @drop="handleDrop" @click="fileInput?.click()">
                <span class="icon-[mdi--cloud-upload-outline]"></span>
                <span>拖拽图片到此处或点击上传</span>
                <input ref="fileInput" type="file" accept="image/*" multiple hidden @change="handleFileSelect" />
            </div>

            <div class="wallpaper-grid">
                <div v-for="wp in allWallpapers" :key="wp.id" class="wallpaper-item"
                    :class="{ active: wallpaperId === wp.id }" :style="{
                        background: wp.type === 'image' ? `url(${(wp as any).url}) center/cover` : wp.preview,
                    }" @click="applyWallpaper(wp.id)">
                    <span class="wallpaper-name">{{ wp.name }}</span>
                    <button v-if="wp.id >= 100" class="wallpaper-remove" @click.stop="removeCustomWallpaper(wp.id)">
                        <span class="icon-[mdi--close]"></span>
                    </button>
                </div>
            </div>
        </section>
    </div>
</template>

<style scoped>
.settings-panel {
    padding: 24px;
    color: var(--foreground);
}

.panel-title {
    font-size: 1.5rem;
    font-weight: 600;
    margin: 0 0 4px 0;
    color: var(--foreground);
}

.panel-subtitle {
    font-size: 0.85rem;
    color: var(--muted-foreground);
    margin: 0 0 24px 0;
}

.settings-section {
    margin-bottom: 28px;
}

.section-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted-foreground);
    margin: 0 0 12px 0;
}

/* Theme Options */
.theme-options {
    display: flex;
    gap: 12px;
}

.theme-option {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    background: var(--secondary);
    border: 2px solid transparent;
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
}

.theme-option:hover {
    background: var(--accent);
}

.theme-option.active {
    border-color: var(--accent-color);
    background: var(--accent);
}

.theme-preview {
    width: 60px;
    height: 44px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    font-size: 24px;
}

.light-preview {
    background: linear-gradient(135deg, #f0f0f0 0%, #e0e0e0 100%);
    color: #f59e0b;
}

.dark-preview {
    background: linear-gradient(135deg, #1e1e2e 0%, #2d2d3d 100%);
    color: #a78bfa;
}

.auto-preview {
    background: linear-gradient(135deg, #f0f0f0 0%, #2d2d3d 100%);
    color: var(--foreground);
}

.theme-label {
    font-size: 0.85rem;
    font-weight: 500;
}

/* Accent Colors */
.accent-colors {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}

.accent-color {
    width: 36px;
    height: 36px;
    border-radius: 50%;
    cursor: pointer;
    display: grid;
    place-items: center;
    color: white;
    font-size: 16px;
    border: 3px solid transparent;
    transition: all 0.2s;
}

.accent-color:hover {
    transform: scale(1.1);
}

.accent-color.active {
    border-color: var(--foreground);
    box-shadow: 0 0 0 2px var(--background);
}

/* Wallpaper Upload */
.wallpaper-upload {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 20px;
    margin-bottom: 12px;
    border: 2px dashed var(--border);
    border-radius: 12px;
    cursor: pointer;
    transition: all 0.2s;
    color: var(--muted-foreground);
    background: var(--secondary);
}

.wallpaper-upload:hover,
.wallpaper-upload.dragging {
    border-color: var(--accent-color);
    background: var(--accent);
    color: var(--foreground);
}

.wallpaper-upload span:first-child {
    font-size: 32px;
}

/* Wallpaper Grid */
.wallpaper-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
}

.wallpaper-item {
    position: relative;
    height: 80px;
    border-radius: 10px;
    cursor: pointer;
    display: flex;
    align-items: flex-end;
    padding: 8px 12px;
    border: 2px solid transparent;
    transition: all 0.2s;
}

.wallpaper-item:hover {
    transform: scale(1.02);
}

.wallpaper-item.active {
    border-color: var(--accent-color);
}

.wallpaper-name {
    font-size: 0.75rem;
    font-weight: 500;
    color: rgba(255, 255, 255, 0.8);
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
}

.wallpaper-remove {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: none;
    background: rgba(0, 0, 0, 0.6);
    color: white;
    cursor: pointer;
    display: grid;
    place-items: center;
    font-size: 12px;
    opacity: 0;
    transition: opacity 0.2s;
}

.wallpaper-item:hover .wallpaper-remove {
    opacity: 1;
}

.wallpaper-remove:hover {
    background: rgba(239, 68, 68, 0.8);
}

/* Setting Item */
.setting-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    background: var(--secondary);
    border-radius: 10px;
}

.setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.setting-label {
    font-size: 0.9rem;
    font-weight: 500;
}

.setting-desc {
    font-size: 0.75rem;
    color: var(--muted-foreground);
}

/* Slider */
.slider-container {
    display: flex;
    align-items: center;
    gap: 12px;
}

.slider-value {
    font-size: 0.85rem;
    color: var(--muted-foreground);
    min-width: 40px;
    text-align: right;
}
</style>
