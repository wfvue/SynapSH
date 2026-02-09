<!--
  AppearancePanel.vue - 外观设置面板
  设置主题、壁纸、强调色等外观配置
  使用 shadcn-vue 组件和 VueUse 实现真实主题切换
-->
<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import { useAppearance } from "@/composables/useAppearance";
import { Slider } from "@/components/ui/slider";

const { dockIconSize } = useAppearance();

const dockIconSizeArray = computed({
    get: () => [dockIconSize.value],
    set: (val) => {
        if (val && val.length > 0) {
            dockIconSize.value = val[0];
        }
    }
});

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
    { value: "#64748b", name: "灰色" },
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
    <div class="p-8 max-w-5xl mx-auto text-foreground animate-in fade-in duration-500">
        <div class="mb-8">
            <h2 class="text-3xl font-bold tracking-tight mb-2">外观</h2>
            <p class="text-muted-foreground">个性化您的桌面外观，打造专属工作空间</p>
        </div>

        <!-- 主题设置 -->
        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">主题模式</h3>
            <div class="grid grid-cols-3 gap-4">
                <button v-for="option in [
                    { value: 'light', label: '浅色', icon: 'icon-[mdi--white-balance-sunny]', bg: 'bg-gradient-to-br from-white to-gray-100' },
                    { value: 'dark', label: '深色', icon: 'icon-[mdi--moon-waning-crescent]', bg: 'bg-gradient-to-br from-gray-800 to-gray-900' },
                    { value: 'auto', label: '自动', icon: 'icon-[mdi--theme-light-dark]', bg: 'bg-gradient-to-br from-gray-200 to-gray-800' }
                ]" :key="option.value"
                    class="group relative flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 outline-none"
                    :class="[
                        theme === option.value
                            ? 'border-primary bg-primary/5 ring-2 ring-primary/20'
                            : 'border-transparent bg-secondary/20 hover:bg-secondary/40 hover:border-border/50'
                    ]" @click="theme = option.value as 'light' | 'dark' | 'auto'">
                    <div class="w-full aspect-video rounded-lg shadow-sm overflow-hidden border border-border flex items-center justify-center text-4xl"
                        :class="option.bg">
                        <span
                            :class="[option.icon, theme === option.value ? 'text-primary' : 'text-foreground/70']"></span>
                    </div>
                    <span class="text-sm font-medium">{{ option.label }}</span>
                </button>
            </div>
        </section>

        <!-- 强调色设置 -->
        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">强调色</h3>
            <div class="flex flex-wrap gap-4 p-4 rounded-xl bg-secondary/20 border border-border">
                <button v-for="color in accentColors" :key="color.value"
                    class="w-10 h-10 rounded-full cursor-pointer transition-all duration-200 hover:scale-110 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-background relative flex items-center justify-center"
                    :style="{ backgroundColor: color.value, boxShadow: accentColor === color.value ? `0 0 0 2px var(--background), 0 0 0 4px ${color.value}` : 'none' }"
                    :title="color.name" @click="applyAccentColor(color.value)">
                    <span v-if="accentColor === color.value"
                        class="icon-[mdi--check] text-white text-lg drop-shadow-md"></span>
                </button>
            </div>
        </section>

        <!-- 壁纸设置 -->
        <section class="mb-10">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">桌面壁纸</h3>

            <!-- 上传区域 -->
            <div class="mb-6 border-2 border-dashed rounded-xl p-8 text-center cursor-pointer transition-all duration-200 group"
                :class="[
                    isDragging
                        ? 'border-primary bg-primary/5 scale-[1.01]'
                        : 'border-border hover:border-primary/50 hover:bg-secondary/20'
                ]" @dragover="handleDragOver" @dragleave="handleDragLeave" @drop="handleDrop"
                @click="fileInput?.click()">
                <div
                    class="w-12 h-12 mx-auto mb-3 rounded-full bg-primary/10 flex items-center justify-center text-primary transition-transform duration-300 group-hover:scale-110 group-hover:rotate-6">
                    <span class="icon-[mdi--cloud-upload-outline] text-2xl"></span>
                </div>
                <p class="text-sm font-medium mb-1">点击或拖拽上传图片</p>
                <p class="text-xs text-muted-foreground">支持 JPG, PNG, WEBP 格式</p>
                <input ref="fileInput" type="file" accept="image/*" multiple hidden @change="handleFileSelect" />
            </div>

            <!-- 壁纸列表 -->
            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
                <div v-for="wp in allWallpapers" :key="wp.id"
                    class="group relative aspect-video rounded-lg overflow-hidden cursor-pointer border-2 transition-all duration-200"
                    :class="[
                        wallpaperId === wp.id
                            ? 'border-primary ring-2 ring-primary/20 shadow-lg shadow-primary/10'
                            : 'border-transparent hover:border-border hover:shadow-lg'
                    ]" @click="applyWallpaper(wp.id)">
                    <div class="w-full h-full bg-cover bg-center transition-transform duration-500 group-hover:scale-110"
                        :style="{
                            background: wp.type === 'image' ? `url(${(wp as any).url}) center/cover` : wp.preview,
                        }"></div>

                    <!-- 选中标记 -->
                    <div v-if="wallpaperId === wp.id"
                        class="absolute inset-0 flex items-center justify-center bg-black/20 backdrop-blur-[1px]">
                        <div
                            class="w-8 h-8 rounded-full bg-primary text-white flex items-center justify-center shadow-lg">
                            <span class="icon-[mdi--check]"></span>
                        </div>
                    </div>

                    <!-- 名称遮罩 -->
                    <div class="absolute inset-x-0 bottom-0 p-2 bg-gradient-to-t from-black/80 to-transparent">
                        <span class="text-xs text-white font-medium truncate block shadow-sm">{{ wp.name }}</span>
                    </div>

                    <!-- 删除按钮 -->
                    <button v-if="wp.id >= 100"
                        class="absolute top-2 right-2 w-6 h-6 rounded-full bg-black/50 hover:bg-destructive text-white flex items-center justify-center backdrop-blur-md opacity-0 group-hover:opacity-100 transition-all duration-200 transform scale-90 group-hover:scale-100"
                        @click.stop="removeCustomWallpaper(wp.id)" title="删除壁纸">
                        <span class="icon-[mdi--close] text-xs"></span>
                    </button>
                </div>
            </div>
        </section>

        <!-- Dock 栏设置 -->
        <section class="mb-10 text-foreground">
            <h3 class="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-4">Dock 栏设置</h3>
            <div class="flex items-center gap-4 bg-secondary/20 p-4 rounded-xl border border-border">
                <div class="flex-1 space-y-1">
                    <span class="text-sm font-medium">图标大小</span>
                    <p class="text-xs text-muted-foreground">调整 Dock 栏图标的显示尺寸</p>
                </div>

                <div class="flex items-center gap-3 w-[200px]">
                    <span class="text-xs w-8 text-right font-mono">{{ dockIconSize }}px</span>
                    <Slider v-model="dockIconSizeArray" :max="96" :min="32" :step="4" class="flex-1" />
                </div>
            </div>
        </section>
    </div>
</template>
