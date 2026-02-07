<script setup lang="ts">
import { ref } from "vue";

const fileRows = ref([
    { name: "home", type: "文件夹", size: "--", modified: "2026-02-07 18:10" },
    { name: "etc", type: "文件夹", size: "--", modified: "2026-02-07 17:42" },
    { name: "var", type: "文件夹", size: "--", modified: "2026-02-07 16:08" },
    { name: "deploy.sh", type: "脚本", size: "12 KB", modified: "2026-02-06 23:12" },
    { name: "report.log", type: "日志", size: "4.2 MB", modified: "2026-02-06 21:05" },
]);

const activeFolder = ref("主目录");
const folders = ["主目录", "下载", "备份", "项目"];
</script>

<template>
    <div class="files-shell">
        <div class="files-toolbar">
            <div class="path">/home/ops</div>
            <div class="files-actions">
                <button>上传</button>
                <button>下载</button>
                <button>新建文件夹</button>
                <button>刷新</button>
            </div>
        </div>
        <div class="files-content">
            <aside class="files-tree">
                <div class="tree-title">位置</div>
                <div v-for="folder in folders" :key="folder" class="tree-item"
                    :class="{ active: activeFolder === folder }" @click="activeFolder = folder">
                    {{ folder }}
                </div>
            </aside>
            <section class="files-list">
                <div class="list-header">
                    <span>名称</span>
                    <span>类型</span>
                    <span>大小</span>
                    <span>修改时间</span>
                </div>
                <div v-for="row in fileRows" :key="row.name" class="list-row">
                    <span class="name">{{ row.name }}</span>
                    <span>{{ row.type }}</span>
                    <span>{{ row.size }}</span>
                    <span>{{ row.modified }}</span>
                </div>
            </section>
        </div>
        <div class="files-status">共 {{ fileRows.length }} 项 · 已同步</div>
    </div>
</template>

<style scoped>
.files-shell {
    height: 100%;
    display: grid;
    grid-template-rows: auto 1fr auto;
}

.files-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.files-toolbar .path {
    font-size: 0.85rem;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.06);
    padding: 6px 12px;
    border-radius: 10px;
}

.files-actions {
    display: flex;
    gap: 8px;
}

.files-actions button {
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
    padding: 6px 10px;
    border-radius: 10px;
    font-size: 0.8rem;
    cursor: pointer;
    transition: border 0.2s ease, transform 0.2s ease;
}

.files-actions button:hover {
    border-color: rgba(125, 211, 252, 0.6);
    transform: translateY(-1px);
}

.files-content {
    display: grid;
    grid-template-columns: 220px 1fr;
    height: 100%;
    overflow: hidden;
}

.files-tree {
    padding: 16px;
    border-right: 1px solid rgba(255, 255, 255, 0.06);
    display: flex;
    flex-direction: column;
    gap: 10px;
    color: var(--text-muted);
}

.tree-title {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--text-secondary);
}

.tree-item {
    padding: 8px 10px;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.03);
    cursor: pointer;
    transition: background 0.2s ease;
}

.tree-item:hover {
    background: rgba(255, 255, 255, 0.06);
}

.tree-item.active {
    background: rgba(125, 211, 252, 0.18);
    color: #c8ecff;
}

.files-list {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    color: var(--text-primary);
    overflow-y: auto;
}

.list-header,
.list-row {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr 1.4fr;
    gap: 12px;
    align-items: center;
}

.list-header {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
}

.list-row {
    padding: 10px 12px;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 12px;
    transition: background 0.2s ease;
}

.list-row:hover {
    background: rgba(255, 255, 255, 0.06);
}

.list-row .name {
    color: #c8ecff;
}

.files-status {
    padding: 10px 16px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
    font-size: 0.8rem;
    color: var(--text-muted);
}
</style>
