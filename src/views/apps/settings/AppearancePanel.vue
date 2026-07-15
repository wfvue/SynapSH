<!--
  AppearancePanel.vue - 外观设置面板
  设置主题、壁纸、强调色等外观配置
  使用 shadcn-vue 组件和 VueUse 实现真实主题切换
-->
<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { useColorMode, useLocalStorage } from "@vueuse/core";
import { useAppearance } from "@/composables/useAppearance";
import { useInterfaceLanguage } from "@/composables/useInterfaceLanguage";
import { Slider } from "@/components/ui/slider";

const { dockIconSize } = useAppearance();
const { text } = useInterfaceLanguage();

const dockIconSizeArray = computed({
  get: () => [dockIconSize.value],
  set: (val) => {
    if (val && val.length > 0) {
      dockIconSize.value = val[0];
    }
  },
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
const accentColor = useLocalStorage("appearance-accent-color", "#0a84ff");
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
  { value: "#0a84ff", nameEn: "Blue", nameZh: "蓝色" },
  { value: "#8b5cf6", nameEn: "Purple", nameZh: "紫色" },
  { value: "#ec4899", nameEn: "Pink", nameZh: "粉色" },
  { value: "#ef4444", nameEn: "Red", nameZh: "红色" },
  { value: "#f97316", nameEn: "Orange", nameZh: "橙色" },
  { value: "#22c55e", nameEn: "Green", nameZh: "绿色" },
  { value: "#06b6d4", nameEn: "Cyan", nameZh: "青色" },
  { value: "#64748b", nameEn: "Gray", nameZh: "灰色" },
];

// 预设壁纸选项
const presetWallpapers = [
  {
    id: 0,
    nameEn: "Default Gradient",
    nameZh: "默认渐变",
    preview: "linear-gradient(135deg, #1a1a2e 0%, #16213e 100%)",
    type: "gradient",
  },
  {
    id: 1,
    nameEn: "Deep Space Blue",
    nameZh: "深空蓝",
    preview: "linear-gradient(135deg, #0c1445 0%, #1a237e 100%)",
    type: "gradient",
  },
  {
    id: 2,
    nameEn: "Midnight Purple",
    nameZh: "暗夜紫",
    preview: "linear-gradient(135deg, #1a0533 0%, #4a1259 100%)",
    type: "gradient",
  },
  {
    id: 3,
    nameEn: "Aurora Green",
    nameZh: "极光绿",
    preview: "linear-gradient(135deg, #0d2818 0%, #1b4332 100%)",
    type: "gradient",
  },
];

// 合并预设和自定义壁纸
const allWallpapers = computed(() => {
  const custom = customWallpapers.value.map((url, index) => ({
    id: 100 + index,
    name: text(`Custom ${index + 1}`, `自定义 ${index + 1}`),
    preview: `url(${url})`,
    type: "image" as const,
    url,
  }));
  return [
    ...presetWallpapers.map((wallpaper) => ({
      ...wallpaper,
      name: text(wallpaper.nameEn, wallpaper.nameZh),
    })),
    ...custom,
  ];
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
    }),
  );
}

// 初始化时应用设置
watch(
  accentColor,
  (color) => {
    document.documentElement.style.setProperty("--accent-color", color);
  },
  { immediate: true },
);

watch(
  wallpaperId,
  (id) => {
    const wallpaper = allWallpapers.value.find((w) => w.id === id);
    window.dispatchEvent(
      new CustomEvent("wallpaper-change", {
        detail: { id, wallpaper },
      }),
    );
  },
  { immediate: true },
);
</script>

<template>
  <div class="space-y-7 animate-in fade-in duration-300 pb-8">
    <!-- 主题模式 -->
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Theme Mode", "主题模式")
        }}</span>
      </div>
      <div
        class="grid grid-cols-3 gap-4 bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-4 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <button
          v-for="option in [
            {
              value: 'light',
              label: text('Light', '浅色'),
              icon: 'icon-[lucide--sun]',
              bg: 'bg-gradient-to-br from-white to-gray-200',
            },
            {
              value: 'dark',
              label: text('Dark', '深色'),
              icon: 'icon-[lucide--moon]',
              bg: 'bg-gradient-to-br from-gray-800 to-black',
            },
            {
              value: 'auto',
              label: text('Auto', '自动'),
              icon: 'icon-[lucide--monitor]',
              bg: 'bg-gradient-to-br from-gray-200 to-gray-800',
            },
          ]"
          :key="option.value"
          class="group relative flex flex-col items-center gap-3 p-3 rounded-2xl border transition-all duration-200 outline-none"
          :class="[
            theme === option.value
              ? 'border-accent bg-accent/5 shadow-[0_8px_20px_rgba(var(--accent-rgb),0.15)]'
              : 'border-subtle bg-surface/30 hover:bg-surface/60 hover:border-strong',
          ]"
          @click="theme = option.value as 'light' | 'dark' | 'auto'"
        >
          <div
            class="w-full aspect-[16/10] rounded-xl shadow-sm overflow-hidden border border-subtle flex items-center justify-center text-3xl"
            :class="option.bg"
          >
            <span
              :class="[option.icon, theme === option.value ? 'text-accent' : 'text-primary/70']"
              class="size-8"
            ></span>
          </div>
          <span class="text-xs font-semibold uppercase tracking-wider">{{ option.label }}</span>
        </button>
      </div>
    </section>

    <!-- 强调色 -->
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Accent Color", "强调色")
        }}</span>
      </div>
      <div
        class="flex flex-wrap gap-4 p-4 rounded-xl bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <button
          v-for="color in accentColors"
          :key="color.value"
          class="size-8 rounded-full cursor-pointer transition-all duration-200 hover:scale-125 focus:outline-none focus:ring-2 focus:ring-accent/40 focus:ring-offset-2 focus:ring-offset-bg-canvas relative flex items-center justify-center group"
          :style="{
            backgroundColor: color.value,
          }"
          :title="text(color.nameEn, color.nameZh)"
          @click="applyAccentColor(color.value)"
        >
          <div
            v-if="accentColor === color.value"
            class="size-full rounded-full border-2 border-white/40 ring-2 ring-black/10 flex items-center justify-center"
          >
            <span class="icon-[lucide--check] text-white size-4 drop-shadow-md"></span>
          </div>
        </button>
      </div>
    </section>

    <!-- 壁纸 -->
    <section>
      <div class="px-2 mb-2 flex items-center justify-between">
        <span class="text-[12px] font-semibold text-tertiary">{{
          text("Desktop Wallpaper", "桌面壁纸")
        }}</span>
      </div>

      <div
        class="bg-white dark:bg-white/5 border border-black/5 dark:border-white/5 rounded-xl p-4 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <!-- 上传 -->
        <div
          class="mb-6 border border-dashed border-black/10 dark:border-white/10 rounded-lg p-6 text-center cursor-pointer transition-all duration-200 group relative overflow-hidden"
          :class="[
            isDragging
              ? 'border-accent bg-accent/5'
              : 'hover:border-accent/40 bg-black/[0.02] dark:bg-white/[0.02]',
          ]"
          @dragover="handleDragOver"
          @dragleave="handleDragLeave"
          @drop="handleDrop"
          @click="fileInput?.click()"
        >
          <div
            class="size-12 mx-auto mb-3 rounded-2xl bg-active flex items-center justify-center text-accent transition-all duration-300 group-hover:scale-110 group-hover:rotate-6 group-hover:bg-accent group-hover:text-white shadow-sm"
          >
            <span class="icon-[lucide--upload-cloud] size-6"></span>
          </div>
          <p class="text-sm font-semibold mb-1">
            {{ text("Click or drag to upload", "点击或拖拽上传") }}
          </p>
          <p class="text-xs text-tertiary">
            {{ text("Common image formats supported (Max 5 MB)", "支持主流图像格式（最大 5 MB）") }}
          </p>
          <input
            ref="fileInput"
            type="file"
            accept="image/*"
            multiple
            hidden
            @change="handleFileSelect"
          />
        </div>

        <!-- 壁纸网格 -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div
            v-for="wp in allWallpapers"
            :key="wp.id"
            class="group relative aspect-[16/10] rounded-xl overflow-hidden cursor-pointer border-2 transition-all duration-360"
            :class="[
              wallpaperId === wp.id
                ? 'border-accent ring-4 ring-accent/15 shadow-xl'
                : 'border-subtle hover:border-accent/50 hover:shadow-lg',
            ]"
            @click="applyWallpaper(wp.id)"
          >
            <div
              class="size-full bg-cover bg-center transition-transform duration-700 group-hover:scale-110"
              :style="{
                background:
                  wp.type === 'image' ? `url(${(wp as any).url}) center/cover` : wp.preview,
              }"
            ></div>

            <div
              v-if="wallpaperId === wp.id"
              class="absolute inset-0 flex items-center justify-center bg-accent/20 backdrop-blur-[1px]"
            >
              <div
                class="size-8 rounded-full bg-accent text-white flex items-center justify-center shadow-lg border border-white/20 animate-in zoom-in-50 duration-300"
              >
                <span class="icon-[lucide--check] size-5"></span>
              </div>
            </div>

            <div
              class="absolute inset-x-0 bottom-0 p-3 bg-gradient-to-t from-black/80 to-transparent translate-y-full group-hover:translate-y-0 transition-transform duration-300"
            >
              <span
                class="text-[10px] text-white font-bold uppercase tracking-widest truncate block"
                >{{ wp.name }}</span
              >
            </div>

            <button
              v-if="wp.id >= 100"
              class="absolute top-2 right-2 size-6 rounded-lg bg-black/50 hover:bg-danger text-white flex items-center justify-center backdrop-blur-md opacity-0 group-hover:opacity-100 transition-all duration-200"
              @click.stop="removeCustomWallpaper(wp.id)"
            >
              <span class="icon-[lucide--trash-2] size-3.5"></span>
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Dock -->
    <section>
      <div class="px-2 mb-2">
        <span class="text-[12px] font-semibold text-tertiary">{{ text("Dock", "Dock 栏") }}</span>
      </div>
      <div
        class="flex items-center gap-6 bg-white dark:bg-white/5 p-4 rounded-xl border border-black/5 dark:border-white/5 shadow-[0_1px_3px_rgba(0,0,0,0.02)]"
      >
        <div
          class="size-10 rounded-lg bg-black/5 dark:bg-white/5 grid place-items-center text-primary group shadow-sm overflow-hidden relative"
        >
          <div
            class="absolute inset-0 bg-accent/10 opacity-0 group-hover:opacity-100 transition-opacity"
          ></div>
          <span
            class="icon-[lucide--grid-2x2] size-6 relative transition-transform group-hover:scale-110"
          ></span>
        </div>
        <div class="flex-1 space-y-1">
          <span class="text-sm font-semibold">{{ text("Icon size", "图标尺寸") }}</span>
          <p class="text-xs text-tertiary">
            {{
              text(
                "Adjust the base size of application icons in the Dock",
                "滑动调整下方快捷栏应用图标的基础显示大小",
              )
            }}
          </p>
        </div>

        <div class="flex items-center gap-4 w-[240px]">
          <span
            class="text-xs w-10 text-right font-bold text-accent px-2 py-0.5 bg-accent/10 rounded-md border border-accent/20"
            >{{ dockIconSize }}px</span
          >
          <Slider
            v-slot="slotProps"
            v-model="dockIconSizeArray"
            :max="96"
            :min="32"
            :step="4"
            class="flex-1"
          >
            <div class="relative w-full h-1.5 bg-active rounded-full overflow-hidden">
              <div
                class="absolute left-0 top-0 h-full bg-accent transition-all duration-300"
                :style="{ width: `${((dockIconSize - 32) / (96 - 32)) * 100}%` }"
              ></div>
            </div>
          </Slider>
        </div>
      </div>
    </section>
  </div>
</template>
