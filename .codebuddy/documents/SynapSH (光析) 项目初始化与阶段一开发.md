## 项目概述

基于 Tauri v2 + Vue 3 + TypeScript + Vite 构建的现代化 SSH 客户端，代号"光析"。

## 技术栈

- **前端**: Vue 3.6 + TypeScript + Vite 8 + shadcn-ui + xterm.js + ECharts
- **后端**: Tauri v2 (Rust) + russh + tokio
- **包管理器**: pnpm

## 实施计划

### 第一阶段：项目初始化

1. 使用 `pnpm create tauri-app` 创建项目骨架
2. 选择 Vue + TypeScript 模板
3. 配置项目基础结构

### 第二阶段：依赖安装与配置

1. 安装前端依赖：
   - xterm.js + xterm-addon-webgl + xterm-addon-fit
   - ECharts
   - @tauri-apps/api
   - shadcn-ui 相关依赖
2. 配置 Rust 依赖 (Cargo.toml)：
   - russh, russh-keys, russh-sftp
   - tokio
   - anyhow
   - serde

### 第三阶段：阶段一功能 - 连接与终端

1. **后端 (Rust)**:
   - 创建 `SSHSession` 结构体封装 russh 连接
   - 实现 `connect_ssh` Tauri Command
   - 实现 `write_to_pty` 和 `read_from_pty` 命令
   - 使用 `tokio::sync::Mutex` 管理会话状态

2. **前端 (Vue)**:
   - 创建 Terminal.vue 组件
   - 集成 xterm.js 并启用 WebGL 渲染
   - 实现与 Rust 后端的 IPC 通信
   - 创建连接配置界面

### 第四阶段：构建与验证

1. 运行 `pnpm tauri dev` 验证开发环境
2. 测试 SSH 连接功能
3. 验证终端输入输出

## 预期产出

- 可运行的 Tauri 应用
- 基础的 SSH 连接功能
- 功能完整的终端界面
- 项目基础架构搭建完成

请确认此计划后，我将开始执行项目初始化。
