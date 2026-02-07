/**
* TextEditorApp.vue - Monaco 文本编辑器组件
* 用于编辑远程服务器上的文本文件
*/
<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, shallowRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { VueMonacoEditor } from "@guolao/vue-monaco-editor";
import * as monaco from "monaco-editor";

interface Props {
    sessionId?: string;
    filePath?: string;
    fileName?: string;
}

const props = defineProps<Props>();

const emit = defineEmits<{
    (e: "close"): void;
    (e: "title-change", title: string): void;
}>();

// 编辑器状态
const editorRef = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
const content = ref("");
const originalContent = ref("");
const isLoading = ref(false);
const isSaving = ref(false);
const error = ref<string | null>(null);
const language = ref("plaintext");

// 是否有未保存的更改
const isDirty = computed(() => content.value !== originalContent.value);

// 文件扩展名到语言的映射
const languageMap: Record<string, string> = {
    js: "javascript",
    ts: "typescript",
    jsx: "javascript",
    tsx: "typescript",
    vue: "html",
    html: "html",
    htm: "html",
    css: "css",
    scss: "scss",
    less: "less",
    json: "json",
    md: "markdown",
    py: "python",
    rs: "rust",
    go: "go",
    java: "java",
    c: "c",
    cpp: "cpp",
    h: "c",
    hpp: "cpp",
    sh: "shell",
    bash: "shell",
    zsh: "shell",
    yml: "yaml",
    yaml: "yaml",
    toml: "ini",
    xml: "xml",
    sql: "sql",
    dockerfile: "dockerfile",
    makefile: "makefile",
    conf: "ini",
    ini: "ini",
    env: "ini",
    log: "plaintext",
    txt: "plaintext",
};

// 根据文件名检测语言
function detectLanguage(fileName: string): string {
    const ext = fileName.split(".").pop()?.toLowerCase() || "";
    const baseName = fileName.toLowerCase();

    // 特殊文件名
    if (baseName === "dockerfile") return "dockerfile";
    if (baseName === "makefile") return "makefile";
    if (baseName.startsWith(".env")) return "ini";

    return languageMap[ext] || "plaintext";
}

// 加载文件内容
async function loadFile() {
    if (!props.sessionId || !props.filePath) {
        error.value = "未指定文件路径";
        return;
    }

    isLoading.value = true;
    error.value = null;

    try {
        const base64Content = await invoke<string>("download_file", {
            sessionId: props.sessionId,
            remotePath: props.filePath,
        });

        // Base64 解码
        const binaryString = atob(base64Content);
        const bytes = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
        }
        const decoder = new TextDecoder("utf-8");
        content.value = decoder.decode(bytes);
        originalContent.value = content.value;

        // 检测语言
        if (props.fileName) {
            language.value = detectLanguage(props.fileName);
        }
    } catch (err: any) {
        console.error("Failed to load file:", err);
        error.value = "加载文件失败: " + err.toString();
    } finally {
        isLoading.value = false;
    }
}

// 保存文件
async function saveFile() {
    if (!props.sessionId || !props.filePath) return;
    if (!isDirty.value) return;

    isSaving.value = true;
    error.value = null;

    try {
        // 编码为 Base64
        const encoder = new TextEncoder();
        const bytes = encoder.encode(content.value);
        let binary = "";
        for (let i = 0; i < bytes.length; i++) {
            binary += String.fromCharCode(bytes[i]);
        }
        const base64Content = btoa(binary);

        await invoke("upload_file", {
            sessionId: props.sessionId,
            remotePath: props.filePath,
            base64Content: base64Content,
        });

        originalContent.value = content.value;
    } catch (err: any) {
        console.error("Failed to save file:", err);
        error.value = "保存失败: " + err.toString();
    } finally {
        isSaving.value = false;
    }
}

// 编辑器挂载回调
function handleEditorMount(editor: monaco.editor.IStandaloneCodeEditor) {
    editorRef.value = editor;

    // 添加保存快捷键
    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
        saveFile();
    });

    // 聚焦编辑器
    editor.focus();
}

// 键盘快捷键
function handleKeydown(event: KeyboardEvent) {
    if ((event.metaKey || event.ctrlKey) && event.key === "s") {
        event.preventDefault();
        saveFile();
    }
}

// 更新标题
watch(isDirty, (dirty) => {
    const title = props.fileName || "未命名";
    emit("title-change", dirty ? `● ${title}` : title);
});

// 初始化
watch(
    () => props.filePath,
    () => {
        if (props.filePath) {
            loadFile();
        }
    },
    { immediate: true }
);

onMounted(() => {
    document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
    document.removeEventListener("keydown", handleKeydown);
});

// 光标位置
const cursorPosition = ref({ line: 1, column: 1 });

function handleCursorChange() {
    if (editorRef.value) {
        const position = editorRef.value.getPosition();
        if (position) {
            cursorPosition.value = { line: position.lineNumber, column: position.column };
        }
    }
}
</script>

<template>
    <div class="text-editor-app">
        <!-- 工具栏 -->
        <header class="editor-toolbar">
            <div class="toolbar-left">
                <button class="toolbar-btn" :disabled="!isDirty || isSaving" :class="{ active: isDirty }"
                    @click="saveFile" title="保存 (⌘S)">
                    <span class="icon-[mdi--content-save]" :class="{ 'animate-pulse': isSaving }"></span>
                    <span class="btn-text">保存</span>
                </button>

                <div class="toolbar-divider"></div>

                <button class="toolbar-btn" @click="editorRef?.trigger('', 'undo', null)" title="撤销 (⌘Z)">
                    <span class="icon-[mdi--undo]"></span>
                </button>
                <button class="toolbar-btn" @click="editorRef?.trigger('', 'redo', null)" title="重做 (⌘⇧Z)">
                    <span class="icon-[mdi--redo]"></span>
                </button>

                <div class="toolbar-divider"></div>

                <button class="toolbar-btn" @click="editorRef?.trigger('', 'actions.find', null)" title="查找 (⌘F)">
                    <span class="icon-[mdi--magnify]"></span>
                </button>
                <button class="toolbar-btn"
                    @click="editorRef?.trigger('', 'editor.action.startFindReplaceAction', null)" title="替换 (⌘H)">
                    <span class="icon-[mdi--find-replace]"></span>
                </button>
            </div>

            <div class="toolbar-right">
                <span class="file-path" :title="props.filePath">{{ props.filePath }}</span>
            </div>
        </header>

        <!-- 编辑器区域 -->
        <main class="editor-container">
            <!-- 加载状态 -->
            <div v-if="isLoading" class="editor-loading">
                <span class="icon-[mdi--loading] animate-spin text-4xl text-blue-500"></span>
                <span class="text-sm text-neutral-400 mt-2">加载中...</span>
            </div>

            <!-- 错误 -->
            <div v-else-if="error" class="editor-error">
                <span class="icon-[mdi--alert-circle] text-4xl text-red-400"></span>
                <span class="text-sm text-red-300 mt-2">{{ error }}</span>
                <button class="retry-btn" @click="loadFile">重试</button>
            </div>

            <!-- Monaco 编辑器 -->
            <VueMonacoEditor v-else v-model:value="content" :language="language" theme="vs-dark" :options="{
                fontSize: 13,
                fontFamily: 'JetBrains Mono, Menlo, Monaco, Consolas, monospace',
                lineHeight: 20,
                minimap: { enabled: true, maxColumn: 80 },
                scrollBeyondLastLine: false,
                wordWrap: 'on',
                tabSize: 2,
                renderWhitespace: 'selection',
                automaticLayout: true,
                padding: { top: 12, bottom: 12 },
                cursorBlinking: 'smooth',
                smoothScrolling: true,
                bracketPairColorization: { enabled: true },
            }" @mount="handleEditorMount" @update:value="handleCursorChange" />
        </main>

        <!-- 状态栏 -->
        <footer class="editor-statusbar">
            <div class="status-left">
                <span class="status-item">
                    <span class="icon-[mdi--code-tags] text-xs"></span>
                    {{ language }}
                </span>
                <span class="status-item">UTF-8</span>
            </div>
            <div class="status-right">
                <span class="status-item">
                    行 {{ cursorPosition.line }}, 列 {{ cursorPosition.column }}
                </span>
                <span v-if="isDirty" class="status-item status-dirty">
                    <span class="icon-[mdi--circle-small]"></span>
                    未保存
                </span>
            </div>
        </footer>
    </div>
</template>

<style scoped>
.text-editor-app {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    border-radius: 0 0 16px 16px;
    overflow: hidden;
}

/* 工具栏 */
.editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 12px;
    background: rgba(30, 30, 30, 0.95);
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    gap: 8px;
}

.toolbar-left,
.toolbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
}

.toolbar-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 5px 8px;
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.6);
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s;
}

.toolbar-btn:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
}

.toolbar-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
}

.toolbar-btn.active {
    color: #4fc3f7;
}

.btn-text {
    font-size: 12px;
}

.toolbar-divider {
    width: 1px;
    height: 16px;
    background: rgba(255, 255, 255, 0.1);
    margin: 0 4px;
}

.file-path {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.4);
    font-family: monospace;
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* 编辑器容器 */
.editor-container {
    flex: 1;
    position: relative;
    overflow: hidden;
}

.editor-loading,
.editor-error {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    background: #1e1e1e;
}

.retry-btn {
    margin-top: 12px;
    padding: 6px 16px;
    background: rgba(59, 130, 246, 0.2);
    border: 1px solid rgba(59, 130, 246, 0.3);
    color: #60a5fa;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    transition: all 0.15s;
}

.retry-btn:hover {
    background: rgba(59, 130, 246, 0.3);
}

/* 状态栏 */
.editor-statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 12px;
    background: #007acc;
    font-size: 11px;
    color: rgba(255, 255, 255, 0.9);
}

.status-left,
.status-right {
    display: flex;
    align-items: center;
    gap: 12px;
}

.status-item {
    display: flex;
    align-items: center;
    gap: 4px;
}

.status-dirty {
    color: #fbbf24;
}
</style>
