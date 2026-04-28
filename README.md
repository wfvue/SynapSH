<p align="center">
  <img src="public/logo.svg" width="80" height="80" alt="SynapSH Logo">
</p>

<h1 align="center">SynapSH · 光析</h1>

<p align="center">
  <strong>下一代服务器可视化管理桌面</strong><br>
  把命令行里的一切，变成看得见、点得着的操作
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Electron-33-47848F?logo=electron&logoColor=white" alt="Electron">
  <img src="https://img.shields.io/badge/Vue-3.6-4FC08D?logo=vue.js&logoColor=white" alt="Vue">
  <img src="https://img.shields.io/badge/TypeScript-5.6-3178C6?logo=typescript&logoColor=white" alt="TypeScript">
  <img src="https://img.shields.io/badge/TailwindCSS-4-06B6D4?logo=tailwindcss&logoColor=white" alt="TailwindCSS">
  <img src="https://img.shields.io/badge/License-Private-red" alt="License">
</p>

---

## 💡 这是什么

**SynapSH（光析）** 是一个基于 Electron 的现代化 SSH 客户端，核心理念是：**让服务器管理从"敲命令"变成"看图操作"。**

整个界面设计成一个类 macOS 的桌面环境，每台服务器就是桌面上的一个窗口。文件拖拽传输、实时性能图表、在线代码编辑、数据库管理——所有以前要靠命令行完成的事，现在都有图形化界面。

## ✨ 核心特性

### 🖥️ 桌面级体验
- **类 macOS 桌面环境**：窗口拖拽、缩放、最大化，多窗口自由排列
- **Dock 栏快捷启动**：一键打开终端、文件管理器、监控等应用
- **多机器快速切换**：不用再开一堆终端窗口，一个桌面管理所有服务器

### 📂 可视化文件管理
- 图形化 SFTP 文件浏览器，像本地文件夹一样操作远程文件
- 支持拖拽上传/下载、批量操作、右键菜单
- 内置代码编辑器（Monaco Editor），服务器上的文件直接在线改

### 📊 实时性能监控
- CPU、内存、磁盘、网络流量实时图表展示
- 进程管理器，一眼看清哪个进程在吃资源
- 异常状态高亮提醒，不用再盯着 `top` 刷数字

### 💻 高性能终端
- 基于 xterm.js + WebGL 渲染的高性能终端
- 多 Tab 终端，一个窗口内管理多个会话
- 支持终端分屏、快捷键自定义

### 🗄️ 数据库管理
- 可视化数据库连接与操作
- 支持远程数据库检测与安装

### 🔐 连接管理
- 本地加密存储服务器连接信息（better-sqlite3）
- 支持密码和密钥认证
- 连接分组与标签管理

## 🛠️ 技术栈

| 类别 | 技术 |
|------|------|
| 桌面框架 | Electron 33 |
| 前端框架 | Vue 3.6 + TypeScript |
| 构建工具 | Vite 8 |
| CSS 框架 | TailwindCSS v4 |
| UI 组件 | shadcn-vue + Reka UI |
| 终端模拟 | xterm.js + WebGL 渲染 |
| 代码编辑 | Monaco Editor |
| 数据可视化 | ECharts 6 |
| SSH 协议 | ssh2 (Node.js) |
| 本地存储 | better-sqlite3 |
| 图标系统 | Iconify (mdi / lucide / carbon) |

## 📦 快速开始

```bash
# 克隆项目
git clone <repo-url>
cd SynapSH

# 安装依赖
pnpm install

# 启动开发模式
pnpm electron:dev

# 构建生产包
pnpm electron:build
```

## 📁 项目结构

```
src/
├── views/                    # 页面
│   ├── MachineManager.vue    # 机器管理（连接列表）
│   ├── DesktopShell.vue      # 桌面环境主界面
│   └── apps/                 # 桌面内置应用
│       ├── TerminalApp.vue        # 终端
│       ├── FilesApp.vue           # 文件管理器
│       ├── ActivityMonitor.vue    # 活动监视器
│       ├── TextEditorApp.vue      # 文本编辑器
│       ├── DatabaseManagerApp.vue # 数据库管理
│       └── SettingsApp.vue        # 系统设置
├── components/               # 可复用组件
│   ├── desktop/              # 桌面 UI 组件（窗口、Dock 等）
│   ├── ConnectionPanel.vue   # 连接面板
│   ├── TabBar.vue            # 标签栏
│   └── ui/                   # shadcn-vue 基础组件
├── composables/              # 组合式函数
├── lib/                      # 工具库
└── style.css                 # 全局样式

electron/
├── main.ts                   # 主进程入口
├── preload.ts                # 预加载脚本（contextBridge）
└── services/
    ├── ssh.ts                # SSH/SFTP 会话管理
    ├── machine-db.ts         # 本地机器数据库
    └── browser.ts            # 浏览器代理
```

## 🎨 设计理念

- **暗色优先**：默认深色主题，减少长时间使用的视觉疲劳
- **Neo-macOS 风格**：毛玻璃质感、精致阴影、流畅动效
- **内容优先**：视觉效果服务于功能，不做无意义的装饰
- **8pt 栅格系统**：统一的间距与圆角规范，界面整洁有序

## 🗺️ Roadmap

- [x] SSH 终端连接与管理
- [x] 图形化 SFTP 文件管理
- [x] 实时服务器性能监控
- [x] 在线代码编辑器
- [x] 数据库可视化管理
- [ ] 🤖 AI 智能运维助手（自然语言驱动的服务器诊断）
- [ ] 多服务器批量操作
- [ ] 插件系统

## 📄 License

Private · © 2026 SynapSH Team
