# SynapSH (光析) - 开发进度

## 项目文档 (2026-07-15) ✅

- [x] README 默认改为英文，并保留完整简体中文版
- [x] README 中英文版本支持顶部链接点击切换
- [x] 重新设计 SynapSH 原创品牌 Logo，替换旧 Tauri 风格图标
- [x] 补充 README 使用的可缩放品牌 Logo 资源
- [x] 重新拍摄英文界面截图并替换英文 README，中文 README 保留原截图

## 首页管理页面优化 (2026-04-01)

### 设计规范对齐 ✅

- [x] 更新全局样式 CSS 变量，符合 Neo-macOS 设计规范
- [x] 暗色主题色彩 Token：`--bg-canvas: #0b0d10`、`--bg-surface: #151922`、`--bg-elevated: #1d2430`、`--accent: #0a84ff`
- [x] 空间规范：侧边栏宽度改为 `w-64`（256px），符合 8pt 栅格系统
- [x] 圆角规范：卡片 `rounded-2xl`（16px）、按钮 `rounded-lg`（8px）、图标容器 `rounded-[10px]`
- [x] 阴影规范：卡片悬浮 `shadow-[0_10px_24px_rgba(0,0,0,0.30)]`、选中态 `shadow-[0_16px_40px_rgba(0,0,0,0.45)]`
- [x] 动效规范：按钮悬停 `120ms`、状态切换 `180ms`，使用语义化缓动函数

### 新增功能 ✅

- [x] **分组功能**：支持按操作系统（Linux/Windows/macOS）或标签分组显示
- [x] **筛选功能**：快速切换显示全部/在线/离线机器
- [x] **批量操作**：
  - [x] 多选模式（点击选择图标或卡片）
  - [x] 批量删除（带确认对话框）
  - [x] 批量测试连接（并发控制，每批 5 台）
  - [x] 全选当前组/清除选择
- [x] **快捷键支持**：
  - [x] `⌘N` - 添加机器
  - [x] `⌘F` - 聚焦搜索框
  - [x] `⌘A` - 全选（选择模式下）
  - [x] `Esc` - 关闭对话框/退出选择模式

### 视觉优化 ✅

- [x] 骨架屏加载动画（替代单一加载图标）
- [x] 在线状态指示灯带脉冲动画
- [x] 标签超过 3 个时 Tooltip 显示全部
- [x] 空状态插图位置优化，增加视觉平衡
- [x] 卡片 hover 状态增强边框和阴影
- [x] 错误提示条使用危险色系设计
- [x] 分组标题使用半透明背景

### 组件开发 ✅

- [x] 创建 Checkbox 组件（用于批量选择）
- [x] 创建语义化组件类：`app-card`、`app-btn`、`app-badge`、`app-skeleton` 等
- [x] 创建状态类：`app-status-online`、`app-status-offline`、`app-status-testing`

### 性能优化 ✅

- [x] 测试连接并发控制（避免同时测试过多机器）
- [x] 标签 Tooltip 懒加载
- [ ] 虚拟滚动（待实现，当机器数 > 100 时启用）

### 其他修复

- [x] better-sqlite3 编译问题：暂时使用内存 fallback，不影响开发
- [x] 修复图标名称问题（使用正确的 Iconify 格式）

## 技术栈迁移 (2026-04-01)

- [x] Tauri v2 + Rust 后端迁移到 Electron 33 + Node.js/TypeScript
- [x] 创建 electron/ 目录结构 (main.ts, preload.ts, services/)
- [x] 迁移 SSH/SFTP 功能 (ssh2 库替代 russh)
- [x] 迁移本地数据库 (better-sqlite3 替代 sqlx)
- [x] 创建 IPC 适配层 (src/lib/api.ts)
- [x] 更新 AGENTS.md 项目规范
- [x] 删除 src-tauri/ 和 .cargo/ 目录

## 工具链升级 (2026-04-02) ✅
- [x] **升级到 Vite 8 / Vite+**：
  - [x] 迁移到基于 Rolldown 的统一构建工具链
  - [x] 更新 `vite.config.ts` 使用 `rolldownOptions` 替代 `rollupOptions`
  - [x] 集成 `staged` 配置块，支持通过 `vp check --fix` 进行 commit 预检
  - [x] 适配 `oxlint` 和 `oxfmt` 替代原有的 lint/format 工具
  - [x] 修复 Electron 插件与 Vite 8 的兼容性依赖（补丁安装 `esbuild`）
  - [x] 全局执行 `vp check --fix` 对齐代码规范

## 当前状态

- [x] 阶段一：连接与终端 (Connectivity)
- [/] 阶段二：桌面式 OS Shell + 文件管理 (Visual OS)
- [ ] 阶段三：监控与 AI (Intelligence)

## 阶段一：连接与终端 ✅

- [x] SSH 会话管理 + connect_ssh / write_to_pty / resize_pty (Electron IPC)
- [x] 终端渲染（xterm.js + WebGL + fit）
- [x] 连接面板 UI（账号/密码/密钥）
- [x] 事件通道 ssh-data-<sessionId> 到前端
- [x] 机器管理页面（SQLite 持久化 + CRUD + 连接测试）

## 阶段二：桌面式 OS Shell + 文件管理

- [x] 桌面 Shell 框架（桌面图标 + Dock + 状态栏 + 窗口容器）
- [x] 文件管理 UI 骨架（路径栏 + 工具栏 + 树 + 列表）
- [x] 浏览器应用（地址栏、导航、历史记录、书签）
- [x] 浏览器代理链路优化（Chrome 非阻塞启动 + SOCKS5 活性检测重建 + 错误事件回传）
- [x] 浏览器代理连通性预检与自动重建（SOCKS5 握手探测 + 失败重试）
- [x] 浏览器交互切换为外部 Chrome 直开（移除应用内浏览器页面）
- [x] SOCKS5 转发并发优化（移除 SSH Handle 串行锁，提升多资源页面加载速度）
- [x] 浏览器单实例管理（同一 session 复用 + 应用退出自动关闭）
- [x] 代理链路性能调优（SSH 参数优化 + copy_bidirectional + 并发限流容错 + 失败目标快速拒绝）
- [x] 浏览器代理切换为系统 OpenSSH -D sidecar（单隧道复用 + sidecar 生命周期托管）
- [x] OpenSSH sidecar 稳定性修复（就绪等待 + 自动重建 + stderr 诊断）
- [x] OpenSSH 浏览器启动提速（预检改为本地 SOCKS 握手，避免目标站点阻塞）
- [x] OpenSSH 多 sidecar 并行分流（会话级连接池 + 轮询端口分配）
- [x] 系统设置模块（通用/外观/终端/连接/关于）
- [x] 关于页介绍支持中英文切换（默认英文）
- [x] 界面语言全局化：英文/中文即时切换并持久化，覆盖机器管理、桌面、顶部操作栏与系统设置
- [x] 顶部 Tab 栏适配原生窗口控件（macOS/Windows）+ 右侧系统外观快捷设置
- [x] Tab 交互优化：垂直居中、`+` 紧随标签，固定"机器管理"主标签、连接后按机器名建会话标签
- [x] 修复 TabBar 外观弹窗配色（按语义化 Token 统一背景/边框/文字/激活态）
- [x] 升级 Electron 技术栈
- [x] 重新定义 AGENTS.md 美学 UI 规范（视觉层级 + 语义化 Token + 动效与可访问性约束）
- [ ] SFTP read_dir(path) 后端实现 + JSON 结构 (需验证)
- [ ] 文件操作：上传 / 下载 / 新建 / 重命名 / 删除 (需验证)
- [ ] get_dir_size(path) 递归统计（用于可视化）
- [ ] Sunburst 可视化视图

## 阶段三：监控与 AI

- [x] /proc 监控拉取与解析（CPU / 内存 / 网卡 / 进程）
- [x] Codex SDK 接入（复用本机 ChatGPT/Codex 订阅登录 + 主进程 IPC 调用）
- [x] AI 运维助手桌面应用（连续对话、中英文、快捷诊断、服务器实时指标分析）
- [x] 监控面板（ECharts 仪表盘）
- [x] 活动监视器增强：
  - [x] 扩展 ProcessInfo 结构体，添加详细进程字段（状态、线程数、内存、优先级等）
  - [x] 优化进程数据获取命令，使用 ps 获取更详细信息
  - [x] 实现 kill_process 命令，支持多种信号（SIGTERM/SIGKILL/SIGSTOP/SIGCONT）
  - [x] 重构 ProcessList 组件，支持多列排序和搜索
  - [x] 创建 ProcessDetail 详情抽屉组件
  - [x] 集成进程终止功能到前端界面
  - [x] 优化 CPU 图表，显示核心数和统计信息
- [ ] Sidecar llama-server 启动
- [ ] 本地 HTTP 调用链打通
