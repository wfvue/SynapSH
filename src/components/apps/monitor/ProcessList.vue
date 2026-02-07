<script setup lang="ts">
import { ref, computed } from "vue";

export interface ProcessInfo {
    pid: number;
    name: string;
    cpu: number;
    memory: number;
    user: string;
}

const props = defineProps<{
    processes: ProcessInfo[];
}>();

const emit = defineEmits<{
    kill: [pid: number];
}>();

const searchQuery = ref("");
const sortKey = ref<"cpu" | "memory" | "name">("cpu");
const sortAsc = ref(false);

const filteredProcesses = computed(() => {
    let list = [...props.processes];

    // 搜索过滤
    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        list = list.filter(p =>
            p.name.toLowerCase().includes(query) ||
            p.user.toLowerCase().includes(query) ||
            String(p.pid).includes(query)
        );
    }

    // 排序
    list.sort((a, b) => {
        let cmp = 0;
        if (sortKey.value === "cpu") cmp = b.cpu - a.cpu;
        else if (sortKey.value === "memory") cmp = b.memory - a.memory;
        else cmp = a.name.localeCompare(b.name);
        return sortAsc.value ? -cmp : cmp;
    });

    return list;
});

function toggleSort(key: "cpu" | "memory" | "name") {
    if (sortKey.value === key) {
        sortAsc.value = !sortAsc.value;
    } else {
        sortKey.value = key;
        sortAsc.value = false;
    }
}

function getSortIcon(key: "cpu" | "memory" | "name") {
    if (sortKey.value !== key) return "";
    return sortAsc.value ? "↑" : "↓";
}
</script>

<template>
    <div class="process-list">
        <div class="list-toolbar">
            <div class="search-box">
                <span class="icon-[mdi--magnify] search-icon"></span>
                <input v-model="searchQuery" type="text" placeholder="搜索进程..." class="search-input" />
            </div>
            <div class="process-count">共 {{ filteredProcesses.length }} 个进程</div>
        </div>

        <div class="list-header">
            <span class="col col-pid">PID</span>
            <span class="col col-name sortable" @click="toggleSort('name')">
                进程名 {{ getSortIcon('name') }}
            </span>
            <span class="col col-user">用户</span>
            <span class="col col-cpu sortable" @click="toggleSort('cpu')">
                CPU {{ getSortIcon('cpu') }}
            </span>
            <span class="col col-mem sortable" @click="toggleSort('memory')">
                内存 {{ getSortIcon('memory') }}
            </span>
            <span class="col col-action"></span>
        </div>

        <div class="list-body">
            <div v-for="proc in filteredProcesses" :key="proc.pid" class="list-row">
                <span class="col col-pid">{{ proc.pid }}</span>
                <span class="col col-name">{{ proc.name }}</span>
                <span class="col col-user">{{ proc.user }}</span>
                <span class="col col-cpu" :class="{ high: proc.cpu > 50 }">{{ proc.cpu.toFixed(1) }}%</span>
                <span class="col col-mem" :class="{ high: proc.memory > 50 }">{{ proc.memory.toFixed(1) }}%</span>
                <span class="col col-action">
                    <button class="kill-btn" title="终止进程" @click="emit('kill', proc.pid)">
                        <span class="icon-[mdi--close]"></span>
                    </button>
                </span>
            </div>
            <div v-if="filteredProcesses.length === 0" class="empty-state">
                <span class="icon-[mdi--magnify]"></span>
                <p>未找到匹配的进程</p>
            </div>
        </div>
    </div>
</template>

<style scoped>
.process-list {
    height: 100%;
    display: flex;
    flex-direction: column;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 16px;
    overflow: hidden;
}

.list-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.search-box {
    position: relative;
    display: flex;
    align-items: center;
}

.search-icon {
    position: absolute;
    left: 10px;
    color: rgba(255, 255, 255, 0.4);
    font-size: 16px;
}

.search-input {
    width: 200px;
    padding: 8px 10px 8px 32px;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    color: #fff;
    font-size: 0.8rem;
    outline: none;
    transition: border-color 0.2s;
}

.search-input::placeholder {
    color: rgba(255, 255, 255, 0.3);
}

.search-input:focus {
    border-color: rgba(125, 211, 252, 0.5);
}

.process-count {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.4);
}

.list-header {
    display: grid;
    grid-template-columns: 70px 2fr 1fr 80px 80px 50px;
    gap: 8px;
    padding: 10px 16px;
    background: rgba(255, 255, 255, 0.03);
    font-size: 0.72rem;
    color: rgba(255, 255, 255, 0.5);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.sortable {
    cursor: pointer;
    user-select: none;
}

.sortable:hover {
    color: rgba(255, 255, 255, 0.7);
}

.list-body {
    flex: 1;
    overflow-y: auto;
}

.list-row {
    display: grid;
    grid-template-columns: 70px 2fr 1fr 80px 80px 50px;
    gap: 8px;
    padding: 10px 16px;
    font-size: 0.8rem;
    color: rgba(255, 255, 255, 0.8);
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
    transition: background 0.2s;
}

.list-row:hover {
    background: rgba(255, 255, 255, 0.04);
}

.col-pid {
    color: rgba(255, 255, 255, 0.5);
    font-family: monospace;
}

.col-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.col-user {
    color: rgba(255, 255, 255, 0.5);
}

.col-cpu,
.col-mem {
    text-align: right;
    font-family: monospace;
}

.col-cpu.high,
.col-mem.high {
    color: #f59e0b;
}

.kill-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: rgba(255, 107, 107, 0.1);
    color: #ff6b6b;
    border-radius: 6px;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.2s, background 0.2s;
}

.list-row:hover .kill-btn {
    opacity: 1;
}

.kill-btn:hover {
    background: rgba(255, 107, 107, 0.25);
}

.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: rgba(255, 255, 255, 0.3);
    gap: 12px;
}

.empty-state span {
    font-size: 32px;
}

.empty-state p {
    font-size: 0.85rem;
}
</style>
