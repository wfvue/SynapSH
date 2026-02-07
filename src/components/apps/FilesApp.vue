<script setup lang="ts">
import { ref, computed, nextTick } from "vue";

// 文件类型定义
type FileType = "folder" | "text" | "image" | "code" | "archive" | "executable" | "unknown";

interface FileItem {
  id: string;
  name: string;
  type: FileType;
  size: number;
  modified: Date;
  isSelected?: boolean;
}

interface Tab {
  id: string;
  path: string;
  title: string;
}

// 视图模式
const viewMode = ref<"list" | "grid">("list");

// 标签页
const tabs = ref<Tab[]>([
  { id: "1", path: "/home/user", title: "主目录" },
]);
const activeTabId = ref("1");

// 当前路径
const currentPath = ref("/home/user");

// 导航历史
const navigationHistory = ref<string[]>(["/home/user"]);
const currentHistoryIndex = ref(0);

// 侧边栏收藏
const favorites = ref([
  { id: "recent", name: "最近使用", icon: "icon-[mdi--clock-outline]" },
  { id: "apps", name: "应用程序", icon: "icon-[mdi--apps]" },
  { id: "desktop", name: "桌面", icon: "icon-[mdi--desktop-mac]" },
  { id: "documents", name: "文稿", icon: "icon-[mdi--file-document-outline]" },
  { id: "downloads", name: "下载", icon: "icon-[mdi--download]" },
  { id: "pictures", name: "图片", icon: "icon-[mdi--image]" },
  { id: "music", name: "音乐", icon: "icon-[mdi--music]" },
  { id: "movies", name: "视频", icon: "icon-[mdi--movie]" },
]);

// 文件列表
const files = ref<FileItem[]>([
  { id: "1", name: "Applications", type: "folder", size: 0, modified: new Date("2026-02-07 10:00") },
  { id: "2", name: "Desktop", type: "folder", size: 0, modified: new Date("2026-02-07 09:30") },
  { id: "3", name: "Documents", type: "folder", size: 0, modified: new Date("2026-02-06 15:20") },
  { id: "4", name: "Downloads", type: "folder", size: 0, modified: new Date("2026-02-07 18:10") },
  { id: "5", name: "Pictures", type: "folder", size: 0, modified: new Date("2026-02-05 12:00") },
  { id: "6", name: "config.yaml", type: "code", size: 2048, modified: new Date("2026-02-07 16:45") },
  { id: "7", name: "deploy.sh", type: "executable", size: 12500, modified: new Date("2026-02-06 23:12") },
  { id: "8", name: "report.log", type: "text", size: 4200000, modified: new Date("2026-02-06 21:05") },
  { id: "9", name: "data.zip", type: "archive", size: 15400000, modified: new Date("2026-02-05 14:30") },
  { id: "10", name: "avatar.png", type: "image", size: 256000, modified: new Date("2026-02-04 10:15") },
]);

// 预览面板
const previewFile = ref<FileItem | null>(null);
const isPreviewOpen = ref(false);

// 上下文菜单
const contextMenu = ref({
  show: false,
  x: 0,
  y: 0,
  fileId: null as string | null,
});

// 重命名状态
const renamingFileId = ref<string | null>(null);
const renamingValue = ref("");
const renameInputRef = ref<HTMLInputElement | null>(null);

// 搜索
const searchQuery = ref("");
const isSearching = ref(false);

// 选中文件
const selectedFiles = computed(() => files.value.filter(f => f.isSelected));

// 格式化文件大小
function formatSize(bytes: number): string {
  if (bytes === 0) return "--";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let unitIndex = 0;
  let size = bytes;
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  return `${size.toFixed(size < 10 ? 1 : 0)} ${units[unitIndex]}`;
}

// 格式化日期
function formatDate(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  
  if (days === 0) {
    return `今天 ${date.getHours().toString().padStart(2, "0")}:${date.getMinutes().toString().padStart(2, "0")}`;
  } else if (days === 1) {
    return `昨天 ${date.getHours().toString().padStart(2, "0")}:${date.getMinutes().toString().padStart(2, "0")}`;
  } else {
    return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, "0")}-${date.getDate().toString().padStart(2, "0")}`;
  }
}

// 获取文件图标
function getFileIcon(type: FileType): string {
  const icons: Record<FileType, string> = {
    folder: "icon-[mdi--folder]",
    text: "icon-[mdi--file-document-outline]",
    image: "icon-[mdi--image]",
    code: "icon-[mdi--code-json]",
    archive: "icon-[mdi--zip-box]",
    executable: "icon-[mdi--console]",
    unknown: "icon-[mdi--file-outline]",
  };
  return icons[type] || icons.unknown;
}

// 获取文件颜色
function getFileColor(type: FileType): string {
  const colors: Record<FileType, string> = {
    folder: "#60a5fa",
    text: "#9ca3af",
    image: "#a78bfa",
    code: "#34d399",
    archive: "#fbbf24",
    executable: "#f87171",
    unknown: "#9ca3af",
  };
  return colors[type] || colors.unknown;
}

// 导航
function navigateToPath(path: string) {
  currentPath.value = path;
  const tab = tabs.value.find(t => t.id === activeTabId.value);
  if (tab) {
    tab.path = path;
    tab.title = path.split("/").pop() || "主目录";
  }
  if (currentHistoryIndex.value < navigationHistory.value.length - 1) {
    navigationHistory.value = navigationHistory.value.slice(0, currentHistoryIndex.value + 1);
  }
  navigationHistory.value.push(path);
  currentHistoryIndex.value++;
}

function goBack() {
  if (currentHistoryIndex.value > 0) {
    currentHistoryIndex.value--;
    currentPath.value = navigationHistory.value[currentHistoryIndex.value];
  }
}

function goForward() {
  if (currentHistoryIndex.value < navigationHistory.value.length - 1) {
    currentHistoryIndex.value++;
    currentPath.value = navigationHistory.value[currentHistoryIndex.value];
  }
}

function goUp() {
  const parts = currentPath.value.split("/").filter(Boolean);
  if (parts.length > 0) {
    parts.pop();
    navigateToPath("/" + parts.join("/"));
  }
}

// 标签页操作
function addTab() {
  const newId = (tabs.value.length + 1).toString();
  tabs.value.push({ id: newId, path: currentPath.value, title: "新标签页" });
  activeTabId.value = newId;
}

function closeTab(tabId: string) {
  if (tabs.value.length === 1) return;
  const index = tabs.value.findIndex(t => t.id === tabId);
  tabs.value = tabs.value.filter(t => t.id !== tabId);
  if (activeTabId.value === tabId) {
    const newIndex = Math.min(index, tabs.value.length - 1);
    activeTabId.value = tabs.value[newIndex].id;
  }
}

function switchTab(tabId: string) {
  activeTabId.value = tabId;
  const tab = tabs.value.find(t => t.id === tabId);
  if (tab) {
    currentPath.value = tab.path;
  }
}

// 文件操作
function selectFile(file: FileItem, event?: MouseEvent) {
  if (event?.ctrlKey || event?.metaKey) {
    file.isSelected = !file.isSelected;
  } else if (event?.shiftKey && selectedFiles.value.length > 0) {
    const lastSelected = files.value.findIndex(f => f.isSelected);
    const currentIndex = files.value.findIndex(f => f.id === file.id);
    const start = Math.min(lastSelected, currentIndex);
    const end = Math.max(lastSelected, currentIndex);
    files.value.forEach((f, i) => {
      f.isSelected = i >= start && i <= end;
    });
  } else {
    files.value.forEach(f => f.isSelected = f.id === file.id);
  }
}

function openFile(file: FileItem) {
  if (file.type === "folder") {
    navigateToPath(`${currentPath.value}/${file.name}`);
  } else {
    previewFile.value = file;
    isPreviewOpen.value = true;
  }
}

async function startRename(file: FileItem) {
  renamingFileId.value = file.id;
  renamingValue.value = file.name;
  contextMenu.value.show = false;
  await nextTick();
  renameInputRef.value?.focus();
  renameInputRef.value?.select();
}

function confirmRename() {
  if (renamingFileId.value && renamingValue.value.trim()) {
    const file = files.value.find(f => f.id === renamingFileId.value);
    if (file) {
      file.name = renamingValue.value.trim();
    }
  }
  renamingFileId.value = null;
  renamingValue.value = "";
}

function cancelRename() {
  renamingFileId.value = null;
  renamingValue.value = "";
}

function deleteFile(fileId: string) {
  files.value = files.value.filter(f => f.id !== fileId);
  contextMenu.value.show = false;
}

function createNewFolder() {
  const newId = Date.now().toString();
  files.value.unshift({
    id: newId,
    name: "新建文件夹",
    type: "folder",
    size: 0,
    modified: new Date(),
    isSelected: true,
  });
  startRename(files.value[0]);
}

// 上下文菜单
function showContextMenu(event: MouseEvent, file?: FileItem) {
  event.preventDefault();
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    fileId: file?.id || null,
  };
}

function hideContextMenu() {
  contextMenu.value.show = false;
}

// 键盘快捷键
function handleKeydown(event: KeyboardEvent) {
  if (renamingFileId.value) {
    if (event.key === "Enter") confirmRename();
    if (event.key === "Escape") cancelRename();
    return;
  }
  
  if (event.key === "Delete" && selectedFiles.value.length > 0) {
    files.value = files.value.filter(f => !f.isSelected);
  }
  if (event.key === "Enter" && selectedFiles.value.length === 1) {
    openFile(selectedFiles.value[0]);
  }
}
</script>

<template>
  <div 
    class="h-full flex flex-col bg-neutral-900/85 backdrop-blur-xl text-neutral-200 relative overflow-hidden"
    @click="hideContextMenu"
    @keydown="handleKeydown"
    tabindex="0"
  >
    <!-- 标签页栏 -->
    <div class="bg-neutral-800/60 border-b border-white/[0.06] pt-2 px-3 flex items-end">
      <div class="flex gap-1 flex-1 overflow-x-auto">
        <div
          v-for="tab in tabs"
          :key="tab.id"
          class="flex items-center gap-1.5 px-3.5 py-2 bg-neutral-700/40 rounded-t-lg cursor-pointer transition-all min-w-[120px] max-w-[200px] border border-transparent border-b-0"
          :class="activeTabId === tab.id ? 'bg-neutral-700/90 border-white/10' : 'hover:bg-neutral-700/50'"
          @click="switchTab(tab.id)"
        >
          <span class="icon-[mdi--folder-outline] text-sm text-blue-400"></span>
          <span class="flex-1 text-[13px] truncate">{{ tab.title }}</span>
          <button
            v-if="tabs.length > 1"
            class="w-4 h-4 rounded flex items-center justify-center text-xs text-neutral-500 opacity-0 hover:opacity-100 hover:bg-red-500/30 hover:text-white transition-all"
            :class="{ 'opacity-100': activeTabId === tab.id }"
            @click.stop="closeTab(tab.id)"
          >
            <span class="icon-[mdi--close]"></span>
          </button>
        </div>
        <button 
          class="w-7 h-7 rounded-md flex items-center justify-center text-neutral-500 hover:bg-white/10 hover:text-white transition-all mb-1"
          @click="addTab"
        >
          <span class="icon-[mdi--plus]"></span>
        </button>
      </div>
    </div>

    <!-- 工具栏 -->
    <div class="flex items-center gap-3 px-4 py-2.5 bg-neutral-800/50 border-b border-white/[0.06]">
      <!-- 导航按钮 -->
      <div class="flex gap-1.5">
        <button
          class="w-7 h-7 rounded-lg bg-white/[0.06] text-neutral-300 flex items-center justify-center text-lg transition-all disabled:opacity-30 disabled:cursor-not-allowed hover:bg-white/[0.12] hover:text-white"
          :disabled="currentHistoryIndex === 0"
          @click="goBack"
        >
          <span class="icon-[mdi--chevron-left]"></span>
        </button>
        <button
          class="w-7 h-7 rounded-lg bg-white/[0.06] text-neutral-300 flex items-center justify-center text-lg transition-all disabled:opacity-30 disabled:cursor-not-allowed hover:bg-white/[0.12] hover:text-white"
          :disabled="currentHistoryIndex === navigationHistory.length - 1"
          @click="goForward"
        >
          <span class="icon-[mdi--chevron-right]"></span>
        </button>
        <button 
          class="w-7 h-7 rounded-lg bg-white/[0.06] text-neutral-300 flex items-center justify-center text-lg transition-all hover:bg-white/[0.12] hover:text-white"
          @click="goUp"
        >
          <span class="icon-[mdi--arrow-up]"></span>
        </button>
      </div>

      <!-- 路径栏 -->
      <div class="flex-1 flex items-center">
        <div class="flex items-center gap-1 bg-white/5 px-3 py-1.5 rounded-lg text-[13px]">
          <button class="bg-transparent border-none text-neutral-400 hover:text-white hover:bg-white/10 px-1.5 py-0.5 rounded transition-all">电脑</button>
          <span class="icon-[mdi--chevron-right] text-sm text-neutral-600"></span>
          <button class="bg-transparent border-none text-neutral-400 hover:text-white hover:bg-white/10 px-1.5 py-0.5 rounded transition-all">Mac OS</button>
          <span class="icon-[mdi--chevron-right] text-sm text-neutral-600"></span>
          <span class="text-white font-medium">{{ currentPath.split('/').pop() || '主目录' }}</span>
        </div>
      </div>

      <!-- 工具按钮 -->
      <div class="flex items-center gap-2.5">
        <!-- 搜索框 -->
        <div 
          class="flex items-center gap-2 bg-white/[0.06] border border-transparent rounded-lg px-3 py-1.5 transition-all"
          :class="isSearching ? 'bg-white/10 border-sky-400/40 w-60' : 'w-44'"
        >
          <span class="icon-[mdi--magnify] text-base text-neutral-500"></span>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索"
            class="bg-transparent border-none text-white text-[13px] outline-none w-full"
            @focus="isSearching = true"
            @blur="isSearching = false"
          />
          <button 
            v-if="searchQuery" 
            class="bg-transparent border-none text-neutral-500 text-sm"
            @click="searchQuery = ''"
          >
            <span class="icon-[mdi--close-circle]"></span>
          </button>
        </div>

        <!-- 视图切换 -->
        <div class="flex bg-white/[0.06] rounded-lg p-0.5 gap-0.5">
          <button
            class="w-7 h-7 rounded-md flex items-center justify-center text-neutral-400 transition-all hover:text-neutral-300"
            :class="viewMode === 'list' ? 'bg-white/[0.12] text-white' : ''"
            @click="viewMode = 'list'"
          >
            <span class="icon-[mdi--view-list]"></span>
          </button>
          <button
            class="w-7 h-7 rounded-md flex items-center justify-center text-neutral-400 transition-all hover:text-neutral-300"
            :class="viewMode === 'grid' ? 'bg-white/[0.12] text-white' : ''"
            @click="viewMode = 'grid'"
          >
            <span class="icon-[mdi--view-grid]"></span>
          </button>
        </div>

        <button 
          class="w-8 h-8 rounded-lg bg-white/[0.06] text-neutral-300 flex items-center justify-center text-lg transition-all hover:bg-white/[0.12] hover:text-white"
          @click="createNewFolder"
        >
          <span class="icon-[mdi--folder-plus]"></span>
        </button>

        <button 
          class="w-8 h-8 rounded-lg bg-white/[0.06] text-neutral-300 flex items-center justify-center text-lg transition-all hover:bg-white/[0.12] hover:text-white"
          @click="showContextMenu"
        >
          <span class="icon-[mdi--dots-vertical]"></span>
        </button>
      </div>
    </div>

    <!-- 主内容区 -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 侧边栏 -->
      <aside class="w-[200px] bg-neutral-800/50 border-r border-white/[0.06] py-4 px-3 overflow-y-auto">
        <div class="mb-5">
          <h3 class="text-[11px] font-semibold uppercase tracking-wider text-neutral-500 mb-2 px-2">收藏</h3>
          <nav class="flex flex-col gap-0.5">
            <button
              v-for="item in favorites"
              :key="item.id"
              class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg bg-transparent border-none text-neutral-300 text-[13px] cursor-pointer transition-all text-left hover:bg-white/[0.08] hover:text-white"
            >
              <span :class="item.icon" class="text-lg w-5 text-center"></span>
              <span>{{ item.name }}</span>
            </button>
          </nav>
        </div>

        <div class="mb-5">
          <h3 class="text-[11px] font-semibold uppercase tracking-wider text-neutral-500 mb-2 px-2">位置</h3>
          <nav class="flex flex-col gap-0.5">
            <button class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg bg-transparent border-none text-neutral-300 text-[13px] cursor-pointer transition-all text-left hover:bg-white/[0.08] hover:text-white">
              <span class="icon-[mdi--harddisk] text-lg w-5 text-center"></span>
              <span>Macintosh HD</span>
            </button>
            <button class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg bg-transparent border-none text-neutral-300 text-[13px] cursor-pointer transition-all text-left hover:bg-white/[0.08] hover:text-white">
              <span class="icon-[mdi--server-network] text-lg w-5 text-center"></span>
              <span>网络</span>
            </button>
          </nav>
        </div>

        <div class="mb-5">
          <h3 class="text-[11px] font-semibold uppercase tracking-wider text-neutral-500 mb-2 px-2">标签</h3>
          <nav class="flex flex-col gap-0.5">
            <button class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg bg-transparent border-none text-neutral-300 text-[13px] cursor-pointer transition-all text-left hover:bg-white/[0.08] hover:text-white">
              <span class="w-2.5 h-2.5 rounded-full bg-red-400"></span>
              <span>重要</span>
            </button>
            <button class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg bg-transparent border-none text-neutral-300 text-[13px] cursor-pointer transition-all text-left hover:bg-white/[0.08] hover:text-white">
              <span class="w-2.5 h-2.5 rounded-full bg-teal-400"></span>
              <span>工作</span>
            </button>
            <button class="flex items-center gap-2.5 px-2.5 py-2 rounded-lg bg-transparent border-none text-neutral-300 text-[13px] cursor-pointer transition-all text-left hover:bg-white/[0.08] hover:text-white">
              <span class="w-2.5 h-2.5 rounded-full bg-emerald-300"></span>
              <span>个人</span>
            </button>
          </nav>
        </div>
      </aside>

      <!-- 文件区域 -->
      <section class="flex-1 overflow-auto p-4" @contextmenu="showContextMenu($event)">
        <!-- 列表视图 -->
        <div v-if="viewMode === 'list'" class="flex flex-col">
          <div class="flex px-3 py-2 text-[11px] font-medium text-neutral-500 uppercase tracking-wider">
            <div class="flex-1 flex items-center gap-1">
              <span>名称</span>
              <span class="icon-[mdi--chevron-down]"></span>
            </div>
            <div class="w-24">大小</div>
            <div class="w-24">种类</div>
            <div class="w-32">修改日期</div>
          </div>

          <div class="flex flex-col">
            <div
              v-for="file in files"
              :key="file.id"
              class="flex px-3 py-2.5 rounded-xl transition-all cursor-pointer"
              :class="file.isSelected ? 'bg-sky-500/20' : 'hover:bg-white/[0.04]'"
              @click="selectFile(file, $event)"
              @dblclick="openFile(file)"
              @contextmenu.stop="showContextMenu($event, file)"
            >
              <div class="flex-1 flex items-center gap-3">
                <span
                  class="text-xl"
                  :class="getFileIcon(file.type)"
                  :style="{ color: getFileColor(file.type) }"
                ></span>
                <div v-if="renamingFileId === file.id" class="flex-1">
                  <input
                    ref="renameInputRef"
                    v-model="renamingValue"
                    class="w-full bg-neutral-800 border border-sky-400/50 rounded px-2 py-1 text-white text-sm outline-none"
                    @keyup.enter="confirmRename"
                    @keyup.esc="cancelRename"
                    @blur="confirmRename"
                  />
                </div>
                <span v-else class="text-neutral-200 text-[13px]">{{ file.name }}</span>
              </div>
              <div class="w-24 text-neutral-400 text-[13px]">{{ formatSize(file.size) }}</div>
              <div class="w-24 text-neutral-400 text-[13px] capitalize">{{ file.type }}</div>
              <div class="w-32 text-neutral-400 text-[13px]">{{ formatDate(file.modified) }}</div>
            </div>
          </div>
        </div>

        <!-- 网格视图 -->
        <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(100px,1fr))] gap-3">
          <div
            v-for="file in files"
            :key="file.id"
            class="flex flex-col items-center p-3 rounded-xl transition-all cursor-pointer gap-2"
            :class="file.isSelected ? 'bg-sky-500/20' : 'hover:bg-white/[0.04]'"
            @click="selectFile(file, $event)"
            @dblclick="openFile(file)"
            @contextmenu.stop="showContextMenu($event, file)"
          >
            <span
              class="text-5xl"
              :class="getFileIcon(file.type)"
              :style="{ color: getFileColor(file.type) }"
            ></span>
            <div v-if="renamingFileId === file.id" class="w-full">
              <input
                ref="renameInputRef"
                v-model="renamingValue"
                class="w-full bg-neutral-800 border border-sky-400/50 rounded px-1.5 py-0.5 text-white text-xs text-center outline-none"
                @keyup.enter="confirmRename"
                @keyup.esc="cancelRename"
                @blur="confirmRename"
              />
            </div>
            <span v-else class="text-neutral-200 text-xs text-center truncate w-full">{{ file.name }}</span>
            <span class="text-neutral-500 text-[10px]">{{ formatSize(file.size) }}</span>
          </div>
        </div>
      </section>

      <!-- 预览面板 -->
      <aside 
        v-if="isPreviewOpen && previewFile" 
        class="w-[280px] bg-neutral-800/50 border-l border-white/[0.06] p-5 flex flex-col"
      >
        <div class="flex justify-end mb-4">
          <button 
            class="w-7 h-7 rounded-lg bg-white/[0.06] text-neutral-400 flex items-center justify-center transition-all hover:bg-white/[0.12] hover:text-white"
            @click="isPreviewOpen = false"
          >
            <span class="icon-[mdi--close]"></span>
          </button>
        </div>
        <div class="flex flex-col items-center mb-6">
          <span
            class="text-[100px] mb-4"
            :class="getFileIcon(previewFile.type)"
            :style="{ color: getFileColor(previewFile.type) }"
          ></span>
          <h3 class="text-white text-lg font-medium mb-1">{{ previewFile.name }}</h3>
          <p class="text-neutral-500 text-sm uppercase">{{ previewFile.type }}</p>
        </div>
        <div class="flex flex-col gap-3 text-[13px]">
          <div class="flex justify-between py-2 border-b border-white/[0.06]">
            <span class="text-neutral-500">大小</span>
            <span class="text-neutral-300">{{ formatSize(previewFile.size) }}</span>
          </div>
          <div class="flex justify-between py-2 border-b border-white/[0.06]">
            <span class="text-neutral-500">修改</span>
            <span class="text-neutral-300">{{ formatDate(previewFile.modified) }}</span>
          </div>
          <div class="flex justify-between py-2 border-b border-white/[0.06]">
            <span class="text-neutral-500">位置</span>
            <span class="text-neutral-300 truncate max-w-[140px]">{{ currentPath }}</span>
          </div>
        </div>
      </aside>
    </div>

    <!-- 状态栏 -->
    <div class="flex justify-between items-center px-4 py-2 bg-neutral-800/50 border-t border-white/[0.06] text-[12px] text-neutral-400">
      <div class="flex items-center gap-3">
        <span>{{ files.length }} 个项目</span>
        <span v-if="selectedFiles.length > 0" class="text-sky-400">
          已选择 {{ selectedFiles.length }} 个
        </span>
      </div>
      <div class="flex items-center gap-4">
        <span 
          class="cursor-pointer hover:text-white transition-colors"
          @click="isPreviewOpen = !isPreviewOpen"
        >
          {{ isPreviewOpen ? '隐藏预览' : '显示预览' }}
        </span>
        <span>剩余 245 GB 可用</span>
      </div>
    </div>

    <!-- 上下文菜单 -->
    <div
      v-if="contextMenu.show"
      class="fixed bg-neutral-800/95 backdrop-blur-xl rounded-xl border border-white/[0.08] shadow-2xl py-2 min-w-[200px] z-50"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="flex flex-col">
        <button 
          class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left"
          @click="createNewFolder"
        >
          <span class="icon-[mdi--folder-plus] text-neutral-400"></span>
          <span class="flex-1">新建文件夹</span>
          <span class="text-neutral-500 text-xs">⇧⌘N</span>
        </button>
        <button class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left">
          <span class="icon-[mdi--file-plus] text-neutral-400"></span>
          <span class="flex-1">新建文件</span>
          <span class="text-neutral-500 text-xs">⌘N</span>
        </button>
      </div>
      <div v-if="contextMenu.fileId" class="border-t border-white/[0.08] my-1"></div>
      <div v-if="contextMenu.fileId" class="flex flex-col">
        <button 
          class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left"
          @click="startRename(files.find(f => f.id === contextMenu.fileId)!)"
        >
          <span class="icon-[mdi--pencil] text-neutral-400"></span>
          <span class="flex-1">重新命名</span>
          <span class="text-neutral-500 text-xs">↵</span>
        </button>
        <button class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left">
          <span class="icon-[mdi--content-copy] text-neutral-400"></span>
          <span class="flex-1">复制</span>
          <span class="text-neutral-500 text-xs">⌘C</span>
        </button>
        <button class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left">
          <span class="icon-[mdi--content-paste] text-neutral-400"></span>
          <span class="flex-1">粘贴</span>
          <span class="text-neutral-500 text-xs">⌘V</span>
        </button>
        <button 
          class="flex items-center gap-3 px-4 py-2 text-[13px] text-red-400 hover:bg-red-500/10 transition-colors text-left"
          @click="deleteFile(contextMenu.fileId!)"
        >
          <span class="icon-[mdi--delete]"></span>
          <span class="flex-1">移到废纸篓</span>
          <span class="text-red-400/60 text-xs">⌘⌫</span>
        </button>
      </div>
      <div class="border-t border-white/[0.08] my-1"></div>
      <div class="flex flex-col">
        <button class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left">
          <span class="icon-[mdi--information] text-neutral-400"></span>
          <span class="flex-1">显示简介</span>
          <span class="text-neutral-500 text-xs">⌘I</span>
        </button>
        <button class="flex items-center gap-3 px-4 py-2 text-[13px] text-neutral-200 hover:bg-white/[0.08] transition-colors text-left">
          <span class="icon-[mdi--refresh] text-neutral-400"></span>
          <span class="flex-1">刷新</span>
          <span class="text-neutral-500 text-xs">⌘R</span>
        </button>
      </div>
    </div>
  </div>
</template>
