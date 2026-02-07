---
name: activity-monitor-enhancement
overview: 增强活动监视器功能，显示真实系统数据和详细进程信息，类似 macOS 活动监视器
design:
  architecture:
    framework: vue
  styleKeywords:
    - Dark Theme
    - Glassmorphism
    - System Monitor
    - Real-time Charts
    - Professional
  fontSystem:
    fontFamily: Inter
    heading:
      size: 1.25rem
      weight: 600
    subheading:
      size: 0.9rem
      weight: 500
    body:
      size: 0.8rem
      weight: 400
  colorSystem:
    primary:
      - "#34d399"
      - "#fbbf24"
      - "#60a5fa"
      - "#f59e0b"
    background:
      - "#0e121c"
      - rgba(255, 255, 255, 0.02)
      - rgba(255, 255, 255, 0.05)
    text:
      - rgba(255, 255, 255, 0.9)
      - rgba(255, 255, 255, 0.7)
      - rgba(255, 255, 255, 0.5)
      - rgba(255, 255, 255, 0.3)
    functional:
      - "#ff6b6b"
      - "#10b981"
      - "#7dd3fc"
todos:
  - id: extend-process-struct
    content: 扩展 Rust ProcessInfo 结构体，添加详细进程字段
    status: completed
  - id: enhance-process-command
    content: 优化进程数据获取命令，解析更多进程信息
    status: completed
    dependencies:
      - extend-process-struct
  - id: add-kill-command
    content: 实现 kill_process Tauri 命令，支持信号选择
    status: completed
  - id: enhance-process-list
    content: 重构 ProcessList 组件，支持多列排序和列配置
    status: completed
    dependencies:
      - extend-process-struct
  - id: add-process-detail
    content: 创建 ProcessDetail 详情抽屉组件
    status: completed
    dependencies:
      - enhance-process-list
  - id: integrate-kill-feature
    content: 集成进程终止功能到前端界面
    status: completed
    dependencies:
      - add-kill-command
      - add-process-detail
  - id: optimize-charts
    content: 优化 CPU 和 Network 图表，增加多核和接口详情显示
    status: completed
---

## 产品概述

SynapSH 是一个基于 Tauri + Vue 3 的远程 SSH 服务器管理工具。当前已实现活动监视器的基础框架，但进程数据仅显示基本信息（PID、名称、用户、CPU%、内存%），需要增强为类似 macOS 活动监视器的完整功能。

## 核心功能

1. **进程详细信息扩展**：显示进程状态、启动时间、运行时长、线程数、物理内存、虚拟内存、命令行参数等
2. **进程管理功能**：实现进程终止（kill）功能，支持信号类型选择
3. **进程列表增强**：支持多列排序、列显示/隐藏、进程过滤
4. **系统数据丰富**：增强 CPU、内存、磁盘、网络数据的展示细节
5. **进程详情面板**：点击进程显示详细信息弹窗/抽屉

## 技术栈

- **前端框架**：Vue 3 + TypeScript + `<script setup>` 语法
- **桌面框架**：Tauri v2 + Rust
- **图表库**：ECharts
- **样式**：Tailwind CSS v4
- **图标**：Iconify (MDI 图标集)

## 实现方案

### 后端增强 (Rust)

1. **扩展 ProcessInfo 结构体**：添加状态、启动时间、线程数、物理内存、虚拟内存、命令行等字段
2. **优化进程数据获取命令**：使用 `ps` 或读取 `/proc/[pid]/stat` 获取更详细的进程信息
3. **实现 kill_process 命令**：支持发送不同信号（SIGTERM、SIGKILL 等）终止进程

### 前端增强

1. **ProcessList 组件重构**：

- 添加可配置列显示（支持显示/隐藏列）
- 增强排序功能（支持所有字段排序）
- 添加进程行点击展开详情
- 优化表格布局，支持水平滚动

2. **新增 ProcessDetail 组件**：

- 抽屉/弹窗形式展示进程完整信息
- 显示命令行参数、环境变量、打开文件等

3. **系统监控图表优化**：

- CPU 图表增加多核显示选项
- 网络图表显示各接口详情
- 磁盘图表增加 I/O 速率监控

### 数据流设计

```
SSH 会话 → exec_command → 解析系统命令输出 → SystemStats 结构体 → 前端组件
```

### 性能优化

- 进程列表使用虚拟滚动（如进程数过多）
- 数据获取使用防抖，避免频繁刷新
- 大列表使用 `shallowRef` 减少响应式开销

## 目录结构

```
src/
├── components/
│   └── apps/
│       ├── monitor/
│       │   ├── CpuChart.vue              # [MODIFY] 支持多核显示
│       │   ├── MemoryChart.vue           # [EXISTING]
│       │   ├── DiskChart.vue             # [EXISTING]
│       │   ├── NetworkChart.vue          # [MODIFY] 增加接口详情
│       │   ├── ProcessList.vue           # [MODIFY] 增强列和交互
│       │   ├── ProcessDetail.vue         # [NEW] 进程详情抽屉
│       │   └── SystemOverview.vue        # [EXISTING]
│       └── ActivityMonitor.vue           # [MODIFY] 集成新组件
src-tauri/
└── src/
    └── lib.rs                            # [MODIFY] 扩展 ProcessInfo，添加 kill 命令
```

## 关键数据结构

```rust
// 扩展后的 ProcessInfo
pub struct ProcessInfo {
    pid: u32,
    name: String,
    cpu: f64,
    memory: f64,          // 内存百分比
    user: String,
    // 新增字段
    status: String,       // 进程状态 (R/S/Z等)
    start_time: String,   // 启动时间
    elapsed_time: String, // 运行时长
    threads: u32,         // 线程数
    rss: u64,            // 物理内存 (KB)
    vsz: u64,            // 虚拟内存 (KB)
    command: String,      // 完整命令行
    ppid: u32,           // 父进程ID
    nice: i32,           // 优先级
}
```

## 设计风格

采用深色主题的现代化系统监控界面，类似 macOS 活动监视器的专业感。使用半透明玻璃态效果（glassmorphism）和微妙的渐变色彩区分不同数据类型。

### 设计要点

1. **深色主题**：深蓝灰色背景 (#0e121c)，白色文字，高对比度确保可读性
2. **数据可视化**：使用 ECharts 绘制流畅的实时图表，CPU 使用绿色渐变，内存使用橙色渐变，网络使用蓝色/黄色区分上传下载
3. **表格设计**：进程列表采用紧凑的行高，悬停显示操作按钮，高亮高 CPU/内存进程
4. **详情抽屉**：右侧滑出式详情面板，展示进程完整信息，支持快速操作
5. **响应式布局**：自适应不同窗口大小，小屏幕下图表堆叠显示