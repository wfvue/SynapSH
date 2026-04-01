<script setup lang="ts">
import { ref, computed, nextTick, onMounted, watch, onUnmounted } from "vue";
import { api } from "@/lib/api";

interface Props {
  sessionId?: string;
  initialPath?: string;
}

const props = withDefaults(defineProps<Props>(), {
  initialPath: "/",
});

// 定义事件
const emit = defineEmits<{
  (e: "open-file", filePath: string, fileName: string): void;
}>();

// Types matching backend
interface FileEntry {
  name: string;
  path: string;
  type: "directory" | "file" | "symlink" | "unknown";
  size: number;
  modifiedTime?: string;
  permissions?: string;
  owner?: string;
  group?: string;
  isHidden?: boolean;
}

interface FileItem extends FileEntry {
  id: string;
  isSelected: boolean;
  icon: string;
}

type SortField = "name" | "size" | "modifiedTime" | "type";
type SortOrder = "asc" | "desc";

// State
const currentPath = ref(props.initialPath);
const files = ref<FileItem[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const viewMode = ref<"grid" | "list">("list");
const searchQuery = ref("");
const history = ref<string[]>([props.initialPath]);
const historyIndex = ref(0);
const selectedFiles = computed(() => files.value.filter(f => f.isSelected));
const contextMenu = ref({ show: false, x: 0, y: 0, targetId: null as string | null });
const isRenaming = ref<string | null>(null);
const renameValue = ref("");
const renameInputRef = ref<HTMLInputElement | null>(null);
const sortField = ref<SortField>("name");
const sortOrder = ref<SortOrder>("asc");
const showHidden = ref(true);
const showDetailsPanel = ref(false);

// Modals
const showNewFileModal = ref(false);
const newFileName = ref("");
const showChmodModal = ref(false);
const chmodValue = ref("");
const chmodTargetFile = ref<FileItem | null>(null);

const IGNORED_FILES = [".DS_Store", "Thumbs.db"];

// Computed: Filtered and Sorted Files
const displayedFiles = computed(() => {
  let result = [...files.value];

  // Filter hidden
  if (!showHidden.value) {
    result = result.filter(f => !f.isHidden);
  }

  // Filter by search
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase();
    result = result.filter(f => f.name.toLowerCase().includes(q));
  }

  // Sort
  result.sort((a, b) => {
    // Folders always first
    if (a.type === "directory" && b.type !== "directory") return -1;
    if (a.type !== "directory" && b.type === "directory") return 1;

    let cmp = 0;
    switch (sortField.value) {
      case "name":
        cmp = a.name.localeCompare(b.name);
        break;
      case "size":
        cmp = a.size - b.size;
        break;
      case "modifiedTime":
        cmp = (a.modifiedTime || "").localeCompare(b.modifiedTime || "");
        break;
      case "type":
        cmp = a.type.localeCompare(b.type);
        break;
    }
    return sortOrder.value === "asc" ? cmp : -cmp;
  });

  return result;
});

// Helper: Get Icon
function getFileIcon(file: FileEntry): string {
  if (file.type === "directory") return "icon-[mdi--folder] text-blue-400";
  if (file.type === "symlink") return "icon-[mdi--link-variant] text-cyan-400";

  const ext = file.name.split('.').pop()?.toLowerCase();
  const iconMap: Record<string, string> = {
    // Images
    png: "icon-[mdi--file-image] text-purple-400",
    jpg: "icon-[mdi--file-image] text-purple-400",
    jpeg: "icon-[mdi--file-image] text-purple-400",
    gif: "icon-[mdi--file-image] text-purple-400",
    svg: "icon-[mdi--file-image] text-purple-400",
    webp: "icon-[mdi--file-image] text-purple-400",
    // Code
    js: "icon-[mdi--language-javascript] text-yellow-400",
    ts: "icon-[mdi--language-typescript] text-blue-400",
    vue: "icon-[mdi--vuejs] text-green-400",
    json: "icon-[mdi--code-json] text-yellow-500",
    html: "icon-[mdi--language-html5] text-orange-400",
    css: "icon-[mdi--language-css3] text-blue-400",
    rs: "icon-[mdi--language-rust] text-orange-300",
    py: "icon-[mdi--language-python] text-blue-300",
    go: "icon-[mdi--language-go] text-cyan-400",
    sh: "icon-[mdi--bash] text-green-400",
    // Archives
    zip: "icon-[mdi--zip-box] text-orange-400",
    tar: "icon-[mdi--zip-box] text-orange-400",
    gz: "icon-[mdi--zip-box] text-orange-400",
    "7z": "icon-[mdi--zip-box] text-orange-400",
    rar: "icon-[mdi--zip-box] text-orange-400",
    // Documents
    pdf: "icon-[mdi--file-pdf-box] text-red-400",
    doc: "icon-[mdi--microsoft-word] text-blue-500",
    docx: "icon-[mdi--microsoft-word] text-blue-500",
    xls: "icon-[mdi--microsoft-excel] text-green-500",
    xlsx: "icon-[mdi--microsoft-excel] text-green-500",
    txt: "icon-[mdi--file-document-outline] text-gray-400",
    md: "icon-[mdi--language-markdown] text-gray-300",
    log: "icon-[mdi--file-document-outline] text-gray-500",
    // Config
    yml: "icon-[mdi--file-cog] text-pink-400",
    yaml: "icon-[mdi--file-cog] text-pink-400",
    toml: "icon-[mdi--file-cog] text-orange-300",
    conf: "icon-[mdi--file-cog] text-gray-400",
    env: "icon-[mdi--file-cog] text-yellow-500",
  };
  return iconMap[ext || ""] || "icon-[mdi--file] text-gray-500";
}

// Formatters
function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

function formatDate(dateStr?: string): string {
  if (!dateStr) return "--";
  try {
    const date = new Date(dateStr);
    return new Intl.DateTimeFormat('zh-CN', {
      year: 'numeric', month: '2-digit', day: '2-digit',
      hour: '2-digit', minute: '2-digit'
    }).format(date);
  } catch {
    return dateStr;
  }
}

function formatPermissions(perms?: string): string {
  if (!perms) return "---";
  const mode = parseInt(perms, 8);
  const r = (m: number) => ((m & 4) ? 'r' : '-') + ((m & 2) ? 'w' : '-') + ((m & 1) ? 'x' : '-');
  return r((mode >> 6) & 7) + r((mode >> 3) & 7) + r(mode & 7);
}

// Core: Fetch Files
async function fetchFiles(path: string) {
  if (!props.sessionId) {
    error.value = "未连接到会话";
    return;
  }

  isLoading.value = true;
  error.value = null;

  try {
    const res = await api.listFiles(props.sessionId, path);

    files.value = res.entries
      .filter(e => !IGNORED_FILES.includes(e.name))
      .map(e => ({
        ...e,
        id: e.path,
        isSelected: false,
        icon: getFileIcon(e)
      }));
  } catch (err: any) {
    console.error("Failed to list files:", err);
    error.value = err.toString();
  } finally {
    isLoading.value = false;
  }
}

// Navigation
async function navigateTo(path: string) {
  if (path === currentPath.value) return;

  if (historyIndex.value < history.value.length - 1) {
    history.value = history.value.slice(0, historyIndex.value + 1);
  }
  history.value.push(path);
  historyIndex.value++;

  currentPath.value = path;
  await fetchFiles(path);
}

async function goBack() {
  if (historyIndex.value > 0) {
    historyIndex.value--;
    currentPath.value = history.value[historyIndex.value];
    await fetchFiles(currentPath.value);
  }
}

async function goForward() {
  if (historyIndex.value < history.value.length - 1) {
    historyIndex.value++;
    currentPath.value = history.value[historyIndex.value];
    await fetchFiles(currentPath.value);
  }
}

async function goUp() {
  const parent = currentPath.value.split('/').slice(0, -1).join('/') || '/';
  await navigateTo(parent);
}

async function refresh() {
  await fetchFiles(currentPath.value);
}

// Selection
function selectFile(file: FileItem, event: MouseEvent) {
  if (event.metaKey || event.ctrlKey) {
    file.isSelected = !file.isSelected;
  } else if (event.shiftKey && selectedFiles.value.length > 0) {
    const lastIdx = displayedFiles.value.findIndex(f => f.isSelected);
    const currIdx = displayedFiles.value.findIndex(f => f.id === file.id);
    const [start, end] = lastIdx < currIdx ? [lastIdx, currIdx] : [currIdx, lastIdx];
    displayedFiles.value.forEach((f, i) => {
      f.isSelected = i >= start && i <= end;
    });
  } else {
    files.value.forEach(f => f.isSelected = false);
    file.isSelected = true;
  }
}

function clearSelection() {
  files.value.forEach(f => f.isSelected = false);
}

function selectAll() {
  displayedFiles.value.forEach(f => f.isSelected = true);
}

async function openItem(file: FileItem) {
  if (file.type === "directory") {
    await navigateTo(file.path);
  } else {
    // 在编辑器中打开文件
    emit("open-file", file.path, file.name);
  }
}

// Context Menu
function onContextMenu(event: MouseEvent, file: FileItem | null) {
  event.preventDefault();
  if (file && !file.isSelected) {
    selectFile(file, event);
  }
  contextMenu.value = {
    show: true,
    x: Math.min(event.clientX, window.innerWidth - 200),
    y: Math.min(event.clientY, window.innerHeight - 300),
    targetId: file?.id || null
  };
}

function hideContextMenu() {
  contextMenu.value.show = false;
}

function copyPath(file: FileItem) {
  hideContextMenu();
  window.navigator.clipboard.writeText(file.path);
}

// File Operations
async function createFolder() {
  if (!props.sessionId) return;
  hideContextMenu();
  try {
    const name = "新建文件夹";
    const path = `${currentPath.value}/${name}`;
    await api.createFolder(props.sessionId, path);
    await fetchFiles(currentPath.value);
    // Auto start rename
    const newFolder = files.value.find(f => f.name === name);
    if (newFolder) {
      await nextTick();
      startRename(newFolder);
    }
  } catch (err: any) {
    error.value = "创建文件夹失败: " + err;
  }
}

async function createFile() {
  if (!props.sessionId || !newFileName.value.trim()) return;
  try {
    const path = `${currentPath.value}/${newFileName.value.trim()}`;
    await api.createFile(props.sessionId, path);
    await fetchFiles(currentPath.value);
    showNewFileModal.value = false;
    newFileName.value = "";
  } catch (err: any) {
    error.value = "创建文件失败: " + err;
  }
}

async function startRename(file: FileItem) {
  hideContextMenu();
  isRenaming.value = file.id;
  renameValue.value = file.name;
  await nextTick();
  renameInputRef.value?.focus();
  renameInputRef.value?.select();
}

async function finishRename(file: FileItem) {
  if (!isRenaming.value || !renameValue.value.trim() || renameValue.value === file.name) {
    isRenaming.value = null;
    return;
  }

  if (!props.sessionId) {
    error.value = "Session not available";
    isRenaming.value = null;
    return;
  }

  try {
    const oldPath = file.path;
    const newPath = `${currentPath.value}/${renameValue.value.trim()}`;
    await api.renameFile(props.sessionId, oldPath, newPath);
    await fetchFiles(currentPath.value);
  } catch (err: any) {
    error.value = "重命名失败: " + err;
  } finally {
    isRenaming.value = null;
  }
}

async function deleteSelected() {
  if (!props.sessionId || selectedFiles.value.length === 0) return;
  hideContextMenu();

  if (!confirm(`确定删除 ${selectedFiles.value.length} 个项目吗？此操作不可恢复！`)) return;

  try {
    for (const file of selectedFiles.value) {
      await api.deleteFile(props.sessionId, file.path, file.type === "directory");
    }
    await fetchFiles(currentPath.value);
  } catch (err: any) {
    error.value = "删除失败: " + err;
  }
}

function openChmodModal(file: FileItem) {
  hideContextMenu();
  chmodTargetFile.value = file;
  chmodValue.value = file.permissions || "644";
  showChmodModal.value = true;
}

async function applyChmod() {
  if (!props.sessionId || !chmodTargetFile.value || !chmodValue.value.trim()) return;

  try {
    const mode = parseInt(chmodValue.value, 8);
    if (isNaN(mode) || mode < 0 || mode > 0o777) {
      error.value = "无效的权限值";
      return;
    }
    await api.chmodFile(props.sessionId, chmodTargetFile.value.path, mode);
    await fetchFiles(currentPath.value);
    showChmodModal.value = false;
  } catch (err: any) {
    error.value = "修改权限失败: " + err;
  }
}

async function downloadFile(file: FileItem) {
  if (!props.sessionId || file.type === "directory") return;
  hideContextMenu();

  try {
    const base64Content = await api.downloadFile(props.sessionId, file.path);

    // Decode and trigger download
    const binaryString = atob(base64Content);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i);
    }
    const blob = new Blob([bytes]);
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = file.name;
    a.click();
    URL.revokeObjectURL(url);
  } catch (err: any) {
    error.value = "下载失败: " + err;
  }
}

// Keyboard Shortcuts
function handleKeydown(event: KeyboardEvent) {
  if (showNewFileModal.value || showChmodModal.value || isRenaming.value) return;

  if ((event.metaKey || event.ctrlKey) && event.key === "a") {
    event.preventDefault();
    selectAll();
  }
  if (event.key === "Delete" || event.key === "Backspace") {
    if (selectedFiles.value.length > 0) {
      deleteSelected();
    }
  }
  if (event.key === "F2" && selectedFiles.value.length === 1) {
    startRename(selectedFiles.value[0]);
  }
  if (event.key === "Enter" && selectedFiles.value.length === 1) {
    openItem(selectedFiles.value[0]);
  }
  if (event.key === "Escape") {
    clearSelection();
    hideContextMenu();
  }
}

// Sorting
function toggleSort(field: SortField) {
  if (sortField.value === field) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortField.value = field;
    sortOrder.value = "asc";
  }
}

// Watchers
watch(() => props.sessionId, (newVal) => {
  if (newVal) fetchFiles(currentPath.value);
}, { immediate: true });

onMounted(() => {
  document.addEventListener('click', hideContextMenu);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('click', hideContextMenu);
  document.removeEventListener('keydown', handleKeydown);
});

// Details panel
const detailsFile = computed(() => selectedFiles.value.length === 1 ? selectedFiles.value[0] : null);
</script>

<template>
  <div class="files-app" tabindex="0">

    <!-- Header / Toolbar -->
    <header
      class="flex items-center gap-3 px-4 py-2.5 bg-card/60 border-b border-border/50 backdrop-blur-md z-10 text-foreground">
      <!-- Nav Controls -->
      <div class="flex items-center gap-0.5">
        <button
          class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground disabled:opacity-30 transition text-xl"
          :disabled="historyIndex <= 0" @click="goBack" title="后退">
          <span class="icon-[mdi--chevron-left]"></span>
        </button>
        <button
          class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground disabled:opacity-30 transition text-xl"
          :disabled="historyIndex >= history.length - 1" @click="goForward" title="前进">
          <span class="icon-[mdi--chevron-right]"></span>
        </button>
        <button class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition text-xl"
          @click="goUp" title="上级目录">
          <span class="icon-[mdi--arrow-up]"></span>
        </button>
        <button class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition text-xl"
          @click="refresh" title="刷新">
          <span class="icon-[mdi--refresh]" :class="{ 'animate-spin': isLoading }"></span>
        </button>
      </div>

      <div class="w-px h-5 bg-border"></div>

      <!-- Path Bar -->
      <div
        class="flex-1 flex bg-muted/30 rounded-lg px-3 py-1.5 border border-border/50 items-center gap-2 overflow-hidden focus-within:ring-1 ring-ring/50">
        <span class="icon-[mdi--folder-home] text-muted-foreground text-base"></span>
        <input v-model="currentPath" @keyup.enter="navigateTo(currentPath)"
          class="bg-transparent border-none outline-none w-full text-sm text-foreground placeholder:-muted-foreground font-mono" />
      </div>

      <!-- Search -->
      <div class="relative w-40">
        <span
          class="absolute left-2.5 top-1/2 -translate-y-1/2 icon-[mdi--magnify] text-muted-foreground text-base"></span>
        <input v-model="searchQuery" placeholder="搜索"
          class="w-full bg-muted/30 border border-border/50 rounded-lg pl-8 pr-3 py-1.5 text-sm outline-none focus:bg-muted/50 focus:border-border transition-all placeholder-muted-foreground text-foreground" />
      </div>

      <div class="w-px h-5 bg-border"></div>

      <!-- Actions -->
      <div class="flex items-center gap-1">
        <button @click="showNewFileModal = true"
          class="p-1.5 rounded-lg hover:bg-accent text-muted-foreground hover:text-foreground transition text-xl"
          title="新建文件">
          <span class="icon-[mdi--file-plus]"></span>
        </button>
        <button @click="createFolder"
          class="p-1.5 rounded-lg hover:bg-white/10 text-neutral-400 hover:text-white transition text-xl" title="新建文件夹">
          <span class="icon-[mdi--folder-plus]"></span>
        </button>
      </div>

      <div class="w-px h-5 bg-border"></div>

      <!-- View Toggle -->
      <div class="flex bg-muted/30 rounded-lg p-0.5 gap-0.5 border border-border/50">
        <button @click="viewMode = 'list'"
          :class="['p-1.5 rounded-md transition text-lg', viewMode === 'list' ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:text-foreground']"
          title="列表视图">
          <span class="icon-[mdi--view-list]"></span>
        </button>
        <button @click="viewMode = 'grid'"
          :class="['p-1.5 rounded-md transition text-lg', viewMode === 'grid' ? 'bg-white/10 text-white' : 'text-neutral-500 hover:text-neutral-300']"
          title="网格视图">
          <span class="icon-[mdi--view-grid]"></span>
        </button>
      </div>

      <button @click="showHidden = !showHidden"
        :class="['p-1.5 rounded-lg transition text-xl', showHidden ? 'bg-primary/20 text-primary' : 'text-muted-foreground hover:text-foreground hover:bg-accent']"
        title="显示隐藏文件">
        <span class="icon-[mdi--eye]"></span>
      </button>

      <button @click="showDetailsPanel = !showDetailsPanel"
        :class="['p-1.5 rounded-lg transition text-xl', showDetailsPanel ? 'bg-blue-500/20 text-blue-400' : 'text-neutral-500 hover:text-white hover:bg-white/10']"
        title="详情面板">
        <span class="icon-[mdi--information-outline]"></span>
      </button>
    </header>

    <!-- Main Content -->
    <div class="flex flex-1 overflow-hidden">

      <!-- Sidebar -->
      <aside
        class="w-52 bg-card/30 backdrop-blur-sm border-r border-border/30 flex flex-col py-3 gap-4 overflow-y-auto">
        <div class="px-2">
          <div class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-3 mb-1.5">收藏</div>
          <div class="flex flex-col gap-0.5">
            <button v-for="item in [
              { path: '/home', icon: 'icon-[mdi--account-group]', name: '用户目录' },
              { path: '/tmp', icon: 'icon-[mdi--folder-clock]', name: '临时目录' },
              { path: '/var/log', icon: 'icon-[mdi--file-document-outline]', name: '日志' },
              { path: '/etc', icon: 'icon-[mdi--cog]', name: '配置' },
            ]" :key="item.path" @click="navigateTo(item.path)"
              :class="['flex items-center gap-2.5 px-3 py-1.5 rounded-lg text-xs transition', currentPath === item.path ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground']">
              <span :class="item.icon"></span>
              {{ item.name }}
            </button>
          </div>
        </div>
        <div class="px-2">
          <div class="text-[10px] font-bold text-muted-foreground uppercase tracking-widest px-3 mb-1.5">系统</div>
          <div class="flex flex-col gap-0.5">
            <button @click="navigateTo('/')"
              :class="['flex items-center gap-2.5 px-3 py-1.5 rounded-lg text-xs transition', currentPath === '/' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground']">
              <span class="icon-[mdi--harddisk]"></span>
              根目录
            </button>
          </div>
        </div>
      </aside>

      <!-- File Area -->
      <main class="flex-1 relative overflow-y-auto" @click.self="clearSelection"
        @contextmenu.self="onContextMenu($event, null)">

        <!-- Loading -->
        <div v-if="isLoading"
          class="absolute inset-0 flex items-center justify-center bg-background/60 z-20 backdrop-blur-sm">
          <span class="icon-[mdi--loading] animate-spin text-4xl text-primary"></span>
        </div>

        <!-- Error -->
        <div v-if="error"
          class="absolute inset-x-0 top-3 mx-auto w-max max-w-md bg-destructive/10 border border-destructive/50 text-destructive-foreground px-4 py-2.5 rounded-lg flex items-center gap-3 shadow-xl backdrop-blur-md z-30">
          <span class="icon-[mdi--alert-circle] text-destructive text-lg"></span>
          <span class="text-sm">{{ error }}</span>
          <button @click="error = null" class="ml-auto hover:bg-red-500/20 rounded p-1">
            <span class="icon-[mdi--close]"></span>
          </button>
        </div>

        <!-- Empty -->
        <div v-if="displayedFiles.length === 0 && !isLoading && !error"
          class="h-full flex flex-col items-center justify-center text-muted-foreground">
          <span class="icon-[mdi--folder-open-outline] text-6xl mb-3 opacity-50"></span>
          <p class="text-sm">文件夹为空</p>
        </div>

        <!-- Content -->
        <div v-else class="p-3 min-h-full">

          <!-- List View -->
          <div v-if="viewMode === 'list'" class="flex flex-col">
            <!-- Header -->
            <div
              class="grid grid-cols-[24px_1fr_100px_80px_80px] gap-3 px-3 py-1.5 text-[10px] font-medium text-muted-foreground border-b border-border/30 uppercase tracking-wide sticky top-0 bg-background/90 backdrop-blur-sm z-10">
              <span></span>
              <button @click="toggleSort('name')"
                class="text-left flex items-center gap-1 hover:text-foreground transition">
                名称
                <span v-if="sortField === 'name'"
                  :class="sortOrder === 'asc' ? 'icon-[mdi--arrow-up]' : 'icon-[mdi--arrow-down]'"
                  class="text-xs"></span>
              </button>
              <button @click="toggleSort('modifiedTime')"
                class="text-right flex items-center justify-end gap-1 hover:text-foreground transition">
                修改时间
                <span v-if="sortField === 'modifiedTime'"
                  :class="sortOrder === 'asc' ? 'icon-[mdi--arrow-up]' : 'icon-[mdi--arrow-down]'"
                  class="text-xs"></span>
              </button>
              <button @click="toggleSort('size')"
                class="text-right flex items-center justify-end gap-1 hover:text-foreground transition">
                大小
                <span v-if="sortField === 'size'"
                  :class="sortOrder === 'asc' ? 'icon-[mdi--arrow-up]' : 'icon-[mdi--arrow-down]'"
                  class="text-xs"></span>
              </button>
              <span class="text-right">权限</span>
            </div>
            <!-- Rows -->
            <div v-for="file in displayedFiles" :key="file.id"
              class="grid grid-cols-[24px_1fr_100px_80px_80px] gap-3 items-center px-3 py-1.5 rounded-lg cursor-default transition-colors group border border-transparent"
              :class="file.isSelected ? 'bg-primary/20 border-primary/30' : 'hover:bg-accent hover:border-border/30'"
              @click.stop="selectFile(file, $event)" @dblclick="openItem(file)"
              @contextmenu.prevent.stop="onContextMenu($event, file)">
              <span :class="[file.icon, 'text-lg']"></span>

              <div v-if="isRenaming === file.id" class="w-full">
                <input ref="renameInputRef" v-model="renameValue" @blur="finishRename(file)"
                  @keydown.enter="finishRename(file)" @keydown.esc="isRenaming = null"
                  class="w-full max-w-[300px] bg-background text-foreground text-sm border border-primary rounded px-2 py-0.5 outline-none" />
              </div>
              <span v-else class="text-sm text-foreground/80 group-hover:text-foreground truncate"
                :class="{ 'text-muted-foreground': file.isHidden }">{{ file.name }}</span>

              <span class="text-[11px] text-muted-foreground text-right tabular-nums">{{ formatDate(file.modifiedTime)
              }}</span>
              <span class="text-[11px] text-muted-foreground text-right tabular-nums">{{ file.type !== 'directory' ?
                formatSize(file.size) : '--' }}</span>
              <span class="text-[11px] text-muted-foreground text-right font-mono">{{
                formatPermissions(file.permissions)
                }}</span>
            </div>
          </div>

          <!-- Grid View -->
          <div v-else class="grid grid-cols-[repeat(auto-fill,minmax(90px,1fr))] gap-3">
            <div v-for="file in displayedFiles" :key="file.id"
              class="group relative flex flex-col items-center gap-1.5 p-2.5 rounded-xl cursor-default transition-all duration-200 border border-transparent"
              :class="file.isSelected ? 'bg-primary/20 border-primary/30 shadow-lg' : 'hover:bg-accent hover:border-border/30'"
              @click.stop="selectFile(file, $event)" @dblclick="openItem(file)"
              @contextmenu.prevent.stop="onContextMenu($event, file)">
              <span :class="[file.icon, 'text-4xl transition-transform group-hover:scale-110 duration-200']"></span>

              <div v-if="isRenaming === file.id" class="w-full">
                <input ref="renameInputRef" v-model="renameValue" @blur="finishRename(file)"
                  @keydown.enter="finishRename(file)" @keydown.esc="isRenaming = null"
                  class="w-full bg-background text-foreground text-[10px] text-center border border-primary rounded px-1 py-0.5 outline-none" />
              </div>
              <span v-else
                class="text-[10px] text-foreground/80 text-center break-all line-clamp-2 w-full px-0.5 group-hover:text-foreground transition-colors"
                :class="{ 'text-muted-foreground': file.isHidden }">
                {{ file.name }}
              </span>
            </div>
          </div>
        </div>
      </main>

      <!-- Details Panel -->
      <aside v-if="showDetailsPanel"
        class="w-56 bg-card/30 backdrop-blur-sm border-l border-border/30 p-4 overflow-y-auto">
        <div v-if="detailsFile" class="flex flex-col gap-4">
          <div class="flex flex-col items-center gap-2">
            <span :class="[detailsFile.icon, 'text-6xl']"></span>
            <span class="text-sm font-medium text-center break-all">{{ detailsFile.name }}</span>
          </div>
          <div class="space-y-2 text-xs">
            <div class="flex justify-between"><span class="text-muted-foreground">类型</span><span>{{ detailsFile.type
            }}</span></div>
            <div class="flex justify-between"><span class="text-muted-foreground">大小</span><span>{{
              formatSize(detailsFile.size) }}</span></div>
            <div class="flex justify-between"><span class="text-muted-foreground">修改时间</span><span class="text-right">{{
              formatDate(detailsFile.modifiedTime) }}</span></div>
            <div class="flex justify-between"><span class="text-muted-foreground">权限</span><span class="font-mono">{{
              formatPermissions(detailsFile.permissions) }}</span></div>
            <div class="flex justify-between"><span class="text-muted-foreground">所有者</span><span>{{ detailsFile.owner
              ||
              '--' }}</span></div>
            <div class="flex justify-between"><span class="text-muted-foreground">组</span><span>{{ detailsFile.group ||
              '--'
                }}</span></div>
          </div>
        </div>
        <div v-else-if="selectedFiles.length > 1" class="text-center text-muted-foreground text-sm">
          已选择 {{ selectedFiles.length }} 个项目
        </div>
        <div v-else class="text-center text-muted-foreground text-sm">
          选择文件查看详情
        </div>
      </aside>
    </div>

    <!-- Context Menu - macOS 风格 -->
    <Teleport to="body">
      <Transition name="context-menu">
        <div v-if="contextMenu.show"
          class="context-menu fixed z-[100] min-w-[220px] bg-popover/80 backdrop-blur-2xl border border-border/50 text-popover-foreground rounded-xl shadow-2xl py-1.5 overflow-hidden"
          :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }">
          <template v-if="contextMenu.targetId">
            <!-- 打开操作 -->
            <button @click="openItem(files.find(f => f.id === contextMenu.targetId)!)" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--open-in-new] group-hover:scale-110"></span>
              <span class="flex-1">打开</span>
              <span class="context-menu-shortcut">⏎</span>
            </button>
            <button v-if="files.find(f => f.id === contextMenu.targetId)?.type !== 'directory'"
              @click="downloadFile(files.find(f => f.id === contextMenu.targetId)!)" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--download] group-hover:scale-110"></span>
              <span class="flex-1">下载</span>
              <span class="context-menu-shortcut">⌘D</span>
            </button>

            <div class="context-menu-divider"></div>

            <!-- 编辑操作 -->
            <button @click="startRename(files.find(f => f.id === contextMenu.targetId)!)"
              class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--rename-box] group-hover:scale-110"></span>
              <span class="flex-1">重命名</span>
              <span class="context-menu-shortcut">F2</span>
            </button>
            <button @click="copyPath(files.find(f => f.id === contextMenu.targetId)!)" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--content-copy] group-hover:scale-110"></span>
              <span class="flex-1">复制路径</span>
              <span class="context-menu-shortcut">⌘C</span>
            </button>
            <button @click="openChmodModal(files.find(f => f.id === contextMenu.targetId)!)"
              class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--shield-key] group-hover:scale-110"></span>
              <span class="flex-1">修改权限</span>
            </button>

            <div class="context-menu-divider"></div>

            <!-- 显示简介 -->
            <button @click="showDetailsPanel = true; hideContextMenu()" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--information-outline] group-hover:scale-110"></span>
              <span class="flex-1">显示简介</span>
              <span class="context-menu-shortcut">⌘I</span>
            </button>

            <div class="context-menu-divider"></div>

            <!-- 危险操作 -->
            <button @click="deleteSelected" class="context-menu-item context-menu-item-danger group">
              <span class="context-menu-icon icon-[mdi--trash-can] group-hover:scale-110"></span>
              <span class="flex-1">移到废纸篓</span>
              <span class="context-menu-shortcut">⌘⌫</span>
            </button>
          </template>

          <template v-else>
            <!-- 新建操作 -->
            <button @click="showNewFileModal = true; hideContextMenu()" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--file-plus] group-hover:scale-110"></span>
              <span class="flex-1">新建文件</span>
              <span class="context-menu-shortcut">⌘N</span>
            </button>
            <button @click="createFolder" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--folder-plus] group-hover:scale-110"></span>
              <span class="flex-1">新建文件夹</span>
              <span class="context-menu-shortcut">⇧⌘N</span>
            </button>

            <div class="context-menu-divider"></div>

            <!-- 视图操作 -->
            <button @click="refresh(); hideContextMenu()" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--refresh] group-hover:scale-110"></span>
              <span class="flex-1">刷新</span>
              <span class="context-menu-shortcut">⌘R</span>
            </button>
            <button @click="selectAll(); hideContextMenu()" class="context-menu-item group">
              <span class="context-menu-icon icon-[mdi--select-all] group-hover:scale-110"></span>
              <span class="flex-1">全选</span>
              <span class="context-menu-shortcut">⌘A</span>
            </button>

            <div class="context-menu-divider"></div>

            <!-- 排列方式 -->
            <div class="context-menu-label">排列方式</div>
            <button v-for="opt in [
              { field: 'name', label: '名称' },
              { field: 'size', label: '大小' },
              { field: 'modifiedTime', label: '修改日期' },
              { field: 'type', label: '类型' }
            ]" :key="opt.field" @click="toggleSort(opt.field as SortField); hideContextMenu()"
              class="context-menu-item group pl-8">
              <span v-if="sortField === opt.field" class="absolute left-3 icon-[mdi--check] text-primary"></span>
              <span class="flex-1">{{ opt.label }}</span>
            </button>

            <div class="context-menu-divider"></div>

            <!-- 显示选项 -->
            <button @click="showHidden = !showHidden; hideContextMenu()" class="context-menu-item group">
              <span v-if="showHidden" class="absolute left-3 icon-[mdi--check] text-primary"></span>
              <span class="context-menu-icon icon-[mdi--eye] group-hover:scale-110 ml-5"></span>
              <span class="flex-1">显示隐藏文件</span>
              <span class="context-menu-shortcut">⇧⌘.</span>
            </button>
          </template>
        </div>
      </Transition>
    </Teleport>

    <!-- New File Modal -->
    <Teleport to="body">
      <div v-if="showNewFileModal"
        class="fixed inset-0 bg-background/60 backdrop-blur-sm flex items-center justify-center z-[200]"
        @click.self="showNewFileModal = false">
        <div class="bg-popover text-popover-foreground border border-border/50 rounded-2xl shadow-2xl w-80 p-5">
          <div class="text-base font-medium mb-4">新建文件</div>
          <input v-model="newFileName" placeholder="文件名" @keyup.enter="createFile"
            class="w-full bg-muted/30 border border-border/50 rounded-lg px-3 py-2 text-sm outline-none focus:border-primary mb-4"
            autofocus />
          <div class="flex justify-end gap-2">
            <button @click="showNewFileModal = false"
              class="px-4 py-1.5 text-sm rounded-lg hover:bg-accent transition">取消</button>
            <button @click="createFile"
              class="px-4 py-1.5 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition">创建</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Chmod Modal -->
    <Teleport to="body">
      <div v-if="showChmodModal"
        class="fixed inset-0 bg-background/60 backdrop-blur-sm flex items-center justify-center z-[200]"
        @click.self="showChmodModal = false">
        <div class="bg-popover text-popover-foreground border border-border/50 rounded-2xl shadow-2xl w-80 p-5">
          <div class="text-base font-medium mb-2">修改权限</div>
          <div class="text-xs text-muted-foreground mb-4">{{ chmodTargetFile?.name }}</div>
          <div class="flex items-center gap-2 mb-4">
            <span class="text-sm text-foreground/80">权限 (八进制):</span>
            <input v-model="chmodValue" placeholder="644" @keyup.enter="applyChmod" maxlength="3"
              class="w-20 bg-muted/30 border border-border/50 rounded-lg px-3 py-1.5 text-sm font-mono text-center outline-none focus:border-primary bg-transparent text-foreground"
              autofocus />
          </div>
          <div class="text-[10px] text-muted-foreground mb-4">
            预览: {{ formatPermissions(chmodValue) }}
          </div>
          <div class="flex justify-end gap-2">
            <button @click="showChmodModal = false"
              class="px-4 py-1.5 text-sm rounded-lg hover:bg-accent transition">取消</button>
            <button @click="applyChmod"
              class="px-4 py-1.5 text-sm rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition">应用</button>
          </div>
        </div>
      </div>
    </Teleport>

    <!-- Footer -->
    <footer
      class="bg-muted/50 border-t border-border/30 py-1 px-4 text-[10px] text-muted-foreground flex justify-between select-none">
      <span class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full" :class="props.sessionId ? 'bg-success' : 'bg-destructive'"></span>
        {{ props.sessionId ? '已连接' : '未连接' }}
      </span>
      <span class="flex gap-4">
        <span>{{ displayedFiles.length }} 个项目</span>
        <span v-if="selectedFiles.length > 0">{{ selectedFiles.length }} 个选中</span>
      </span>
    </footer>
  </div>
</template>

<style scoped>
.files-app {
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
  background: var(--background);
  color: var(--foreground);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  user-select: none;
  overflow: hidden;
  position: relative;
  border-radius: 0 0 18px 18px;
}

::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 99px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}

/* Context Menu - macOS 风格 */
.context-menu {
  box-shadow:
    0 8px 32px rgba(0, 0, 0, 0.5),
    0 0 0 1px rgba(255, 255, 255, 0.05) inset;
}

.context-menu-item {
  position: relative;
  width: 100%;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 14px;
  font-size: 13px;
  color: var(--foreground);
  transition: all 0.15s ease;
}

.context-menu-item:hover {
  background: var(--primary);
  color: var(--primary-foreground);
}

.context-menu-item-danger {
  color: var(--destructive);
}

.context-menu-item-danger:hover {
  background: var(--destructive);
  color: var(--destructive-foreground);
}

.context-menu-icon {
  font-size: 16px;
  opacity: 0.9;
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.context-menu-shortcut {
  font-size: 11px;
  color: var(--muted-foreground);
  font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  margin-left: auto;
  flex-shrink: 0;
}

.context-menu-item:hover .context-menu-shortcut {
  color: var(--primary-foreground);
}

.context-menu-divider {
  height: 1px;
  background: var(--border);
  opacity: 0.5;
  margin: 4px 12px;
}

.context-menu-label {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--muted-foreground);
  padding: 6px 14px 4px;
}

/* Context Menu 动画 */
.context-menu-enter-active {
  animation: context-menu-in 0.15s ease-out;
}

.context-menu-leave-active {
  animation: context-menu-out 0.1s ease-in;
}

@keyframes context-menu-in {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-4px);
  }

  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

@keyframes context-menu-out {
  from {
    opacity: 1;
    transform: scale(1);
  }

  to {
    opacity: 0;
    transform: scale(0.95);
  }
}
</style>